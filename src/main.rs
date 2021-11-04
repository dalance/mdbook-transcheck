mod config;
mod fixer;
mod linter;
mod matcher;
mod printer;
mod util;

use crate::config::Config;
use crate::fixer::Fixer;
use crate::linter::Linter;
use crate::matcher::Matcher;
use crate::printer::Printer;
use crate::util::print_error;
use anyhow::{Context, Error};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process;
use structopt::{clap, StructOpt};

// ---------------------------------------------------------------------------------------------------------------------
// Opt
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Debug, StructOpt)]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
#[structopt(setting(clap::AppSettings::DeriveDisplayOrder))]
pub struct Opt {
    /// Source directory ( or file if single file mode )
    #[structopt(name = "SOURCE")]
    pub source: PathBuf,

    /// Target directory ( or file if single file mode )
    #[structopt(name = "TARGET")]
    pub target: PathBuf,

    /// Apply auto fix
    #[structopt(long = "fix")]
    pub fix: bool,

    /// Lint check
    #[structopt(long = "lint")]
    pub lint: bool,

    /// Prints verbose message
    #[structopt(short = "v", long = "verbose")]
    pub verbose: bool,

    /// Dry run
    #[structopt(long = "dry-run")]
    pub dry_run: bool,

    /// Single file mode
    #[structopt(short = "1", long = "single")]
    pub single: bool,

    /// Config file
    #[structopt(long = "config", default_value = "transcheck.toml")]
    pub config: PathBuf,

    /// Color mode
    #[structopt(
        short = "c",
        long = "color",
        possible_value = "auto",
        possible_value = "always",
        possible_value = "disable",
        default_value = "auto"
    )]
    pub color: String,
}

// ---------------------------------------------------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------------------------------------------------

#[cfg_attr(tarpaulin, skip)]
fn main() {
    match run() {
        Ok(x) => {
            if !x {
                process::exit(1);
            }
        }
        Err(x) => {
            print_error(x);
            process::exit(1);
        }
    }
}

fn run() -> Result<bool, Error> {
    let opt = Opt::from_args();

    match opt.color.as_str() {
        "auto" => console::set_colors_enabled(console::Term::stdout().is_term()),
        "always" => console::set_colors_enabled(true),
        "disable" => console::set_colors_enabled(false),
        _ => unreachable!(),
    }

    let config = search_config(&opt.config);

    let config = if let Some(config) = config {
        let mut f = File::open(&config)
            .with_context(|| format!("Failed to open '{}'", config.to_string_lossy()))?;
        let mut s = String::new();
        let _ = f.read_to_string(&mut s);
        let ret = toml::from_str(&s)
            .with_context(|| format!("Failed to parse toml '{}'", config.to_string_lossy()))?;

        ret
    } else {
        Config::default()
    };

    let matcher = Matcher {
        enable_code_comment_tweak: config.matcher.enable_code_comment_tweak,
        code_comment_header: config.matcher.code_comment_header,
        keep_markdown_comment: config.matcher.keep_markdown_comment,
        markdown_comment_begin: config.matcher.markdown_comment_begin,
        markdown_comment_end: config.matcher.markdown_comment_end,
        similar_threshold: config.matcher.similar_threshold,
    };

    let linter = Linter {
        enable_emphasis_check: config.linter.enable_emphasis_check,
        enable_half_paren_check: config.linter.enable_half_paren_check,
        enable_full_paren_check: config.linter.enable_full_paren_check,
    };

    let printer = Printer {
        verbose: opt.verbose,
    };

    let excludes: Vec<_> = config.excludes.iter().map(|x| x.into()).collect();
    let (mismatches, target_onlys) = if opt.single {
        let (mismatch, target_only) = matcher.check_file(&opt.source, &opt.target)?;
        (vec![mismatch], vec![target_only])
    } else {
        matcher.check_dir(&opt.source, &opt.target, &excludes)?
    };

    let success = if opt.fix {
        let fixer = Fixer {
            dry_run: opt.dry_run,
        };
        fixer.fix(&mismatches)?
    } else if opt.lint {
        let lint_errors = linter.check(target_onlys)?;
        printer.print_lint(&lint_errors)?
    } else {
        printer.print_mismatch(&mismatches)?
    };

    Ok(success)
}

#[cfg_attr(tarpaulin, skip)]
fn search_config(path: &Path) -> Option<PathBuf> {
    if let Ok(current) = env::current_dir() {
        for dir in current.ancestors() {
            let candidate = dir.join(path);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        None
    } else {
        None
    }
}

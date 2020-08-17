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
use anyhow::{Context, Error};
use console::style;
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
    /// Source directory
    #[structopt(name = "SOURCE")]
    pub source: PathBuf,

    /// Target directory
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

    /// Config file
    #[structopt(long = "config", default_value = "transcheck.toml")]
    pub config: PathBuf,
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
            let mut cause = x.chain();
            eprintln!(
                "{}{}",
                style("Error").red().bold(),
                style(format!(": {}", cause.next().unwrap())).white().bold()
            );

            for x in cause {
                let _ = eprintln!(
                    "  {}{}",
                    console::style("caused by: ").white().bold(),
                    console::style(x).white()
                );
            }

            process::exit(1);
        }
    }
}

fn run() -> Result<bool, Error> {
    let opt = Opt::from_args();

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

    let (mismatches, target_onlys) = matcher.check_dir(&opt.source, &opt.target)?;

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

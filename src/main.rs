mod matcher;
mod printer;

use crate::matcher::Matcher;
use crate::printer::Printer;
use anyhow::Error;
use console::style;
use std::path::PathBuf;
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

    let matcher = Matcher {
        code_comment: true,
        similar_threshold: 0.5,
    };
    let printer = Printer { verbose: false };

    let mismatches = matcher.check_dir(&opt.source, &opt.target)?;
    let success = printer.print(&mismatches)?;
    Ok(success)
}

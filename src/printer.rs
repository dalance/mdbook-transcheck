use crate::linter::{LintError, LintErrorKind};
use crate::matcher::{Line, Mismatch, MismatchLines, MissingFile};
use crate::util::{combine_line, CombinedLine};
use anyhow::Error;
use console::style;

pub struct Printer {
    pub verbose: bool,
}

impl Printer {
    pub fn print_mismatch(&self, mismatches: &[Mismatch]) -> Result<bool, Error> {
        let mut ret = true;
        for mismatch in mismatches {
            match mismatch {
                Mismatch::MissingFile(x) => {
                    ret &= self.print_missing_file(x)?;
                }
                Mismatch::MismatchLines(x) => {
                    ret &= self.print_mismatch_lines(x)?;
                }
            }
        }
        Ok(ret)
    }

    pub fn print_lint(&self, lint_errors: &[LintError]) -> Result<bool, Error> {
        let mut ret = true;
        for error in lint_errors {
            self.print_lint_error(error);
            ret = false;
        }
        Ok(ret)
    }

    fn print_missing_file(&self, missing: &MissingFile) -> Result<bool, Error> {
        println!(
            "\n{}{}",
            style("Error").red().bold(),
            style(": target path is not found").white().bold()
        );
        println!(
            "    source path: {}",
            style(missing.source_path.to_string_lossy()).white()
        );
        println!(
            "    target path: {}\n",
            style(missing.target_path.to_string_lossy()).white()
        );
        Ok(false)
    }

    fn print_mismatch_lines(&self, mismatch: &MismatchLines) -> Result<bool, Error> {
        if mismatch.lines.is_empty() {
            Ok(true)
        } else {
            let lines = combine_line(mismatch);

            for line in &lines {
                match line {
                    CombinedLine::Modified((x, y)) => {
                        self.print_modified_line(mismatch, x, y)?;
                    }
                    CombinedLine::Missing(x) => {
                        self.print_missing_line(mismatch, x.as_slice())?;
                    }
                    CombinedLine::Garbage(x) => {
                        self.print_garbage_line(mismatch, x.as_slice())?;
                    }
                }
            }
            Ok(false)
        }
    }

    fn print_modified_line(
        &self,
        mismatch: &MismatchLines,
        source: &Line,
        target: &Line,
    ) -> Result<(), Error> {
        let mut source_anno = String::from("");
        let mut target_anno = String::from("");
        for diff in diff::chars(&source.content, &target.content) {
            match diff {
                diff::Result::Both(x, _) => {
                    let witdh = console::measure_text_width(&format!("{}", x));
                    source_anno.push_str(&" ".repeat(witdh));
                    target_anno.push_str(&" ".repeat(witdh));
                }
                diff::Result::Left(x) => {
                    let witdh = console::measure_text_width(&format!("{}", x));
                    source_anno.push_str(&"^".repeat(witdh));
                }
                diff::Result::Right(x) => {
                    let witdh = console::measure_text_width(&format!("{}", x));
                    target_anno.push_str(&"^".repeat(witdh));
                }
            }
        }

        println!(
            "\n{}{}",
            style("Error").red().bold(),
            style(": source line has been modified").white().bold()
        );
        println!(
            "{}{}",
            style(" source --> ").blue().bold(),
            style(format!(
                "{}:{}",
                mismatch.source_path.to_string_lossy(),
                source.number
            ))
            .white()
        );
        let number = format!("{}", source.number);
        let number_space = " ".repeat(number.len());
        println!("{}", style(format!("{} |", number_space)).blue().bold());
        println!(
            "{}{}",
            style(format!("{} | ", number)).blue().bold(),
            style(&source.content).white(),
        );
        println!(
            "{}{}",
            style(format!("{} | ", number_space)).blue().bold(),
            style(&source_anno).yellow().bold(),
        );
        println!("{}", style(format!("{} |\n", number_space)).blue().bold());

        println!(
            "{}{}",
            style(" target --> ").blue().bold(),
            style(format!(
                "{}:{}",
                mismatch.target_path.to_string_lossy(),
                target.number
            ))
            .white()
        );
        let number = format!("{}", target.number);
        let number_space = " ".repeat(number.len());
        println!("{}", style(format!("{} |", number_space)).blue().bold());
        println!(
            "{}{}",
            style(format!("{} | ", number)).blue().bold(),
            style(&target.content).white(),
        );
        println!(
            "{}{}",
            style(format!("{} | ", number_space)).blue().bold(),
            style(&target_anno).yellow().bold(),
        );
        println!("{}", style(format!("{} |\n", number_space)).blue().bold());
        Ok(())
    }

    fn print_missing_line(&self, mismatch: &MismatchLines, sources: &[&Line]) -> Result<(), Error> {
        println!(
            "\n{}{}",
            style("Error").red().bold(),
            style(": lines has been inserted to the source file")
                .white()
                .bold()
        );
        println!(
            "{}{}",
            style(" source --> ").blue().bold(),
            style(format!(
                "{}:{}",
                mismatch.source_path.to_string_lossy(),
                sources[0].number
            ))
            .white()
        );

        let mut max_width = 0;
        for source in sources {
            let number = format!("{}", source.number);
            max_width = std::cmp::max(number.len(), max_width);
        }
        let number_space = " ".repeat(max_width);

        println!("{}", style(format!("{} |", number_space)).blue().bold());
        for source in sources {
            let number = format!("{}", source.number);
            println!(
                "{}{}",
                style(format!(
                    "{}{} | ",
                    number,
                    " ".repeat(max_width - number.len())
                ))
                .blue()
                .bold(),
                style(&source.content).white(),
            );
        }
        println!("{}", style(format!("{} |", number_space)).blue().bold());
        println!(
            "{}{}{}",
            style(format!("{} =", number_space)).blue().bold(),
            style(" hint").yellow().bold(),
            style(format!(
                ": The lines should be inserted at {}:{}\n",
                mismatch.target_path.to_string_lossy(),
                sources[0].last_both
            ))
            .white()
            .bold()
        );
        Ok(())
    }

    fn print_garbage_line(&self, mismatch: &MismatchLines, targets: &[&Line]) -> Result<(), Error> {
        println!(
            "\n{}{}",
            style("Error").red().bold(),
            style(": lines has been removed from the source file")
                .white()
                .bold()
        );
        println!(
            "{}{}",
            style(" target --> ").blue().bold(),
            style(format!(
                "{}:{}",
                mismatch.target_path.to_string_lossy(),
                targets[0].number
            ))
            .white()
        );

        let mut max_width = 0;
        for target in targets {
            let number = format!("{}", target.number);
            max_width = std::cmp::max(number.len(), max_width);
        }
        let number_space = " ".repeat(max_width);

        println!("{}", style(format!("{} |", number_space)).blue().bold());
        for target in targets {
            let number = format!("{}", target.number);
            println!(
                "{}{}",
                style(format!(
                    "{}{} | ",
                    number,
                    " ".repeat(max_width - number.len())
                ))
                .blue()
                .bold(),
                style(&target.content).white(),
            );
        }
        println!("{}", style(format!("{} |", number_space)).blue().bold());
        Ok(())
    }

    fn print_lint_error(&self, error: &LintError) {
        let message = match error.kind {
            LintErrorKind::EmphasisWithoutSpace => "emphasis must have spaces before and after it",
            LintErrorKind::EmphasisMismatch => "emphasis token is mismatched",
            LintErrorKind::HalfParenWithNonAscii => "non-ascii string must have full-width paren",
            LintErrorKind::FullParenWithoutNonAscii => "ascii string must have half-width paren",
        };

        println!(
            "\n{}{}",
            style("Error").red().bold(),
            style(format!(": {}", message)).white().bold()
        );
        println!(
            "{}{}",
            style(" target --> ").blue().bold(),
            style(format!(
                "{}:{}",
                error.path.to_string_lossy(),
                error.line.number
            ))
            .white()
        );

        let number = format!("{}", error.line.number);
        let number_space = " ".repeat(number.len());

        let before_mark =
            console::measure_text_width(&format!("{}", &error.line.content[..error.start]));
        let mark = console::measure_text_width(&format!(
            "{}",
            &error.line.content[error.start..error.end]
        ));

        println!("{}", style(format!("{} |", number_space)).blue().bold());
        println!(
            "{}{}",
            style(format!("{} | ", number)).blue().bold(),
            style(&error.line.content).white(),
        );
        println!(
            "{}{}{}",
            style(format!("{} | ", number_space)).blue().bold(),
            " ".repeat(before_mark),
            style("^".repeat(mark)).yellow().bold(),
        );
        println!("{}", style(format!("{} |", number_space)).blue().bold());
    }
}

use crate::differ::{Line, Missing, MissingFile, MissingLines};
use anyhow::Error;
use console::style;

pub struct Printer {
    pub verbose: bool,
}

impl Printer {
    pub fn print(&self, missings: &[Missing]) -> Result<bool, Error> {
        let mut ret = true;
        for missing in missings {
            match missing {
                Missing::File(x) => {
                    ret = ret & self.print_file(x)?;
                }
                Missing::Lines(x) => {
                    ret = ret & self.print_lines(x)?;
                }
            }
        }
        Ok(ret)
    }

    fn print_file(&self, missing: &MissingFile) -> Result<bool, Error> {
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

    fn print_lines(&self, missing: &MissingLines) -> Result<bool, Error> {
        if missing.lines.is_empty() {
            Ok(true)
        } else {
            for line in &missing.lines {
                if let Some(ref target) = line.target {
                    self.print_modified_line(missing, &line.source, target)?;
                } else {
                    self.print_missing_line(missing, &line.source)?;
                }
            }
            Ok(false)
        }
    }

    fn print_modified_line(
        &self,
        missing: &MissingLines,
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
            style(": target line is modifies").white().bold()
        );
        println!(
            "{}{}",
            style(" source --> ").blue().bold(),
            style(format!(
                "{}:{}",
                missing.source_path.to_string_lossy(),
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
                missing.target_path.to_string_lossy(),
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

    fn print_missing_line(&self, missing: &MissingLines, source: &Line) -> Result<(), Error> {
        println!(
            "\n{}{}",
            style("Error").red().bold(),
            style(": source line is missing").white().bold()
        );
        println!(
            "{}{}",
            style(" source --> ").blue().bold(),
            style(format!(
                "{}:{}",
                missing.source_path.to_string_lossy(),
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
        println!("{}", style(format!("{} |\n", number_space)).blue().bold());
        Ok(())
    }
}

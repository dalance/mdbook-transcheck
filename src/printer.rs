use crate::differ::{Line, Missing, MissingFile, MissingLines};
use anyhow::Error;
use console::style;

pub struct Printer {
    pub verbose: bool,
}

#[derive(Debug)]
pub enum PrintLine<'a> {
    Modified((&'a Line, &'a Line)),
    Missing(Vec<&'a Line>),
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
            let mut lines = Vec::new();
            let mut neighbor = Vec::new();
            let mut prev_line = None;
            for line in &missing.lines {
                if let Some(ref target) = line.target {
                    lines.push(PrintLine::Modified((&line.source, target)));
                } else {
                    match prev_line {
                        Some(x) => {
                            if x + 1 == line.source.number {
                                neighbor.push(&line.source);
                            } else {
                                if !neighbor.is_empty() {
                                    lines.push(PrintLine::Missing(neighbor.clone()));
                                }
                                neighbor.clear();
                                neighbor.push(&line.source);
                            }
                        }
                        None => {
                            neighbor.push(&line.source);
                        }
                    }
                    prev_line = Some(line.source.number);
                }
            }
            if !neighbor.is_empty() {
                lines.push(PrintLine::Missing(neighbor.clone()));
            }

            for line in &lines {
                match line {
                    PrintLine::Modified((x, y)) => {
                        self.print_modified_line(missing, x, y)?;
                    }
                    PrintLine::Missing(x) => {
                        self.print_missing_line(missing, x.as_slice())?;
                    }
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
            style(": source line has been modified").white().bold()
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

    fn print_missing_line(&self, missing: &MissingLines, sources: &[&Line]) -> Result<(), Error> {
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
                missing.source_path.to_string_lossy(),
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
            println!(
                "{}{}",
                style(format!("{} | ", source.number)).blue().bold(),
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
                missing.target_path.to_string_lossy(),
                sources[0].last_both
            ))
            .white()
            .bold()
        );
        Ok(())
    }
}

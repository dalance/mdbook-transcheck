use crate::matcher::{Mismatch, MismatchLines, MissingFile};
use crate::util::{combine_line, CombinedLine};
use anyhow::{Context, Error};
use console::style;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};

pub struct Fixer {
    pub verbose: bool,
}

impl Fixer {
    pub fn fix(&self, mismatches: &[Mismatch]) -> Result<bool, Error> {
        let mut ret = true;
        for mismatch in mismatches {
            match mismatch {
                Mismatch::MissingFile(x) => {
                    ret = ret & self.fix_file(x)?;
                }
                Mismatch::MismatchLines(x) => {
                    ret = ret & self.fix_lines(x)?;
                }
            }
        }
        Ok(ret)
    }

    fn fix_file(&self, missing: &MissingFile) -> Result<bool, Error> {
        if self.verbose {
            println!(
                "{}{}",
                style("  Copy").green().bold(),
                style(format!(
                    ": {} -> {}",
                    missing.source_path.to_string_lossy(),
                    missing.target_path.to_string_lossy()
                ))
                .white()
                .bold()
            );
        }
        fs::copy(&missing.source_path, &missing.target_path)?;
        Ok(true)
    }

    fn log(&self, header: &str, message: &str) -> Result<(), Error> {
        if self.verbose {
            println!(
                "{}{}",
                style(header).green().bold(),
                style(format!(": {}", message)).white().bold()
            );
        }
        Ok(())
    }

    fn fix_lines(&self, mismatch: &MismatchLines) -> Result<bool, Error> {
        if !mismatch.lines.is_empty() {
            let mut lines = combine_line(mismatch);

            let mut target = String::new();
            let target_path = &mismatch.target_path;

            {
                let mut target_reader =
                    BufReader::new(File::open(target_path).with_context(|| {
                        format!("Failed to open '{}'", target_path.to_string_lossy())
                    })?);
                target_reader.read_to_string(&mut target).with_context(|| {
                    format!("Failed to read '{}'", target_path.to_string_lossy())
                })?;
            }

            // sort by line number
            lines.sort_by_key(|x| match x {
                CombinedLine::Modified(x) => x.1.number,
                CombinedLine::Missing(x) => x[0].last_both,
                CombinedLine::Garbage(x) => x[0].number,
            });

            let mut iter = lines.iter().peekable();
            let mut modified = String::new();
            let mut removed_numbers = Vec::new();

            for (i, line) in target.lines().enumerate() {
                let number = i + 1;
                if removed_numbers.contains(&number) {
                    continue;
                }

                match iter.peek() {
                    Some(CombinedLine::Modified(x)) => {
                        if x.1.number == number {
                            self.log(
                                "Modify",
                                &format!("{}:{}", target_path.to_string_lossy(), number),
                            )?;
                            modified.push_str(&format!("{}\n", x.0.content));
                            iter.next();
                        } else {
                            modified.push_str(&format!("{}\n", line));
                        }
                    }
                    Some(CombinedLine::Missing(x)) => {
                        if x[0].last_both == number {
                            self.log(
                                "Insert",
                                &format!("{}:{}", target_path.to_string_lossy(), number),
                            )?;
                            modified.push_str(&format!("{}\n", line));
                            for line in x {
                                modified.push_str(&format!("{}\n", line.content));
                            }
                            iter.next();
                        } else {
                            modified.push_str(&format!("{}\n", line));
                        }
                    }
                    Some(CombinedLine::Garbage(x)) => {
                        if x[0].number == number {
                            self.log(
                                "Remove",
                                &format!("{}:{}", target_path.to_string_lossy(), number),
                            )?;
                            for line in &x[1..] {
                                removed_numbers.push(line.number);
                            }
                            iter.next();
                        } else {
                            modified.push_str(&format!("{}\n", line));
                        }
                    }
                    None => {
                        modified.push_str(&format!("{}\n", line));
                    }
                }
            }
            dbg!(&modified);

            let mut target_writer =
                BufWriter::new(File::create(target_path).with_context(|| {
                    format!("Failed to open '{}'", target_path.to_string_lossy())
                })?);
            target_writer
                .write_all(modified.as_bytes())
                .with_context(|| format!("Failed to write '{}'", target_path.to_string_lossy()))?;
            target_writer.flush()?;
        }
        Ok(true)
    }
}

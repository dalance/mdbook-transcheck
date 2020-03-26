use crate::matcher::{Line, TargetOnly};
use anyhow::Error;
use regex::Regex;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct LintError {
    pub kind: LintErrorKind,
    pub path: PathBuf,
    pub line: Line,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug)]
pub enum LintErrorKind {
    EmphasisWithoutSpace,
    EmphasisMismatch,
    HalfParenWithNonAscii,
    FullParenWithoutNonAscii,
}

#[derive(Clone, Debug)]
pub struct Linter {
    pub enable_emphasis_check: bool,
    pub enable_half_paren_check: bool,
    pub enable_full_paren_check: bool,
}

impl Linter {
    pub fn check(&self, target_onlys: Vec<TargetOnly>) -> Result<Vec<LintError>, Error> {
        let mut ret = Vec::new();

        for target_only in target_onlys {
            for line in &target_only.lines {
                if self.enable_emphasis_check {
                    ret.append(&mut self.check_emphasis(line, &target_only.target_path));
                }
                if self.enable_half_paren_check {
                    ret.append(&mut self.check_half_paren(line, &target_only.target_path));
                }
                if self.enable_full_paren_check {
                    ret.append(&mut self.check_full_paren(line, &target_only.target_path));
                }
            }
        }

        Ok(ret)
    }

    fn check_emphasis(&self, line: &Line, path: &Path) -> Vec<LintError> {
        let mut ret = Vec::new();

        let emphasis = Regex::new(r"\*\*?[^*\s][^*]*\*\*?").unwrap();

        for mat in emphasis.find_iter(&line.content) {
            let before = &line.content[..mat.start()];
            let after = &line.content[mat.end()..];

            // check emphasis mismatch like **...* or *...**
            if mat.as_str().starts_with("**") ^ mat.as_str().ends_with("**") {
                ret.push(LintError {
                    kind: LintErrorKind::EmphasisMismatch,
                    path: PathBuf::from(path),
                    line: line.clone(),
                    start: mat.start(),
                    end: mat.end(),
                });
            } else {
                let before_check = before.is_empty() | before.ends_with(" ");
                let after_check = after.is_empty() | after.starts_with(" ");
                let check = before_check & after_check;

                if !check {
                    ret.push(LintError {
                        kind: LintErrorKind::EmphasisWithoutSpace,
                        path: PathBuf::from(path),
                        line: line.clone(),
                        start: mat.start(),
                        end: mat.end(),
                    });
                }
            }
        }

        ret
    }

    fn check_half_paren(&self, line: &Line, path: &Path) -> Vec<LintError> {
        let mut ret = Vec::new();

        let paren = Regex::new(r"\(([^)]*)\)").unwrap();

        for cap in paren.captures_iter(&line.content) {
            if !cap.get(1).unwrap().as_str().is_ascii() {
                ret.push(LintError {
                    kind: LintErrorKind::HalfParenWithNonAscii,
                    path: PathBuf::from(path),
                    line: line.clone(),
                    start: cap.get(0).unwrap().start(),
                    end: cap.get(0).unwrap().end(),
                });
            }
        }

        ret
    }

    fn check_full_paren(&self, line: &Line, path: &Path) -> Vec<LintError> {
        let mut ret = Vec::new();

        let paren = Regex::new(r"（([^）]*)）").unwrap();

        for cap in paren.captures_iter(&line.content) {
            if cap.get(1).unwrap().as_str().is_ascii() {
                ret.push(LintError {
                    kind: LintErrorKind::FullParenWithoutNonAscii,
                    path: PathBuf::from(path),
                    line: line.clone(),
                    start: cap.get(0).unwrap().start(),
                    end: cap.get(0).unwrap().end(),
                });
            }
        }

        ret
    }
}

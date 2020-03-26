use anyhow::{Context, Error};
use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct Line {
    pub number: usize,
    pub content: String,
    pub last_both: usize,
    pub comment: bool,
}

#[derive(Clone, Debug)]
pub struct ModifiedLine {
    pub source: Line,
    pub target: Line,
}

#[derive(Clone, Debug)]
pub struct MissingLine {
    pub source: Line,
}

#[derive(Clone, Debug)]
pub struct GarbageLine {
    pub target: Line,
}

#[derive(Clone, Debug)]
pub enum MismatchLine {
    Modified(ModifiedLine),
    Missing(MissingLine),
    Garbage(GarbageLine),
}

#[derive(Clone, Debug)]
pub struct MismatchLines {
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub lines: Vec<MismatchLine>,
}

#[derive(Clone, Debug)]
pub struct MissingFile {
    pub source_path: PathBuf,
    pub target_path: PathBuf,
}

#[derive(Clone, Debug)]
pub enum Mismatch {
    MissingFile(MissingFile),
    MismatchLines(MismatchLines),
}

#[derive(Clone, Debug)]
pub struct TargetOnly {
    pub target_path: PathBuf,
    pub lines: Vec<Line>,
}

#[derive(Clone, Debug)]
pub struct Matcher {
    pub enable_code_comment_tweak: bool,
    pub code_comment_header: String,
    pub similar_threshold: f64,
}

impl Matcher {
    pub fn check_dir<T: AsRef<Path>>(
        &self,
        source: T,
        target: T,
    ) -> Result<(Vec<Mismatch>, Vec<TargetOnly>), Error> {
        let mut mismatches = Vec::new();
        let mut target_onlys = Vec::new();
        let source = source.as_ref();
        for entry in WalkDir::new(&source) {
            let source_path = entry
                .with_context(|| format!("Failed to enumerate '{}'", source.to_string_lossy()))?
                .into_path();
            let mut target_path = PathBuf::new();
            target_path.push(&target);
            target_path.push(source_path.strip_prefix(&source)?);
            if source_path.is_file() {
                let source_path = PathBuf::from(&source_path);
                let target_path = PathBuf::from(&target_path);
                if !target_path.exists() {
                    let mismatch = Mismatch::MissingFile(MissingFile {
                        source_path,
                        target_path,
                    });
                    mismatches.push(mismatch);
                } else {
                    let (mismatch_lines, target_only) =
                        self.check_file(&source_path, &target_path)?;
                    mismatches.push(Mismatch::MismatchLines(mismatch_lines));
                    target_onlys.push(TargetOnly {
                        target_path,
                        lines: target_only,
                    });
                }
            }
        }
        Ok((mismatches, target_onlys))
    }

    pub fn check_file<T: AsRef<Path>>(
        &self,
        source: T,
        target: T,
    ) -> Result<(MismatchLines, Vec<Line>), Error> {
        let source_path = source.as_ref();
        let target_path = target.as_ref();

        let mut source_reader = BufReader::new(
            File::open(source_path)
                .with_context(|| format!("Failed to open '{}'", source_path.to_string_lossy()))?,
        );
        let mut target_reader = BufReader::new(
            File::open(target_path)
                .with_context(|| format!("Failed to open '{}'", target_path.to_string_lossy()))?,
        );
        let mut source = String::new();
        let mut target = String::new();
        source_reader
            .read_to_string(&mut source)
            .with_context(|| format!("Failed to read '{}'", source_path.to_string_lossy()))?;
        target_reader
            .read_to_string(&mut target)
            .with_context(|| format!("Failed to read '{}'", target_path.to_string_lossy()))?;

        let target = self.revert_code_comment(&target);

        let (mismatch_lines, right_only_lines) = Matcher::get_mismatch_lines(&source, &target);

        let mut lines = Vec::new();
        for (lefts, rights) in mismatch_lines {
            let mut rights = rights.as_slice();
            for left in &lefts {
                let (similar_line, r, garbage) = self.get_similar_line(left, rights);
                for g in garbage {
                    if g.comment {
                        lines.push(MismatchLine::Garbage(GarbageLine { target: g.clone() }));
                    }
                }
                rights = r;
                if let Some(similar_line) = similar_line {
                    lines.push(MismatchLine::Modified(ModifiedLine {
                        source: left.clone(),
                        target: similar_line,
                    }));
                } else {
                    lines.push(MismatchLine::Missing(MissingLine {
                        source: left.clone(),
                    }));
                }
            }
            for g in rights {
                if g.comment {
                    lines.push(MismatchLine::Garbage(GarbageLine { target: g.clone() }));
                }
            }
        }

        let mismatch_lines = MismatchLines {
            source_path: PathBuf::from(source_path),
            target_path: PathBuf::from(target_path),
            lines,
        };

        Ok((mismatch_lines, right_only_lines))
    }

    fn revert_code_comment<'a>(&self, target: &'a str) -> Cow<'a, str> {
        if self.enable_code_comment_tweak {
            let mut ret = String::new();
            let mut code_block = false;
            for line in target.lines() {
                if line.trim().starts_with("```rust") {
                    code_block = true;
                } else if line.trim().starts_with("```") {
                    code_block = false;
                }

                let line = if code_block & line.starts_with(&self.code_comment_header) {
                    &line[2..]
                } else {
                    line
                };
                ret.push_str(&format!("{}\n", line));
            }

            ret.into()
        } else {
            target.into()
        }
    }

    fn get_mismatch_lines(source: &str, target: &str) -> (Vec<(Vec<Line>, Vec<Line>)>, Vec<Line>) {
        let mut source_line = 0;
        let mut target_line = 0;
        let mut last_both_source_line = 0;
        let mut last_both_target_line = 0;
        let mut target_comment = false;
        let mut left_lines = Vec::new();
        let mut right_lines = Vec::new();
        let mut mismatch_lines = Vec::new();
        let mut right_only_lines = Vec::new();
        for d in diff::lines(&source, &target) {
            match d {
                diff::Result::Both(_, _) => {
                    source_line += 1;
                    target_line += 1;
                    last_both_source_line = source_line;
                    last_both_target_line = target_line;
                    if !left_lines.is_empty() {
                        mismatch_lines.push((left_lines.clone(), right_lines.clone()));
                    } else if right_lines.iter().any(|x: &Line| x.comment) {
                        let right_lines: Vec<_> = right_lines
                            .iter()
                            .filter(|x| x.comment)
                            .map(|x| x.clone())
                            .collect();
                        mismatch_lines.push((left_lines.clone(), right_lines));
                    }
                    left_lines.clear();
                    right_lines.clear();
                }
                diff::Result::Left(x) => {
                    source_line += 1;
                    let line = Line {
                        number: source_line,
                        content: String::from(x),
                        last_both: last_both_target_line,
                        comment: false,
                    };
                    left_lines.push(line);
                }
                diff::Result::Right(x) => {
                    if x.trim().starts_with("-->") {
                        target_comment = false;
                    }

                    target_line += 1;
                    let line = Line {
                        number: target_line,
                        content: String::from(x),
                        last_both: last_both_source_line,
                        comment: target_comment,
                    };
                    right_only_lines.push(line.clone());
                    right_lines.push(line);

                    if x.trim().starts_with("<!--") {
                        target_comment = true;
                    }
                }
            }
        }

        if !left_lines.is_empty() {
            mismatch_lines.push((left_lines.clone(), right_lines.clone()));
        } else if right_lines.iter().any(|x: &Line| x.comment) {
            let right_lines: Vec<_> = right_lines
                .iter()
                .filter(|x| x.comment)
                .map(|x| x.clone())
                .collect();
            mismatch_lines.push((left_lines.clone(), right_lines));
        }

        (mismatch_lines, right_only_lines)
    }

    fn get_similar_line<'a, 'b>(
        &self,
        source: &'a Line,
        target: &'b [Line],
    ) -> (Option<Line>, &'b [Line], &'b [Line]) {
        let mut max_similarity = 0.0;
        let mut similar_line = None;
        let mut index = None;
        for (i, t) in target.iter().enumerate() {
            let similarity = diff::chars(&source.content, &t.content)
                .iter()
                .filter(|x| {
                    if let diff::Result::Both(_, _) = x {
                        true
                    } else {
                        false
                    }
                })
                .count();
            let similarity = similarity as f64 / source.content.len() as f64;
            if similarity > max_similarity && similarity > self.similar_threshold {
                max_similarity = similarity;
                similar_line = Some(t.clone());
                index = Some(i);
            }
        }

        if let Some(index) = index {
            (similar_line, &target[index + 1..], &target[0..index])
        } else {
            (similar_line, target, &[])
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_mismatch_lines_match() {
        let source = r##"
        aaa
        bbb
        ccc
            "##;
        let target = r##"
        aaa
        ddd
        bbb
        eee
        ccc
            "##;

        let ret = Matcher::get_mismatch_lines(source, target);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn test_get_mismatch_lines_diff() {
        let source = r##"
        aaa
        bbb
        ccc
            "##;
        let target = r##"
        aaa
        ddd
        bbc
        eee
        ccc
            "##;

        let ret = Matcher::get_mismatch_lines(source, target);
        assert_eq!(ret.len(), 1);
        assert_eq!(ret[0].0.len(), 1);
        assert_eq!(ret[0].0[0].number, 3);
        assert_eq!(ret[0].0[0].content, "        bbb");
        assert_eq!(ret[0].1.len(), 3);
        assert_eq!(ret[0].1[0].number, 3);
        assert_eq!(ret[0].1[0].content, "        ddd");
        assert_eq!(ret[0].1[1].number, 4);
        assert_eq!(ret[0].1[1].content, "        bbc");
        assert_eq!(ret[0].1[2].number, 5);
        assert_eq!(ret[0].1[2].content, "        eee");
    }

    #[test]
    fn test_check_dir() {
        let matcher = Matcher {
            enable_code_comment_tweak: true,
            code_comment_header: String::from("# "),
            similar_threshold: 0.5,
        };
        let mut ret = matcher
            .check_dir(
                format!("{}/testcase/original", std::env!("CARGO_MANIFEST_DIR")),
                format!("{}/testcase/translated", std::env!("CARGO_MANIFEST_DIR")),
            )
            .unwrap();
        ret.sort_by_key(|x| match x {
            Mismatch::MismatchLines(x) => x.source_path.clone(),
            Mismatch::MissingFile(x) => x.source_path.clone(),
        });
        assert_eq!(ret.len(), 3);
        assert!(
            matches!(&ret[0], Mismatch::MismatchLines(x) if x.source_path.file_name().unwrap() == "hello.md" && x.lines.is_empty())
        );
        assert!(
            matches!(&ret[1], Mismatch::MismatchLines(x) if x.source_path.file_name().unwrap() == "mismatch_lines.md" && !x.lines.is_empty())
        );
        assert!(
            matches!(&ret[2], Mismatch::MissingFile(x) if x.source_path.file_name().unwrap() == "missing_file.md")
        );
    }
}

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
}

#[derive(Clone, Debug)]
pub struct MissingLine {
    pub source: Line,
    pub target: Option<Line>,
}

#[derive(Clone, Debug)]
pub struct MissingLines {
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub lines: Vec<MissingLine>,
}

#[derive(Clone, Debug)]
pub struct MissingFile {
    pub source_path: PathBuf,
    pub target_path: PathBuf,
}

#[derive(Clone, Debug)]
pub enum Missing {
    File(MissingFile),
    Lines(MissingLines),
}

#[derive(Clone, Debug)]
pub struct Differ {
    pub code_comment: bool,
}

impl Differ {
    pub fn check_dir<T: AsRef<Path>>(&self, source: T, target: T) -> Result<Vec<Missing>, Error> {
        let mut ret = Vec::new();
        let source = source.as_ref();
        for entry in WalkDir::new(&source) {
            let source_path = entry
                .context(format!(
                    "Failed to enumerate '{}'",
                    source.to_string_lossy()
                ))?
                .into_path();
            let mut target_path = PathBuf::new();
            target_path.push(&target);
            target_path.push(source_path.strip_prefix(&source)?);
            if source_path.is_file() {
                if !target_path.exists() {
                    let source_path = PathBuf::from(&source_path);
                    let target_path = PathBuf::from(&target_path);
                    let missing = Missing::File(MissingFile {
                        source_path,
                        target_path,
                    });
                    ret.push(missing);
                } else {
                    let missing_lines = self.check_file(&source_path, &target_path)?;
                    ret.push(Missing::Lines(missing_lines));
                }
            }
        }
        Ok(ret)
    }

    pub fn check_file<T: AsRef<Path>>(&self, source: T, target: T) -> Result<MissingLines, Error> {
        let source_path = source.as_ref();
        let target_path = target.as_ref();

        let mut source_reader = BufReader::new(File::open(source_path).context(format!(
            "Failed to open '{}'",
            source_path.to_string_lossy()
        ))?);
        let mut target_reader = BufReader::new(File::open(target_path).context(format!(
            "Failed to open '{}'",
            target_path.to_string_lossy()
        ))?);
        let mut source = String::new();
        let mut target = String::new();
        source_reader.read_to_string(&mut source).context(format!(
            "Failed to read '{}'",
            source_path.to_string_lossy()
        ))?;
        target_reader.read_to_string(&mut target).context(format!(
            "Failed to read '{}'",
            target_path.to_string_lossy()
        ))?;

        let target = self.revert_code_comment(&target);

        let missing_lines = Differ::get_missing_lines(&source, &target);

        let mut lines = Vec::new();
        for (lefts, rights) in missing_lines {
            let mut rights = rights.as_slice();
            for left in &lefts {
                let (similar_line, r) = Differ::get_similar_line(left, rights);
                rights = r;
                if let Some(similar_line) = similar_line {
                    lines.push(MissingLine {
                        source: left.clone(),
                        target: Some(similar_line),
                    });
                } else {
                    lines.push(MissingLine {
                        source: left.clone(),
                        target: None,
                    });
                }
            }
        }

        Ok(MissingLines {
            source_path: PathBuf::from(source_path),
            target_path: PathBuf::from(target_path),
            lines,
        })
    }

    fn revert_code_comment<'a>(&self, target: &'a str) -> Cow<'a, str> {
        if self.code_comment {
            let mut ret = String::new();
            let mut code_block = false;
            for line in target.lines() {
                if line.trim().starts_with("```rust") {
                    code_block = true;
                } else if line.trim().starts_with("```") {
                    code_block = false;
                }

                let line = if code_block & line.starts_with("# ") {
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

    fn get_missing_lines(source: &str, target: &str) -> Vec<(Vec<Line>, Vec<Line>)> {
        let mut source_line = 0;
        let mut target_line = 0;
        let mut left_lines = Vec::new();
        let mut right_lines = Vec::new();
        let mut missing_lines = Vec::new();
        for d in diff::lines(&source, &target) {
            match d {
                diff::Result::Both(_, _) => {
                    source_line += 1;
                    target_line += 1;
                    if !left_lines.is_empty() {
                        missing_lines.push((left_lines.clone(), right_lines.clone()));
                    }
                    left_lines.clear();
                    right_lines.clear();
                }
                diff::Result::Left(x) => {
                    source_line += 1;
                    let line = Line {
                        number: source_line,
                        content: String::from(x),
                    };
                    left_lines.push(line);
                }
                diff::Result::Right(x) => {
                    target_line += 1;
                    let line = Line {
                        number: target_line,
                        content: String::from(x),
                    };
                    right_lines.push(line);
                }
            }
        }
        if !left_lines.is_empty() {
            missing_lines.push((left_lines.clone(), right_lines.clone()));
        }
        missing_lines
    }

    fn get_similar_line<'a, 'b>(
        source: &'a Line,
        target: &'b [Line],
    ) -> (Option<Line>, &'b [Line]) {
        let mut max_similarity = 0;
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
            if similarity > max_similarity {
                max_similarity = similarity;
                similar_line = Some(t.clone());
                index = Some(i);
            }
        }

        if let Some(index) = index {
            (similar_line, &target[index + 1..])
        } else {
            (similar_line, target)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_missing_lines_match() {
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

        let ret = Differ::get_missing_lines(source, target);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn test_get_missing_lines_diff() {
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

        let ret = Differ::get_missing_lines(source, target);
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
        let differ = Differ { code_comment: true };
        let mut ret = differ
            .check_dir(
                format!("{}/testcase/original", std::env!("CARGO_MANIFEST_DIR")),
                format!("{}/testcase/translated", std::env!("CARGO_MANIFEST_DIR")),
            )
            .unwrap();
        ret.sort_by_key(|x| match x {
            Missing::Lines(x) => x.source_path.clone(),
            Missing::File(x) => x.source_path.clone(),
        });
        assert_eq!(ret.len(), 3);
        assert!(
            matches!(&ret[0], Missing::Lines(x) if x.source_path.file_name().unwrap() == "hello.md" && x.lines.is_empty())
        );
        assert!(
            matches!(&ret[1], Missing::File(x) if x.source_path.file_name().unwrap() == "missing_file.md")
        );
        assert!(
            matches!(&ret[2], Missing::Lines(x) if x.source_path.file_name().unwrap() == "missing_lines.md" && !x.lines.is_empty())
        );
    }
}

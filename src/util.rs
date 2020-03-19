use crate::matcher::{Line, MismatchLine, MismatchLines};

#[derive(Debug)]
pub enum CombinedLine<'a> {
    Modified((&'a Line, &'a Line)),
    Missing(Vec<&'a Line>),
    Garbage(Vec<&'a Line>),
}

pub fn combine_line<'a>(mismatch: &'a MismatchLines) -> Vec<CombinedLine<'a>> {
    let mut lines = Vec::new();
    let mut source_neighbor = Vec::new();
    let mut target_neighbor = Vec::new();
    let mut source_prev_line = None;
    let mut target_prev_line = None;
    for line in &mismatch.lines {
        match line {
            MismatchLine::Modified(line) => {
                lines.push(CombinedLine::Modified((&line.source, &line.target)));
            }
            MismatchLine::Missing(line) => {
                match source_prev_line {
                    Some(x) => {
                        if x + 1 == line.source.number {
                            source_neighbor.push(&line.source);
                        } else {
                            if !source_neighbor.is_empty() {
                                lines.push(CombinedLine::Missing(source_neighbor.clone()));
                            }
                            source_neighbor.clear();
                            source_neighbor.push(&line.source);
                        }
                    }
                    None => {
                        source_neighbor.push(&line.source);
                    }
                }
                source_prev_line = Some(line.source.number);
            }
            MismatchLine::Garbage(line) => {
                match target_prev_line {
                    Some(x) => {
                        if x + 1 == line.target.number {
                            target_neighbor.push(&line.target);
                        } else {
                            if !target_neighbor.is_empty() {
                                lines.push(CombinedLine::Garbage(target_neighbor.clone()));
                            }
                            target_neighbor.clear();
                            target_neighbor.push(&line.target);
                        }
                    }
                    None => {
                        target_neighbor.push(&line.target);
                    }
                }
                target_prev_line = Some(line.target.number);
            }
        }
    }
    if !source_neighbor.is_empty() {
        lines.push(CombinedLine::Missing(source_neighbor.clone()));
    }
    if !target_neighbor.is_empty() {
        lines.push(CombinedLine::Garbage(target_neighbor.clone()));
    }

    lines
}

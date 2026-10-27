//! Small bounded document printer, translating Prettier's group/indent/line model.
#[derive(Clone, Debug)]
pub(crate) enum Doc {
    Text(String),
    Concat(Vec<Doc>),
    Line(u8), // 0 soft, 1 space, 2 hard
    Indent(Box<Doc>),
    Group(Box<Doc>),
    IfBreak(Box<Doc>, Box<Doc>),
    Suffix(Box<Doc>),
}
pub(crate) fn text(s: impl Into<String>) -> Doc {
    Doc::Text(s.into())
}
pub(crate) fn cat(v: Vec<Doc>) -> Doc {
    Doc::Concat(v)
}
pub(crate) fn join(sep: Doc, items: Vec<Doc>) -> Doc {
    let mut out = Vec::new();
    for (i, item) in items.into_iter().enumerate() {
        if i > 0 {
            out.push(sep.clone());
        }
        out.push(item);
    }
    cat(out)
}
pub(crate) fn hard() -> Doc {
    Doc::Line(2)
}
pub(crate) fn line() -> Doc {
    Doc::Line(1)
}
pub(crate) fn soft() -> Doc {
    Doc::Line(0)
}
pub(crate) fn indent(d: Doc) -> Doc {
    Doc::Indent(Box::new(d))
}
pub(crate) fn group(d: Doc) -> Doc {
    Doc::Group(Box::new(d))
}
pub(crate) fn if_break(b: Doc, f: Doc) -> Doc {
    Doc::IfBreak(Box::new(b), Box::new(f))
}
fn forced(d: &Doc) -> bool {
    match d {
        Doc::Line(2) | Doc::Suffix(_) => true,
        Doc::Concat(v) => v.iter().any(forced),
        Doc::Indent(d) | Doc::Group(d) => forced(d),
        _ => false,
    }
}
fn fits(mut left: isize, mut stack: Vec<(usize, bool, &Doc)>, budget: &mut usize) -> bool {
    while let Some((n, flat, d)) = stack.pop() {
        if *budget == 0 {
            return false;
        }
        *budget -= 1;
        if left < 0 {
            return false;
        }
        match d {
            Doc::Text(s) => left -= unicode_width::UnicodeWidthStr::width(s.as_str()) as isize,
            Doc::Concat(v) => {
                for x in v.iter().rev() {
                    stack.push((n, flat, x));
                }
            }
            Doc::Indent(x) => stack.push((n + 1, flat, x)),
            Doc::Group(x) => stack.push((n, flat, x)),
            Doc::Line(k) => {
                if !flat || *k == 2 {
                    return true;
                }
                if *k == 1 {
                    left -= 1;
                }
            }
            Doc::IfBreak(b, f) => stack.push((n, flat, if flat { f } else { b })),
            Doc::Suffix(_) => {}
        }
    }
    left >= 0
}
pub(crate) fn print(d: &Doc, tabs: bool, width: usize, max: usize) -> Result<String, &'static str> {
    let mut stack = vec![(0, false, d)];
    let mut out = String::new();
    let mut col = 0;
    let mut pending_indent = 0;
    let mut suffixes = Vec::new();
    let mut work = 4_000_000usize;
    while let Some((level, flat, d)) = stack.pop() {
        if work == 0 {
            return Err("Formatting work limit exceeded");
        }
        work -= 1;
        match d {
            Doc::Text(s) => {
                if !s.is_empty() {
                    if pending_indent > 0 {
                        out.push_str(&if tabs {
                            "\t".repeat(pending_indent)
                        } else {
                            " ".repeat(pending_indent * width)
                        });
                        col = pending_indent * width;
                        pending_indent = 0;
                    }
                    out.push_str(s);
                    col += unicode_width::UnicodeWidthStr::width(s.as_str());
                }
            }
            Doc::Concat(v) => {
                for x in v.iter().rev() {
                    stack.push((level, flat, x));
                }
            }
            Doc::Indent(x) => stack.push((level + 1, flat, x)),
            Doc::Group(x) => {
                let flat = !forced(x)
                    && (flat || {
                        let mut probe = stack.clone();
                        probe.push((level, true, x));
                        fits(80isize - col as isize, probe, &mut work)
                    });
                stack.push((level, flat, x));
            }
            Doc::IfBreak(b, f) => stack.push((level, flat, if flat { f } else { b })),
            Doc::Suffix(x) => suffixes.push((level, flat, x.as_ref())),
            Doc::Line(k) => {
                if *k == 2 || !flat {
                    if !suffixes.is_empty() {
                        stack.push((level, flat, d));
                        for x in suffixes.drain(..).rev() {
                            stack.push(x);
                        }
                        continue;
                    }
                    while out.ends_with([' ', '\t']) {
                        out.pop();
                    }
                    out.push('\n');
                    col = level * width;
                    pending_indent = level;
                } else if *k == 1 {
                    out.push(' ');
                    col += 1;
                }
            }
        }
        if out.len() > max {
            return Err("Formatted output limit exceeded");
        }
    }
    Ok(out)
}

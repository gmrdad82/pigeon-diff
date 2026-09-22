#[derive(Debug, Default, PartialEq, Eq)]
pub struct Diff {
    pub sections: Vec<Section>,
    pub cut: Option<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Section {
    pub title: String,
    pub stat: Vec<String>,
    pub files: Vec<File>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct File {
    pub path: String,
    pub old: Option<String>,
    pub new: Option<String>,
    pub notes: Vec<String>,
    pub binary: bool,
    pub hunks: Vec<Hunk>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Hunk {
    pub header: String,
    pub lines: Vec<Line>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Line {
    Context(String),
    Added(String),
    Removed(String),
    NoNewline,
}

impl Diff {
    pub fn totals(&self) -> (usize, usize, usize) {
        let mut files = 0;
        let mut added = 0;
        let mut removed = 0;
        for file in self.sections.iter().flat_map(|s| s.files.iter()) {
            files += 1;
            let (a, r) = file.counts();
            added += a;
            removed += r;
        }
        (files, added, removed)
    }
}

impl File {
    pub fn counts(&self) -> (usize, usize) {
        let mut added = 0;
        let mut removed = 0;
        for line in self.hunks.iter().flat_map(|h| h.lines.iter()) {
            match line {
                Line::Added(_) => added += 1,
                Line::Removed(_) => removed += 1,
                _ => {}
            }
        }
        (added, removed)
    }
}

pub fn parse(text: &str) -> Diff {
    let mut diff = Diff::default();
    let mut in_hunk = false;
    for raw in text.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let line = line.strip_suffix('\r').unwrap_or(line);
        if in_hunk {
            let hunk = current_hunk(&mut diff);
            match line.as_bytes().first() {
                Some(b' ') => hunk.lines.push(Line::Context(line[1..].to_string())),
                Some(b'+') => hunk.lines.push(Line::Added(line[1..].to_string())),
                Some(b'-') => hunk.lines.push(Line::Removed(line[1..].to_string())),
                Some(b'\\') => hunk.lines.push(Line::NoNewline),
                None => hunk.lines.push(Line::Context(String::new())),
                _ => in_hunk = false,
            }
            if in_hunk {
                continue;
            }
        }
        if let Some(title) = line.strip_prefix("# ") {
            diff.sections.push(Section {
                title: title.to_string(),
                ..Default::default()
            });
        } else if let Some(words) = line.strip_prefix("… ") {
            diff.cut = Some(words.to_string());
        } else if let Some(rest) = line.strip_prefix("diff --git ") {
            let (old, new) = split_git_header(rest);
            let path = new.clone().or(old.clone()).unwrap_or_default();
            current_section(&mut diff).files.push(File {
                path,
                old,
                new,
                ..Default::default()
            });
        } else if let Some(file) = current_file(&mut diff) {
            if let Some(header) = line.strip_prefix("@@") {
                file.hunks.push(Hunk {
                    header: format!("@@{header}"),
                    lines: Vec::new(),
                });
                in_hunk = true;
            } else if let Some(old) = line.strip_prefix("--- ") {
                file.old = strip_side(old, "a/");
            } else if let Some(new) = line.strip_prefix("+++ ") {
                file.new = strip_side(new, "b/");
                if let Some(new) = &file.new {
                    file.path = new.clone();
                } else if let Some(old) = &file.old {
                    file.path = old.clone();
                }
            } else if line.starts_with("Binary files ") || line == "GIT binary patch" {
                file.binary = true;
                file.notes.push(line.to_string());
            } else if !line.is_empty() {
                file.notes.push(line.to_string());
            }
        } else if !line.trim().is_empty() {
            current_section(&mut diff)
                .stat
                .push(line.trim().to_string());
        }
    }
    diff
}

fn split_git_header(rest: &str) -> (Option<String>, Option<String>) {
    let Some(at) = rest.find(" b/") else {
        return (None, None);
    };
    let old = rest[..at].strip_prefix("a/").map(str::to_string);
    let new = Some(rest[at + 3..].to_string());
    (old, new)
}

fn strip_side(side: &str, prefix: &str) -> Option<String> {
    let side = side.split('\t').next().unwrap_or(side);
    if side == "/dev/null" {
        return None;
    }
    Some(side.strip_prefix(prefix).unwrap_or(side).to_string())
}

fn current_section(diff: &mut Diff) -> &mut Section {
    if diff.sections.is_empty() {
        diff.sections.push(Section::default());
    }
    diff.sections.last_mut().expect("one section")
}

fn current_file(diff: &mut Diff) -> Option<&mut File> {
    diff.sections.last_mut()?.files.last_mut()
}

fn current_hunk(diff: &mut Diff) -> &mut Hunk {
    let file = current_file(diff).expect("a hunk sits in a file");
    if file.hunks.is_empty() {
        file.hunks.push(Hunk::default());
    }
    file.hunks.last_mut().expect("one hunk")
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_FILE: &str = "diff --git a/plan.md b/plan.md
index 422c2b7..0f7bc76 100644
--- a/plan.md
+++ b/plan.md
@@ -1,2 +1,2 @@
 A quiet corner
-by the window.
+by the window, with a lamp.
";

    #[test]
    fn a_diff_git_header_opens_a_file_with_both_sides() {
        let diff = parse("diff --git a/src/one.rs b/src/one.rs\n");
        let file = &diff.sections[0].files[0];
        assert_eq!(file.path, "src/one.rs");
        assert_eq!(file.old.as_deref(), Some("src/one.rs"));
        assert_eq!(file.new.as_deref(), Some("src/one.rs"));
        assert!(file.hunks.is_empty());
        assert_eq!(diff.totals(), (1, 0, 0));
    }

    #[test]
    fn the_minus_and_plus_lines_name_the_sides_and_dev_null_is_none() {
        let added =
            parse("diff --git a/n.md b/n.md\nnew file mode 100644\n--- /dev/null\n+++ b/n.md\n");
        let file = &added.sections[0].files[0];
        assert_eq!(file.old, None);
        assert_eq!(file.new.as_deref(), Some("n.md"));
        assert_eq!(file.path, "n.md");
        assert_eq!(file.notes, ["new file mode 100644"]);
        let gone = parse("diff --git a/g.md b/g.md\n--- a/g.md\t2026-01-01\n+++ /dev/null\n");
        let file = &gone.sections[0].files[0];
        assert_eq!(file.old.as_deref(), Some("g.md"));
        assert_eq!(file.new, None);
        assert_eq!(file.path, "g.md");
    }

    #[test]
    fn a_hunk_keeps_its_header_and_its_signed_lines() {
        let diff = parse(ONE_FILE);
        let file = &diff.sections[0].files[0];
        assert_eq!(file.hunks.len(), 1);
        let hunk = &file.hunks[0];
        assert_eq!(hunk.header, "@@ -1,2 +1,2 @@");
        assert_eq!(
            hunk.lines,
            [
                Line::Context("A quiet corner".into()),
                Line::Removed("by the window.".into()),
                Line::Added("by the window, with a lamp.".into()),
            ]
        );
        assert_eq!(file.counts(), (1, 1));
        assert_eq!(diff.totals(), (1, 1, 1));
    }

    #[test]
    fn two_hunks_and_a_second_file_land_in_order() {
        let text = format!(
            "{ONE_FILE}@@ -10,1 +10,2 @@ fn main\n ten\n+eleven\ndiff --git a/b.md b/b.md\n--- a/b.md\n+++ b/b.md\n@@ -1 +1 @@\n-x\n+y\n"
        );
        let diff = parse(&text);
        let files = &diff.sections[0].files;
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].hunks.len(), 2);
        assert_eq!(files[0].hunks[1].header, "@@ -10,1 +10,2 @@ fn main");
        assert_eq!(files[1].path, "b.md");
        assert_eq!(diff.totals(), (2, 3, 2));
    }

    #[test]
    fn no_newline_at_end_of_file_is_its_own_line() {
        let text = "diff --git a/e b/e\n--- a/e\n+++ b/e\n@@ -1 +1 @@\n-old\n\\ No newline at end of file\n+new\n\\ No newline at end of file\n";
        let diff = parse(text);
        let lines = &diff.sections[0].files[0].hunks[0].lines;
        assert_eq!(
            lines,
            &[
                Line::Removed("old".into()),
                Line::NoNewline,
                Line::Added("new".into()),
                Line::NoNewline,
            ]
        );
    }

    #[test]
    fn a_binary_notice_marks_the_file_without_a_hunk() {
        let text = "diff --git a/i.png b/i.png\nnew file mode 100644\nindex 0000000..1234567\nBinary files /dev/null and b/i.png differ\n";
        let diff = parse(text);
        let file = &diff.sections[0].files[0];
        assert!(file.binary);
        assert!(file.hunks.is_empty());
        assert_eq!(
            file.notes.last().unwrap(),
            "Binary files /dev/null and b/i.png differ"
        );
        let patch = parse("diff --git a/i.png b/i.png\nGIT binary patch\nliteral 12\nzcmZ\n");
        assert!(patch.sections[0].files[0].binary);
    }

    #[test]
    fn the_stat_block_sits_on_its_section_before_the_files() {
        let text = format!(
            "# Committed on task/lft3 since 80b816d\n plan.md | 2 +-\n 1 file changed, 1 insertion(+), 1 deletion(-)\n{ONE_FILE}# Not yet committed\n"
        );
        let diff = parse(&text);
        assert_eq!(diff.sections.len(), 2);
        let first = &diff.sections[0];
        assert_eq!(first.title, "Committed on task/lft3 since 80b816d");
        assert_eq!(
            first.stat,
            [
                "plan.md | 2 +-",
                "1 file changed, 1 insertion(+), 1 deletion(-)"
            ]
        );
        assert_eq!(first.files.len(), 1);
        assert_eq!(diff.sections[1].title, "Not yet committed");
        assert!(diff.sections[1].files.is_empty());
        assert_eq!(diff.cut, None);
    }

    #[test]
    fn the_cut_line_is_kept_and_a_hunk_cut_mid_way_still_draws() {
        let text = "diff --git a/big.txt b/big.txt\n--- /dev/null\n+++ b/big.txt\n@@ -0,0 +1,3 @@\n+one\n+tw\n… cut: the diff is over 2 MiB\n";
        let diff = parse(text);
        assert_eq!(diff.cut.as_deref(), Some("cut: the diff is over 2 MiB"));
        let lines = &diff.sections[0].files[0].hunks[0].lines;
        assert_eq!(
            lines,
            &[Line::Added("one".into()), Line::Added("tw".into())]
        );
    }

    #[test]
    fn an_empty_text_is_an_empty_diff() {
        let diff = parse("");
        assert!(diff.sections.is_empty());
        assert_eq!(diff.totals(), (0, 0, 0));
        let sections = parse("# Committed on task/x since abc\n# Not yet committed\n");
        assert_eq!(sections.sections.len(), 2);
        assert!(sections.sections.iter().all(|s| s.files.is_empty()));
    }
}

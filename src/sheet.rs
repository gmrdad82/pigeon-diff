use crate::diff::{Diff, Line as Hunk};

pub const MAX_LINES: usize = 4_000;
pub const MAX_TEXT: usize = 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Heading,
    Totals,
    Section,
    File,
    Meta,
    Context,
    Added,
    Removed,
    Cut,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub kind: Kind,
    pub text: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Sheet {
    pub lines: Vec<Line>,
    pub dropped: usize,
}

impl Sheet {
    pub fn text_bytes(&self) -> usize {
        self.lines.iter().map(|l| l.text.len()).sum()
    }
}

pub fn build(task: &str, run: &str, diff: &Diff) -> Sheet {
    let mut sheet = Builder::new();
    sheet.push(Kind::Heading, format!("{task} · {run}"));
    let (files, added, removed) = diff.totals();
    sheet.push(
        Kind::Totals,
        format!(
            "{files} {} · +{added} −{removed}",
            if files == 1 { "file" } else { "files" }
        ),
    );
    for section in &diff.sections {
        if !section.title.is_empty() {
            sheet.push(Kind::Section, section.title.clone());
        }
        if section.files.is_empty() && !section.title.is_empty() {
            sheet.push(Kind::Meta, "nothing".to_string());
        }
        for file in &section.files {
            let (a, r) = file.counts();
            let rename = match (&file.old, &file.new) {
                (Some(old), Some(new)) if old != new => format!("{old} → {new}"),
                _ => file.path.clone(),
            };
            sheet.push(Kind::File, format!("{rename}  +{a} −{r}"));
            for note in &file.notes {
                sheet.push(Kind::Meta, note.clone());
            }
            for hunk in &file.hunks {
                sheet.push(Kind::Meta, hunk.header.clone());
                for line in &hunk.lines {
                    let (kind, text) = match line {
                        Hunk::Context(t) => (Kind::Context, format!(" {t}")),
                        Hunk::Added(t) => (Kind::Added, format!("+{t}")),
                        Hunk::Removed(t) => (Kind::Removed, format!("-{t}")),
                        Hunk::NoNewline => (Kind::Meta, "\\ No newline at end of file".into()),
                    };
                    sheet.push(kind, text);
                }
            }
        }
    }
    if let Some(cut) = &diff.cut {
        sheet.push(Kind::Cut, format!("… {cut}"));
    }
    sheet.finish()
}

pub fn error(task: &str, run: &str, words: &str) -> Sheet {
    let mut sheet = Builder::new();
    sheet.push(Kind::Heading, format!("{task} · {run}"));
    sheet.push(Kind::Error, words.to_string());
    sheet.finish()
}

struct Builder {
    sheet: Sheet,
    text: usize,
}

impl Builder {
    fn new() -> Self {
        Self {
            sheet: Sheet::default(),
            text: 0,
        }
    }
    fn push(&mut self, kind: Kind, text: String) {
        if self.sheet.dropped > 0
            || self.sheet.lines.len() >= MAX_LINES
            || self.text + text.len() > MAX_TEXT
        {
            self.sheet.dropped += 1;
            return;
        }
        self.text += text.len();
        self.sheet.lines.push(Line { kind, text });
    }
    fn finish(mut self) -> Sheet {
        if self.sheet.dropped > 0 {
            let n = self.sheet.dropped;
            let words = format!(
                "… {n} more {} than the sheet holds",
                if n == 1 { "line" } else { "lines" }
            );
            self.sheet.lines.push(Line {
                kind: Kind::Cut,
                text: words,
            });
        }
        self.sheet
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::parse;

    const TWO_PARTS: &str = "# Committed on task/lft3 since 80b816d
 plan.md | 2 +-
 1 file changed, 1 insertion(+), 1 deletion(-)
diff --git a/plan.md b/plan.md
index 422c2b7..0f7bc76 100644
--- a/plan.md
+++ b/plan.md
@@ -1 +1 @@
-A quiet corner by the window.
+A quiet corner by the window, with a lamp.
# Not yet committed
diff --git a/notes.md b/notes.md
new file mode 100644
index 0000000..3e75765
--- /dev/null
+++ b/notes.md
@@ -0,0 +1 @@
+Measure the shelf before ordering.
";

    fn kinds(sheet: &Sheet) -> Vec<(Kind, &str)> {
        sheet
            .lines
            .iter()
            .map(|l| (l.kind, l.text.as_str()))
            .collect()
    }

    #[test]
    fn the_fixture_run_lays_out_as_the_card_describes() {
        let sheet = build("lft3", "lft3-review1", &parse(TWO_PARTS));
        assert_eq!(sheet.dropped, 0);
        assert_eq!(
            kinds(&sheet),
            [
                (Kind::Heading, "lft3 · lft3-review1"),
                (Kind::Totals, "2 files · +2 −1"),
                (Kind::Section, "Committed on task/lft3 since 80b816d"),
                (Kind::File, "plan.md  +1 −1"),
                (Kind::Meta, "index 422c2b7..0f7bc76 100644"),
                (Kind::Meta, "@@ -1 +1 @@"),
                (Kind::Removed, "-A quiet corner by the window."),
                (Kind::Added, "+A quiet corner by the window, with a lamp."),
                (Kind::Section, "Not yet committed"),
                (Kind::File, "notes.md  +1 −0"),
                (Kind::Meta, "new file mode 100644"),
                (Kind::Meta, "index 0000000..3e75765"),
                (Kind::Meta, "@@ -0,0 +1 @@"),
                (Kind::Added, "+Measure the shelf before ordering."),
            ]
        );
    }

    #[test]
    fn a_section_without_files_says_nothing_and_a_rename_shows_both_names() {
        let text = "# Committed on task/x since abc\ndiff --git a/old.md b/new.md\nsimilarity index 100%\nrename from old.md\nrename to new.md\n# Not yet committed\n";
        let sheet = build("x1", "x1-code1", &parse(text));
        let lines = kinds(&sheet);
        assert!(lines.contains(&(Kind::File, "old.md → new.md  +0 −0")));
        assert_eq!(lines.last(), Some(&(Kind::Meta, "nothing")));
        assert_eq!(lines[1], (Kind::Totals, "1 file · +0 −0"));
    }

    #[test]
    fn the_hosts_cut_line_closes_the_sheet() {
        let text = format!("{TWO_PARTS}… cut: the diff is over 2 MiB\n");
        let sheet = build("lft3", "lft3-review1", &parse(&text));
        assert_eq!(
            sheet.lines.last().map(|l| (l.kind, l.text.as_str())),
            Some((Kind::Cut, "… cut: the diff is over 2 MiB"))
        );
    }

    #[test]
    fn a_long_diff_stays_under_the_trees_bounds_and_counts_the_rest() {
        let mut text =
            String::from("diff --git a/big b/big\n--- /dev/null\n+++ b/big\n@@ -0,0 +1,9000 @@\n");
        for n in 0..9_000 {
            text.push_str(&format!("+line {n}\n"));
        }
        let sheet = build("lft3", "lft3-review1", &parse(&text));
        assert_eq!(sheet.lines.len(), MAX_LINES + 1);
        assert!(sheet.text_bytes() < 65_536);
        assert_eq!(sheet.dropped, 9_000 + 4 - MAX_LINES);
        assert_eq!(
            sheet.lines.last().unwrap().text,
            format!("… {} more lines than the sheet holds", sheet.dropped)
        );
        let mut wide =
            String::from("diff --git a/w b/w\n--- /dev/null\n+++ b/w\n@@ -0,0 +1,100 @@\n");
        for _ in 0..100 {
            wide.push_str(&format!("+{}\n", "x".repeat(1_000)));
        }
        let sheet = build("lft3", "lft3-review1", &parse(&wide));
        assert!(sheet.text_bytes() <= MAX_TEXT);
        assert!(sheet.dropped > 0);
        assert!(sheet.lines.len() < 100);
        let kept = sheet.lines.len() - 1;
        assert_eq!(kept + sheet.dropped, 100 + 4);
        assert!(
            sheet.lines[..kept]
                .iter()
                .skip(4)
                .all(|l| l.kind == Kind::Added && l.text.len() == 1_001)
        );
        let mut fill =
            String::from("diff --git a/f b/f\n--- /dev/null\n+++ b/f\n@@ -0,0 +1,3 @@\n");
        fill.push_str(&format!(
            "+{}\n+{}\n+last short line\n",
            "a".repeat(59_800),
            "m".repeat(200)
        ));
        let sheet = build("lft3", "lft3-review1", &parse(&fill));
        assert_eq!(sheet.dropped, 2);
        let tail = &kinds(&sheet)[4..];
        assert_eq!(tail.len(), 2);
        assert_eq!(tail[0].0, Kind::Added);
        assert_eq!(tail[0].1.len(), 59_801);
        assert_eq!(tail[1], (Kind::Cut, "… 2 more lines than the sheet holds"));
        let one = format!("{TWO_PARTS}+{}\n", "b".repeat(60_000));
        let sheet = build("lft3", "lft3-review1", &parse(&one));
        assert_eq!(sheet.dropped, 1);
        assert_eq!(
            sheet.lines.last().unwrap().text,
            "… 1 more line than the sheet holds"
        );
    }

    #[test]
    fn a_refusal_is_the_heading_and_one_error_line() {
        let sheet = error("lft1", "lft1-code1", "no run with a workspace");
        assert_eq!(
            kinds(&sheet),
            [
                (Kind::Heading, "lft1 · lft1-code1"),
                (Kind::Error, "no run with a workspace"),
            ]
        );
    }
}

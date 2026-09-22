wit_bindgen::generate!({ world: "diff", path: "wit", generate_all });

use crate::context::Context;
use crate::sheet::{self, Kind, Line, Sheet};
use pito::host::{log, ui, ui_types};
use pito::pigeon::door;

const SHOW: &str = "show-diff";

static LAST: std::sync::Mutex<Option<Context>> = std::sync::Mutex::new(None);

fn remember(context: Context) {
    *LAST.lock().unwrap_or_else(|p| p.into_inner()) = Some(context);
}

fn last() -> Option<(String, String)> {
    LAST.lock()
        .unwrap_or_else(|p| p.into_inner())
        .as_ref()
        .map(|c| (c.task.clone(), c.run.clone()))
}

struct Plugin;

impl Guest for Plugin {
    fn info() -> String {
        format!("gmrdad82/pigeon-diff@{}", env!("CARGO_PKG_VERSION"))
    }
    fn activate() {
        log::log(log::Level::Info, "diff activated");
    }
    fn deactivate() {}
    fn render(slot: ui::Slot, context: String) -> ui_types::Tree {
        let context = Context::parse(&context);
        let git = context.git;
        remember(context);
        match slot {
            ui::Slot::Command if git => column(vec![button(SHOW, "Show the diff")]),
            _ => column(Vec::new()),
        }
    }
    fn on_event(event: ui_types::Event) {
        let ui_types::Event::Click(id) = event else {
            return;
        };
        if id != SHOW {
            return;
        }
        let Some((task, run)) = last() else {
            return;
        };
        let sheet = match door::run_diff(&task) {
            Ok(text) => sheet::build(&task, &run, &crate::diff::parse(&text)),
            Err(words) => {
                log::log(log::Level::Warn, &format!("run-diff {task}: {words}"));
                sheet::error(&task, &run, &words)
            }
        };
        ui::render(ui::Slot::Sheet, &tree(&sheet));
    }
}

fn tree(sheet: &Sheet) -> ui_types::Tree {
    column(sheet.lines.iter().map(node).collect())
}

fn node(line: &Line) -> ui_types::Node {
    let (bold, code, tone) = match line.kind {
        Kind::Heading | Kind::Section => (true, false, None),
        Kind::Totals => (false, false, Some(ui_types::Tone::Dim)),
        Kind::File => (true, true, None),
        Kind::Meta => (false, true, Some(ui_types::Tone::Dim)),
        Kind::Context => (false, true, None),
        Kind::Added => (false, true, Some(ui_types::Tone::Ok)),
        Kind::Removed => (false, true, Some(ui_types::Tone::Danger)),
        Kind::Cut => (false, false, Some(ui_types::Tone::Dim)),
        Kind::Error => (false, false, Some(ui_types::Tone::Danger)),
    };
    ui_types::Node::Text(vec![ui_types::Span {
        text: line.text.clone(),
        bold,
        italic: matches!(line.kind, Kind::Cut),
        code,
        tone,
        link: None,
    }])
}

fn column(leaves: Vec<ui_types::Node>) -> ui_types::Tree {
    let mut nodes = vec![ui_types::Node::Column(ui_types::Container {
        children: (1..=leaves.len() as u32).collect(),
        gap: 0,
    })];
    nodes.extend(leaves);
    ui_types::Tree { root: 0, nodes }
}

fn button(id: &str, label: &str) -> ui_types::Node {
    ui_types::Node::Button(ui_types::Button {
        id: id.into(),
        label: label.into(),
        tone: Some(ui_types::Tone::Accent),
    })
}

export!(Plugin);

// The desk's context for the `command` slot, read into the three facts the plugin
// needs: the Task's key, its latest run's name and whether that run has a git branch.
// Plain Rust so the one rule the desk cannot ride yet — a branch means the verb,
// none means an empty command tree — tests on the host; `guest.rs` keeps the last
// parsed context for the click.

/// The Task the desk last asked about: its key, its run's name and whether the run
/// has a branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub task: String,
    /// The run's session name, or `no run` when the desk sent null or nothing.
    pub run: String,
    /// True when `branch` is a string: the run is a git flavour's and has a diff.
    pub git: bool,
}

impl Context {
    /// The desk's context JSON: `task`, `run` (null without one), `branch` (null for a
    /// folder run or none). Anything unreadable is an empty Task without a run.
    pub fn parse(context: &str) -> Self {
        let value: serde_json::Value = serde_json::from_str(context).unwrap_or_default();
        let field = |key: &str| value.get(key).and_then(|v| v.as_str()).map(str::to_string);
        Self {
            task: field("task").unwrap_or_default(),
            run: field("run").unwrap_or_else(|| "no run".to_string()),
            git: field("branch").is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The desk's seven fields as `render_plugin_slots` sends them for the fixture run.
    fn desk(run: &str, branch: &str) -> String {
        format!(
            r#"{{"task":"lft3","id":"lft3","title":"The lamp","status":"active","project":"Loft","run":{run},"branch":{branch}}}"#
        )
    }

    #[test]
    fn a_branch_means_the_verb_and_none_means_an_empty_command() {
        let git = Context::parse(&desk(r#""lft3-review1""#, r#""task/lft3""#));
        assert_eq!(
            git,
            Context {
                task: "lft3".into(),
                run: "lft3-review1".into(),
                git: true
            }
        );
        // A folder run: a run, no branch.
        let folder = Context::parse(&desk(r#""lft3-review1""#, "null"));
        assert!(!folder.git);
        assert_eq!(folder.run, "lft3-review1");
        // No run at all: both null.
        let none = Context::parse(&desk("null", "null"));
        assert!(!none.git);
        assert_eq!(none.run, "no run");
        assert_eq!(none.task, "lft3");
        // A desk that sends no `branch` field: the same as null.
        let absent = Context::parse(r#"{"task":"lft2","run":null}"#);
        assert!(!absent.git);
        assert_eq!(absent.run, "no run");
        // An empty or unreadable context is an empty Task without a run, never a panic.
        for text in ["", "not json", "[]", "42"] {
            let c = Context::parse(text);
            assert_eq!(
                c,
                Context {
                    task: String::new(),
                    run: "no run".into(),
                    git: false
                },
                "{text:?}"
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub task: String,
    pub run: String,
    pub git: bool,
}

impl Context {
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
        let folder = Context::parse(&desk(r#""lft3-review1""#, "null"));
        assert!(!folder.git);
        assert_eq!(folder.run, "lft3-review1");
        let none = Context::parse(&desk("null", "null"));
        assert!(!none.git);
        assert_eq!(none.run, "no run");
        assert_eq!(none.task, "lft3");
        let absent = Context::parse(r#"{"task":"lft2","run":null}"#);
        assert!(!absent.git);
        assert_eq!(absent.run, "no run");
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

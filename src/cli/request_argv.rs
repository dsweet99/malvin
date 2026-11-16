use malvin::config::model_list::ModelList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestKind {
    Router,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedRequest {
    pub text: String,
    pub kind: RequestKind,
    pub model: Option<ModelList>,
}

impl TaggedRequest {
    #[must_use]
    pub const fn is_router(&self) -> bool {
        matches!(self.kind, RequestKind::Router)
    }
}

pub(crate) fn tag_requests(requests: &[String]) -> Vec<TaggedRequest> {
    requests
        .iter()
        .map(|text| TaggedRequest {
            text: text.clone(),
            kind: RequestKind::Router,
            model: None,
        })
        .collect()
}

pub(crate) fn tag_requests_with_models(
    requests: &[String],
    models: Vec<ModelList>,
) -> Vec<TaggedRequest> {
    let mut tagged = tag_requests(requests);
    for (job, model) in tagged.iter_mut().zip(models) {
        job.model = Some(model);
    }
    tagged
}

#[cfg(test)]
mod tests {
    use crate::cli::Cli;
    use clap::Parser;

    fn models(argv: &[&str]) -> Vec<Option<String>> {
        let (cli, _) =
            crate::cli::config_defaults::parse_cli_with_config_defaults(argv.iter().copied())
                .expect("parse");
        cli.tagged_requests
            .iter()
            .map(|t| t.model.as_ref().map(|m| m.cli_value()))
            .collect()
    }

    #[test]
    fn repeated_model_applies_to_following_prompts() {
        let got = models(&[
            "malvin",
            "--model=cursor:opus",
            "--ml=inf",
            "a.md",
            "--model",
            "cursor:sonnet",
            "b.md",
            "c.md",
        ]);
        assert_eq!(got.len(), 3);
        assert_ne!(got[0], got[1]);
        assert_eq!(got[1], got[2]);
        assert!(got.iter().all(Option::is_some));
    }

    #[test]
    fn prompt_before_first_model_uses_first_model() {
        let got = models(&[
            "malvin",
            "a.md",
            "--model=cursor:opus",
            "b.md",
            "--model=cursor:sonnet",
        ]);
        assert_eq!(got[0], got[1]);
    }

    #[test]
    fn single_model_leaves_requests_untagged() {
        let got = models(&["malvin", "--model=cursor:opus", "a.md", "b.md"]);
        assert_eq!(got, vec![None, None]);
        assert!(Cli::try_parse_from(["malvin", "--model=cursor:opus", "a.md"]).is_ok());
    }
}

use super::{ModelList, split_model_list};
use crate::config::malvin_config_file::{parse_agent_config, parse_model_list_cli_arg};
use crate::test_support::test_utils::with_isolated_home;
use std::collections::BTreeMap;

#[test]
fn model_list_keeps_bracket_parameters_together() {
    assert_eq!(
        split_model_list("codex:gpt-5.6[thinking=high,service=priority], claude:sonnet").unwrap(),
        [
            "codex:gpt-5.6[thinking=high,service=priority]",
            "claude:sonnet"
        ]
    );
    for raw in [
        "",
        ",cursor:auto",
        "cursor:auto,",
        "cursor:auto,,claude:sonnet",
    ] {
        assert!(split_model_list(raw).is_err(), "{raw}");
    }
    assert!(ModelList::new(vec![]).is_err());
}

#[test]
fn config_models_preserve_order_and_expand_aliases() {
    let aliases = BTreeMap::from([("cheap".into(), "cursor:auto".into())]);
    let config =
        parse_agent_config("[agent]\nmodel = ['cheap', 'claude:sonnet']", &aliases).unwrap();
    assert_eq!(config.model.first().canonical(), "cursor:auto");
    assert_eq!(config.model.cli_value(), "cursor:auto,claude:sonnet");
    for value in [
        "[]",
        "['']",
        "['cursor:auto', 1]",
        "1",
        "['cursor:auto', 'bad']",
    ] {
        assert!(
            parse_agent_config(&format!("[agent]\nmodel = {value}"), &aliases).is_err(),
            "{value}"
        );
    }
}

#[test]
fn cli_models_parse_single_and_multiple_models() {
    with_isolated_home(|_| {
        let single = parse_model_list_cli_arg("cursor:auto").unwrap();
        assert_eq!(single.iter().count(), 1);
        let models =
            parse_model_list_cli_arg("codex:gpt-5.6[thinking=high,service=priority],claude:sonnet")
                .unwrap();
        assert_eq!(models.iter().count(), 2);
        assert_eq!(
            models.cli_value(),
            "codex:gpt-5.6[thinking=high,service=priority],claude:sonnet"
        );
        assert!(parse_model_list_cli_arg("cursor:auto,bad").is_err());
    });
}

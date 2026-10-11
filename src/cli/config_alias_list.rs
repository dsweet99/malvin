use std::collections::BTreeMap;

use malvin::output::{MALVIN_WHO, print_stdout_line};

pub(crate) const MODEL_ALIAS_SECTION: &str = "aliases.models";
pub(crate) const REMOTE_ALIAS_SECTION: &str = "aliases.remotes";

pub(crate) fn alias_section_lines(
    section: &str,
    aliases: &BTreeMap<String, String>,
) -> Vec<String> {
    if aliases.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![
        String::new(),
        format!("~/.malvinconf/config.toml [{section}]"),
    ];
    lines.extend(
        aliases
            .iter()
            .map(|(name, target)| format!("{name}\t{target}")),
    );
    lines
}

pub(crate) fn print_alias_section(section: &str, aliases: &BTreeMap<String, String>) {
    for line in alias_section_lines(section, aliases) {
        print_stdout_line(MALVIN_WHO, &line);
    }
}

pub(crate) fn print_home_model_aliases() {
    print_alias_section(
        MODEL_ALIAS_SECTION,
        &malvin::config::malvin_config_file::load_model_aliases(),
    );
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{MODEL_ALIAS_SECTION, alias_section_lines};

    #[test]
    fn alias_section_is_empty_without_entries_and_lists_names_in_order() {
        assert!(alias_section_lines(MODEL_ALIAS_SECTION, &BTreeMap::new()).is_empty());
        let lines = alias_section_lines(
            MODEL_ALIAS_SECTION,
            &BTreeMap::from([
                ("nova".to_string(), "pi:openai/gpt-5".to_string()),
                ("astra".to_string(), "cursor:gpt-5".to_string()),
            ]),
        );
        assert_eq!(
            lines,
            vec![
                String::new(),
                "~/.malvinconf/config.toml [aliases.models]".to_string(),
                "astra\tcursor:gpt-5".to_string(),
                "nova\tpi:openai/gpt-5".to_string(),
            ]
        );
    }
}

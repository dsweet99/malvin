/// Append family aliases that `resolve_codex_model_slug` accepts.
///
/// A family is the catalog id with its last `-segment` removed. It is listed
/// only when two or more catalog ids share that prefix and the family itself
/// is not already a catalog id. The alias points at the first matching row.
pub(super) fn with_family_aliases(models: Vec<(String, String)>) -> Vec<(String, String)> {
    let aliases = family_alias_rows(&models);
    let mut listed = models;
    listed.extend(aliases);
    listed
}

fn family_alias_rows(models: &[(String, String)]) -> Vec<(String, String)> {
    let mut aliases = Vec::new();
    for family in unique_families(models) {
        if catalog_has_id(models, &family) {
            continue;
        }
        if let Some(target) = first_prefix_match(models, &family) {
            aliases.push((family, format!("alias → {target}")));
        }
    }
    aliases
}

fn unique_families(models: &[(String, String)]) -> Vec<String> {
    let mut families = Vec::new();
    for (id, _) in models {
        if let Some(family) = family_of(id)
            && !families.iter().any(|seen| seen == family)
            && family_member_count(models, family) >= 2
        {
            families.push(family.to_owned());
        }
    }
    families
}

fn family_of(id: &str) -> Option<&str> {
    let (family, suffix) = id.rsplit_once('-')?;
    if family.is_empty() || !suffix_is_variant(suffix) {
        return None;
    }
    Some(family)
}

fn suffix_is_variant(suffix: &str) -> bool {
    !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_alphabetic())
}

fn family_member_count(models: &[(String, String)], family: &str) -> usize {
    let prefix = format!("{family}-");
    models
        .iter()
        .filter(|(id, _)| id.starts_with(&prefix))
        .count()
}

fn catalog_has_id(models: &[(String, String)], id: &str) -> bool {
    models.iter().any(|(existing, _)| existing == id)
}

fn first_prefix_match(models: &[(String, String)], family: &str) -> Option<String> {
    let prefix = format!("{family}-");
    models
        .iter()
        .find(|(id, _)| id.starts_with(&prefix))
        .map(|(id, _)| id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str) -> (String, String) {
        (id.into(), id.to_ascii_uppercase())
    }

    #[test]
    fn family_alias_adds_shared_prefix_once() {
        let listed = with_family_aliases(vec![
            row("gpt-5.6-sol"),
            row("gpt-5.6-terra"),
            row("gpt-5.5"),
            row("codex-auto-review"),
        ]);
        assert_eq!(listed[0].0, "gpt-5.6-sol");
        assert_eq!(listed.last().unwrap().0, "gpt-5.6");
        assert_eq!(listed.last().unwrap().1, "alias → gpt-5.6-sol");
        assert_eq!(listed.iter().filter(|(id, _)| id == "gpt-5.6").count(), 1);
        assert!(
            listed
                .iter()
                .any(|(id, label)| id == "gpt-5.5" && label == "GPT-5.5")
        );
        assert!(
            listed
                .iter()
                .all(|(id, _)| id != "gpt" && id != "codex-auto")
        );
    }

    #[test]
    fn family_alias_skips_existing_catalog_id() {
        let listed = with_family_aliases(vec![row("gpt-5.4"), row("gpt-5.4-mini")]);
        assert!(
            listed
                .iter()
                .all(|(id, label)| { id != "gpt-5.4" || !label.starts_with("alias") })
        );
        assert_eq!(listed.len(), 2);
    }

    #[test]
    fn family_helpers_cover_empty_and_missing() {
        assert!(family_of("").is_none());
        assert!(family_of("-tail").is_none());
        assert!(family_of("gpt-5.5").is_none());
        assert!(!suffix_is_variant("5.5"));
        assert!(suffix_is_variant("sol"));
        assert_eq!(family_of("gpt-5.6-sol"), Some("gpt-5.6"));
        assert!(first_prefix_match(&[], "gpt-5.6").is_none());
        assert!(!catalog_has_id(&[], "gpt-5.6"));
        assert_eq!(family_member_count(&[], "gpt-5.6"), 0);
        assert!(family_alias_rows(&[]).is_empty());
        assert!(unique_families(&[]).is_empty());
        assert!(with_family_aliases(Vec::new()).is_empty());
    }
}

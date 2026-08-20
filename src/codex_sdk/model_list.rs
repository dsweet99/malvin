use super::ModelListPage;

pub(super) fn parse_model_list_page(value: &serde_json::Value) -> Result<ModelListPage, String> {
    reject_model_list_error(value)?;
    let result = model_list_result(value)?;
    Ok(ModelListPage {
        models: parse_model_rows(result)?,
        next_cursor: parse_next_cursor(result),
    })
}

fn reject_model_list_error(value: &serde_json::Value) -> Result<(), String> {
    if let Some(error) = value.get("error") {
        return Err(format!("codex model/list: {error}"));
    }
    Ok(())
}

fn model_list_result(value: &serde_json::Value) -> Result<&serde_json::Value, String> {
    value
        .get("result")
        .ok_or_else(|| "codex model/list response missing result".into())
}

fn parse_model_rows(result: &serde_json::Value) -> Result<Vec<(String, String)>, String> {
    let rows = result
        .get("data")
        .and_then(serde_json::Value::as_array)
        .ok_or("codex model/list response missing data")?;
    Ok(rows.iter().filter_map(parse_model_row).collect())
}

fn parse_model_row(row: &serde_json::Value) -> Option<(String, String)> {
    Some((
        row.get("id")?.as_str()?.to_owned(),
        row.get("displayName")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    ))
}

fn parse_next_cursor(result: &serde_json::Value) -> Option<String> {
    result
        .get("nextCursor")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_list_page_type_fields_are_constructible() {
        let page = ModelListPage {
            models: vec![("gpt-test".into(), "Test".into())],
            next_cursor: Some("next".into()),
        };
        assert_eq!(page.models.len(), 1);
        assert_eq!(page.next_cursor.as_deref(), Some("next"));
    }
    #[test]
    fn model_list_page_reads_next_cursor() {
        let value = serde_json::json!({
            "result": {
                "data": [{"id": "gpt-test", "displayName": "Test"}],
                "nextCursor": "page-2"
            }
        });
        let page = parse_model_list_page(&value).expect("page");
        assert_eq!(page.models, vec![("gpt-test".into(), "Test".into())]);
        assert_eq!(page.next_cursor.as_deref(), Some("page-2"));
    }

    #[test]
    fn model_list_page_without_cursor_finishes() {
        let value = serde_json::json!({
            "result": {"data": [{"id": "gpt-test"}]}
        });
        let page = parse_model_list_page(&value).expect("page");
        assert!(page.next_cursor.is_none());
        assert_eq!(page.models[0].0, "gpt-test");
    }

    #[test]
    fn model_list_page_propagates_error() {
        let value = serde_json::json!({"error": {"message": "bad"}});
        let err = parse_model_list_page(&value).expect_err("error");
        assert!(err.contains("bad"));
    }

    #[test]
    fn model_list_page_rejects_missing_result_and_data() {
        let missing_result = serde_json::json!({});
        assert!(parse_model_list_page(&missing_result)
            .expect_err("missing result")
            .contains("missing result"));

        let missing_data = serde_json::json!({"result": {}});
        assert!(parse_model_list_page(&missing_data)
            .expect_err("missing data")
            .contains("missing data"));
    }

    #[test]
    fn model_list_page_skips_rows_without_string_id() {
        let value = serde_json::json!({
            "result": {
                "data": [
                    {"displayName": "Missing id"},
                    {"id": 42, "displayName": "Wrong type"},
                    {"id": "valid"}
                ]
            }
        });
        let page = parse_model_list_page(&value).expect("page");
        assert_eq!(page.models, vec![("valid".into(), String::new())]);
    }

    #[test]
    fn model_list_row_requires_id_and_defaults_display_name() {
        assert!(parse_model_row(&serde_json::json!({})).is_none());
        assert_eq!(
            parse_model_row(&serde_json::json!({"id": "valid"})),
            Some(("valid".into(), String::new()))
        );
    }
}

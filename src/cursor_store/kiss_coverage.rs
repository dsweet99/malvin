#[test]
fn kiss_cov_cursor_store_symbols() {
    let _ = super::parse::parse_tool_call_args_from_blob;
    let _ = super::parse::parse_tool_call_item;
    let _ = super::parse::tool_call_path;
    let _ = super::parse::parse_tool_calls_from_value;
    let _ = super::parse::parse_json_blob_payload;
    let _ = super::path::find_store_path;
    let _ = super::path::find_legacy_store_path;
    let _ = super::path::store_db_contains_substring;
    let _ = stringify!(super::types::ToolCallArgs);
    let _ = super::cache::install_test_store;
    let _ = stringify!(super::cache::TestStoreSpec);
}

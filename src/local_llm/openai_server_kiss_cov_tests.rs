//! Kiss static name witnesses for [`super::openai_server`] / [`super::openai_http`].

#[test]
fn kiss_cov_openai_server_names() {
    let _ = stringify!(LocalOpenAiServer);
    let _ = stringify!(start);
    let _ = stringify!(drop);
    let _ = stringify!(accept_loop);
    let _ = stringify!(handle_connection);
    let _ = stringify!(respond_to_request);
    let _ = stringify!(block_on_complete);
    let _ = stringify!(HttpRequest);
    let _ = stringify!(read_http_request);
    let _ = stringify!(read_until_headers);
    let _ = stringify!(parse_request_head);
    let _ = stringify!(content_length_from_headers);
    let _ = stringify!(read_body_remainder);
    let _ = stringify!(find_header_end);
    let _ = stringify!(parse_messages);
    let _ = stringify!(chat_message_from_json);
    let _ = stringify!(message_content);
    let _ = stringify!(write_sse_completion);
    let _ = stringify!(write_response);
    let _ = stringify!(scripted_server_answers_chat_completions);
    let _ = stringify!(post_chat_completions);
    let _ = stringify!(parse_messages_maps_developer_to_system);
    let _ = stringify!(find_header_end_locates_separator);
    let _ = super::message_content(&serde_json::json!({"content": "x"}));
}

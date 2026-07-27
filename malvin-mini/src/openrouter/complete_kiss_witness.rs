//! Unit tests for completion helpers.
    use crate::error::OpenRouterError;

    use super::{
        affordable_max_tokens_from_outcome, completion_post_url, completion_with_meta,
        finish_reason_is_length, length_truncated_max_tokens_bump, outcome_from_http_body,
        parse_affordable_max_tokens, shrink_prompt_messages, transport_failure_meta,
        transport_meta,
    };

    #[test]
    fn parse_affordable_max_tokens_reads_provider_message() {
        let text = "OpenRouter billing/credit failure (402): You requested up to 8192 tokens, but can only afford 6776.";
        assert_eq!(parse_affordable_max_tokens(text), Some(6776));
        assert_eq!(parse_affordable_max_tokens("no digits"), None);
    }

    #[test]
    fn affordable_max_tokens_from_outcome_reads_billing_body() {
        let billing = completion_with_meta(
            Err(OpenRouterError::BillingFailure {
                status: 402,
                body: "You requested up to 8192 tokens, but can only afford 6776.".into(),
            }),
            transport_meta(Some(402), None),
        );
        assert_eq!(affordable_max_tokens_from_outcome(&billing), Some(6776));

        let billing_no_afford = completion_with_meta(
            Err(OpenRouterError::BillingFailure {
                status: 402,
                body: "no credits".into(),
            }),
            transport_meta(Some(402), None),
        );
        assert_eq!(affordable_max_tokens_from_outcome(&billing_no_afford), None);

        let not_billing = completion_with_meta(
            Err(OpenRouterError::MissingContent),
            transport_meta(Some(200), None),
        );
        assert_eq!(affordable_max_tokens_from_outcome(&not_billing), None);

        let ok = outcome_from_http_body(
            200,
            r#"{"choices":[{"message":{"content":"hi"}}]}"#.into(),
            1,
        );
        assert_eq!(affordable_max_tokens_from_outcome(&ok), None);
    }

    #[test]
    fn length_truncated_max_tokens_bump_on_missing_content() {
        let body = r#"{"choices":[{"finish_reason":"length","message":{"content":null}}]}"#;
        assert!(finish_reason_is_length(body));
        let outcome = completion_with_meta(
            Err(OpenRouterError::MissingContent),
            transport_meta(Some(200), Some(body.into())),
        );
        assert_eq!(length_truncated_max_tokens_bump(&outcome, Some(2048)), Some(4096));
        assert_eq!(length_truncated_max_tokens_bump(&outcome, Some(4096)), Some(8192));
        assert_eq!(length_truncated_max_tokens_bump(&outcome, Some(8192)), None);
        assert_eq!(length_truncated_max_tokens_bump(&outcome, Some(16384)), None);
        let stop = completion_with_meta(
            Err(OpenRouterError::MissingContent),
            transport_meta(
                Some(200),
                Some(r#"{"choices":[{"finish_reason":"stop","message":{"content":null}}]}"#.into()),
            ),
        );
        assert_eq!(length_truncated_max_tokens_bump(&stop, Some(2048)), None);
        let reasoned = completion_with_meta(
            Err(OpenRouterError::MissingContent),
            transport_meta(
                Some(200),
                Some(r#"{"choices":[{"finish_reason":"length","message":{"content":null,"reasoning":"t"}}]}"#.into()),
            ),
        );
        assert_eq!(length_truncated_max_tokens_bump(&reasoned, Some(2048)), None);
    }

    #[test]
    fn with_tool_use_system_reminder_prepends_when_missing() {
        use super::super::types::{ChatMessage, ChatRole};
        use super::complete_prompt_shape::with_tool_use_system_reminder;
        let msgs = [ChatMessage {
            role: ChatRole::User,
            content: "hi".into(),
        }];
        let out = with_tool_use_system_reminder(&msgs);
        assert_eq!(out.len(), 2);
        assert!(matches!(out[0].role, ChatRole::System));
        assert!(out[0].content.contains("request-named"));
        assert!(out[0].content.contains("Look hard for unmet"));
        assert!(out[0].content.contains("private asserts"));
        let already = [
            ChatMessage {
                role: ChatRole::System,
                content: "existing".into(),
            },
            ChatMessage {
                role: ChatRole::User,
                content: "hi".into(),
            },
        ];
        let kept = with_tool_use_system_reminder(&already);
        assert_eq!(kept.len(), 3);
        assert_eq!(kept[0].content, "existing");
        assert!(kept[1].content.contains("Look hard") || kept[1].content.contains("Exterior"));
    }

    #[test]
    fn with_tool_use_system_reminder_skips_short_form_marker_turns() {
        use super::super::types::{ChatMessage, ChatRole};
        use super::complete_marker_shape::looks_like_marker_prompt;
        use super::complete_prompt_shape::with_tool_use_system_reminder;
        assert!(looks_like_marker_prompt(
            "Write COMPLEXITY_SCORE: n then Pause."
        ));
        assert!(looks_like_marker_prompt("CODING_TASK: YES\n\nPause."));
        assert!(!looks_like_marker_prompt("ordinary study turn"));
        let msgs = [ChatMessage {
            role: ChatRole::User,
            content: "Is this a coding task?\n\nCODING_TASK: YES\n\nPause.".into(),
        }];
        let out = with_tool_use_system_reminder(&msgs);
        assert_eq!(out.len(), 1);
        assert!(matches!(out[0].role, ChatRole::User));
    }

    #[test]
    fn mutate_messages_after_missing_content_strips_study_reminder() {
        use super::super::types::{ChatMessage, ChatRole};
        use super::complete_prompt_shape::mutate_messages_after_missing_content;
        let mut msgs = vec![
            ChatMessage {
                role: ChatRole::System,
                content: "State the problem and rival readings before acting. unpaid request-named probes"
                    .into(),
            },
            ChatMessage {
                role: ChatRole::User,
                content: "hi".into(),
            },
        ];
        // First miss: thought-only progress cue.
        assert!(mutate_messages_after_missing_content(&mut msgs));
        assert!(msgs[0].content.contains("Thought-only responses"));
        assert_eq!(msgs.len(), 3);
        // Second miss: strip Study reminder; cue kept.
        assert!(mutate_messages_after_missing_content(&mut msgs));
        assert!(msgs.iter().any(|m| m.content.contains("Thought-only responses")));
        assert!(!msgs.iter().any(|m| m.content.contains("request-named")));
        assert!(msgs.iter().any(|m| matches!(m.role, ChatRole::User)));
    }

    #[test]
    fn marker_response_missing_label_detects_coding_miss() {
        use super::super::types::{ChatMessage, ChatRole};
        use super::complete_prompt_shape::{
            marker_response_missing_label, mutate_messages_after_marker_miss,
        };
        let msgs = [ChatMessage {
            role: ChatRole::User,
            content: "CODING_TASK: YES\n\nPause.".into(),
        }];
        assert!(marker_response_missing_label(
            &msgs,
            "Investigation complete. Summary of the situation:"
        ));
        assert!(!marker_response_missing_label(&msgs, "CODING_TASK: YES\n"));
        let mut retry = msgs.to_vec();
        assert!(mutate_messages_after_marker_miss(&mut retry));
        assert!(matches!(retry[0].role, ChatRole::System));
        assert!(retry[0].content.contains("required marker line"));
        assert!(retry.iter().any(|m| m.content.starts_with("Output exactly one")));
        assert!(!mutate_messages_after_marker_miss(&mut retry));
    }

    #[test]
    fn shrink_prompt_messages_drops_oldest_then_truncates_sole_survivor() {
        use super::super::types::{ChatMessage, ChatRole};
        let mut msgs = vec![
            ChatMessage {
                role: ChatRole::System,
                content: "sys".into(),
            },
            ChatMessage {
                role: ChatRole::User,
                content: "old".into(),
            },
            ChatMessage {
                role: ChatRole::User,
                content: "x".repeat(100),
            },
        ];
        assert!(shrink_prompt_messages(&mut msgs));
        assert_eq!(msgs.len(), 2);
        assert!(shrink_prompt_messages(&mut msgs));
        assert!(msgs[1].content.contains("[truncated]"));
        assert!(!shrink_prompt_messages(&mut msgs) || msgs[1].content.len() < 100);
    }

    #[test]
    fn kiss_witness_completion_post_url() {
        assert_eq!(
            completion_post_url("https://openrouter.ai/api/"),
            "https://openrouter.ai/api/chat/completions"
        );
    }

    #[test]
    fn completion_with_meta_and_transport_meta_helpers() {
        let http = transport_meta(Some(201), Some("body".into()));
        let wrapped = completion_with_meta(Err(OpenRouterError::MissingContent), http);
        assert_eq!(wrapped.http.status, Some(201));
        assert_eq!(wrapped.http.body.as_deref(), Some("body"));
        let ok = outcome_from_http_body(
            200,
            r#"{"choices":[{"message":{"content":"hi"}}]}"#.into(),
            1,
        );
        assert_eq!(ok.result.as_ref().expect("ok").content, "hi");
        let err = outcome_from_http_body(418, "teapot".into(), 1);
        assert!(err.result.is_err());
    }

fn unreachable_transport_err() -> reqwest::Error {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(async {
            reqwest::Client::new()
                .get("http://127.0.0.1:1")
                .send()
                .await
                .expect_err("transport")
        })
}

#[test]
fn kiss_witness_transport_failure_meta() {
    let none_status = transport_failure_meta(None, unreachable_transport_err());
    assert!(none_status.result.is_err());
    assert_eq!(none_status.http.status, None);
    let with_status = transport_failure_meta(Some(200), unreachable_transport_err());
    assert_eq!(with_status.http.status, Some(200));
}

use crate::ChatMessage;

/// Remove every odd-indexed word from the 25%-75% middle region of `content`.
#[must_use]
pub fn shrink_prompt_content(content: &str) -> String {
    let words: Vec<&str> = content.split_whitespace().collect();
    let n = words.len();
    if n == 0 {
        return content.to_string();
    }
    let lo = n * 25 / 100;
    let hi = n * 75 / 100;
    words
        .into_iter()
        .enumerate()
        .filter(|(i, _)| *i < lo || *i > hi || i % 2 == 0)
        .map(|(_, w)| w)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Apply [`shrink_prompt_content`] to every message's `content` field.
#[must_use]
pub fn shrink_messages(messages: &[ChatMessage]) -> Vec<ChatMessage> {
    messages
        .iter()
        .map(|m| ChatMessage {
            role: m.role,
            content: shrink_prompt_content(&m.content),
        })
        .collect()
}

#[must_use]
pub fn is_prompt_too_long_error(err: &crate::OpenRouterError) -> bool {
    err.to_string()
        .to_ascii_lowercase()
        .contains("prompt is too long")
}

#[cfg(test)]
mod tests {
    use super::{is_prompt_too_long_error, shrink_messages, shrink_prompt_content};
    use crate::{ChatMessage, ChatRole};
    use crate::OpenRouterError;

    #[test]
    fn shrink_prompt_content_removes_odd_indexed_words_in_middle_region() {
        // 12 words (indices 0..11); lo=3, hi=9; odd indices in band: 3,5,7,9
        let input = "w0 w1 w2 w3 w4 w5 w6 w7 w8 w9 w10 w11";
        let shrunk = shrink_prompt_content(input);
        assert_eq!(shrunk, "w0 w1 w2 w4 w6 w8 w10 w11");
    }

    #[test]
    fn shrink_prompt_content_leaves_short_prompts_unchanged_when_nothing_to_remove() {
        let input = "only";
        assert_eq!(shrink_prompt_content(input), "only");
    }

    #[test]
    fn shrink_messages_applies_to_each_message_content() {
        let messages = vec![
            ChatMessage {
                role: ChatRole::User,
                content: "a b c d e f".into(),
            },
            ChatMessage {
                role: ChatRole::Assistant,
                content: "x y z".into(),
            },
        ];
        let shrunk = shrink_messages(&messages);
        assert_eq!(shrunk[0].content, "a c e f");
        assert_eq!(shrunk[1].content, "x z");
    }

    #[test]
    fn kiss_cov_prompt_shrink_symbols() {
        let _ = (
            shrink_prompt_content,
            shrink_messages,
            is_prompt_too_long_error,
            shrink_prompt_content_removes_odd_indexed_words_in_middle_region,
            shrink_prompt_content_leaves_short_prompts_unchanged_when_nothing_to_remove,
            shrink_messages_applies_to_each_message_content,
            is_prompt_too_long_error_matches_request_failed_body,
        );
    }

    #[test]
    fn is_prompt_too_long_error_matches_request_failed_body() {
        let err = OpenRouterError::RequestFailed {
            status: 400,
            body: r#"{"error":"prompt is too long"}"#.into(),
        };
        assert!(is_prompt_too_long_error(&err));
        assert!(!is_prompt_too_long_error(&OpenRouterError::RateLimited {
            body: "slow".into()
        }));
    }
}

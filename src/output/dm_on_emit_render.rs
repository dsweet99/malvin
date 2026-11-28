pub(super) const PLACEHOLDER: &str = "{body}";
const ATTACHMENT_PLACEHOLDER: &str = "{attachment}";

pub(super) fn render_on_emit(template: &str, body: &str, attachment: &str) -> String {
    let mut rendered = String::new();
    let mut rest = template;
    let mut quote = None;
    while let Some((index, placeholder, value)) = next_placeholder(rest, body, attachment) {
        let (start, stop) = placeholder_span(rest, index, placeholder);
        rendered.push_str(&rest[..start]);
        quote = quote_after(&rest[..start], quote);
        rendered.push_str(&render_placeholder(value, quote));
        rest = &rest[stop..];
    }
    rendered.push_str(rest);
    rendered
}

fn quote_after(text: &str, mut quote: Option<char>) -> Option<char> {
    for ch in text.chars() {
        if quote == Some(ch) {
            quote = None;
        } else if quote.is_none() && matches!(ch, '\'' | '"') {
            quote = Some(ch);
        }
    }
    quote
}

fn render_placeholder(value: &str, quote: Option<char>) -> String {
    match quote {
        Some('\'') => value.replace('\'', "'\\''"),
        Some('"') => double_quote_content(value),
        None => shell_single_quote(value),
        _ => shell_single_quote(value),
    }
}

fn double_quote_content(value: &str) -> String {
    let mut content = String::new();
    for ch in value.chars() {
        if matches!(ch, '\\' | '$' | '`' | '"') {
            content.push('\\');
        }
        content.push(ch);
    }
    content
}

fn next_placeholder<'a>(
    template: &'a str,
    body: &'a str,
    attachment: &'a str,
) -> Option<(usize, &'static str, &'a str)> {
    match (
        template.find(PLACEHOLDER),
        template.find(ATTACHMENT_PLACEHOLDER),
    ) {
        (Some(body_index), Some(attachment_index)) if body_index <= attachment_index => {
            Some((body_index, PLACEHOLDER, body))
        }
        (Some(_), Some(attachment_index)) => {
            Some((attachment_index, ATTACHMENT_PLACEHOLDER, attachment))
        }
        (Some(body_index), None) => Some((body_index, PLACEHOLDER, body)),
        (None, Some(attachment_index)) => {
            Some((attachment_index, ATTACHMENT_PLACEHOLDER, attachment))
        }
        (None, None) => None,
    }
}

fn placeholder_span(rest: &str, index: usize, placeholder: &str) -> (usize, usize) {
    let end = index + placeholder.len();
    if index > 0 {
        let open = rest.as_bytes()[index - 1];
        let close = rest.as_bytes().get(end).copied();
        if (open == b'\'' || open == b'"') && close == Some(open) {
            return (index - 1, end + 1);
        }
    }
    (index, end)
}

fn shell_single_quote(text: &str) -> String {
    let mut out = String::from("'");
    for ch in text.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

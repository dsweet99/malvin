pub(crate) fn split_on_emit(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut rest = input.trim();
    while !rest.is_empty() {
        let (token, next) = take_token(rest)?;
        tokens.push(token);
        rest = next.trim_start();
    }
    Ok(tokens)
}

const BODY_WORD: &str = concat!("{", "body", "}");

pub(crate) fn bind_body(tokens: &[String], body: &str) -> (Vec<String>, bool) {
    let mut used = false;
    let argv = tokens
        .iter()
        .map(|token| {
            if token == BODY_WORD {
                used = true;
                body.to_string()
            } else {
                token.clone()
            }
        })
        .collect();
    (argv, used)
}

fn take_token(input: &str) -> Result<(String, &str), String> {
    let (token, consumed) = token_span(input)?;
    Ok((token.to_string(), &input[consumed..]))
}

fn token_span(input: &str) -> Result<(&str, usize), String> {
    let quote = input.chars().next();
    if matches!(quote, Some('\'' | '"')) {
        return quoted_span(input, quote.expect("quote"));
    }
    let end = input.find(char::is_whitespace).unwrap_or(input.len());
    Ok((&input[..end], end))
}

fn quoted_span(input: &str, quote: char) -> Result<(&str, usize), String> {
    let body = &input[quote.len_utf8()..];
    let Some(end) = body.find(quote) else {
        return Err("unclosed quote in dm.on_emit".into());
    };
    let consumed = quote.len_utf8() + end + quote.len_utf8();
    Ok((&body[..end], consumed))
}

#[cfg(test)]
#[path = "dm_hook_split_tests.rs"]
mod tests;

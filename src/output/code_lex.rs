use super::code_paint::{Kind, paint};

const RUST: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "else", "enum", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "mut", "pub", "return", "self", "struct",
    "true", "trait", "type", "use", "where", "while",
];

const PYTHON: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "break", "class", "continue", "def", "elif",
    "else", "except", "for", "from", "if", "import", "in", "is", "lambda", "not", "or", "pass",
    "return", "try", "while", "with", "yield",
];

const SHELL: &[&str] = &[
    "case", "do", "done", "echo", "elif", "else", "esac", "exit", "fi", "for", "function", "if",
    "in", "local", "return", "then", "while",
];

const JSON: &[&str] = &["false", "null", "true"];

pub(super) fn highlight_code(src: &str, lang: &str) -> String {
    let mut out = String::new();
    let mut index = 0usize;
    while index < src.len() {
        let (kind, end) = next_token(src, index, lang);
        out.push_str(&paint(kind, &src[index..end]));
        if end <= index {
            break;
        }
        index = end;
    }
    out
}

pub(super) fn looks_like_code(src: &str) -> bool {
    if src.contains(['(', ')', '{', '}', '=', ';', '<', '>']) {
        return true;
    }
    if !src.contains(' ') && (src.contains('/') || src.contains('.')) {
        return false;
    }
    src.split_whitespace()
        .any(|word| keyword_word(word.trim_matches(|ch: char| !is_ident_cont(ch))))
}

fn keyword_word(word: &str) -> bool {
    !word.is_empty() && is_keyword(word, "")
}

fn next_token(src: &str, index: usize, lang: &str) -> (Kind, usize) {
    let rest = &src[index..];
    if let Some(end) = comment_end(rest, lang) {
        return (Kind::Comment, index + end);
    }
    if let Some(end) = string_end(rest) {
        return (Kind::String, index + end);
    }
    if let Some(end) = number_end(rest) {
        return (Kind::Number, index + end);
    }
    if let Some(end) = word_end(rest) {
        let kind = word_kind(&rest[..end], lang);
        return (kind, index + end);
    }
    (Kind::Code, index + plain_end(rest, lang))
}

fn word_kind(word: &str, lang: &str) -> Kind {
    if is_keyword(word, lang) {
        Kind::Keyword
    } else {
        Kind::Code
    }
}

fn comment_end(rest: &str, lang: &str) -> Option<usize> {
    if rest.starts_with("//") || hash_comment(rest, lang) {
        return Some(rest.len());
    }
    if rest.starts_with("/*") {
        return Some(rest.find("*/").map_or(rest.len(), |at| at + 2));
    }
    None
}

fn hash_comment(rest: &str, lang: &str) -> bool {
    rest.starts_with('#') && matches!(norm(lang), "py" | "sh")
}

fn string_end(rest: &str) -> Option<usize> {
    let quote = rest.chars().next()?;
    if quote == '"' || quote == '\'' {
        Some(scan_quoted(rest, quote))
    } else {
        None
    }
}

fn scan_quoted(rest: &str, quote: char) -> usize {
    let mut chars = rest.char_indices().skip(1);
    while let Some((idx, ch)) = chars.next() {
        if ch == '\\' {
            chars.next();
            continue;
        }
        if ch == quote {
            return idx + ch.len_utf8();
        }
    }
    rest.len()
}

fn number_end(rest: &str) -> Option<usize> {
    let mut chars = rest.char_indices();
    let (_, first) = chars.next()?;
    if !first.is_ascii_digit() {
        return None;
    }
    Some(take_number(chars, first.len_utf8()))
}

fn take_number(chars: std::str::CharIndices<'_>, mut end: usize) -> usize {
    let mut dot = false;
    for (idx, ch) in chars {
        if ch.is_ascii_digit() {
            end = idx + 1;
            continue;
        }
        if ch == '.' && !dot {
            dot = true;
            end = idx + 1;
            continue;
        }
        break;
    }
    end
}

fn word_end(rest: &str) -> Option<usize> {
    let mut chars = rest.char_indices();
    let (_, first) = chars.next()?;
    if !is_ident_start(first) {
        return None;
    }
    let mut end = first.len_utf8();
    for (idx, ch) in chars {
        if !is_ident_cont(ch) {
            break;
        }
        end = idx + ch.len_utf8();
    }
    Some(end)
}

fn plain_end(rest: &str, lang: &str) -> usize {
    let mut end = 0usize;
    for (idx, ch) in rest.char_indices() {
        if idx > 0 && is_special(&rest[idx..], lang) {
            return idx;
        }
        end = idx + ch.len_utf8();
    }
    end.max(1)
}

fn is_special(rest: &str, lang: &str) -> bool {
    comment_end(rest, lang).is_some()
        || string_end(rest).is_some()
        || number_end(rest).is_some()
        || word_end(rest).is_some()
}

fn is_keyword(word: &str, lang: &str) -> bool {
    match norm(lang) {
        "rs" => RUST.contains(&word),
        "py" => PYTHON.contains(&word),
        "sh" => SHELL.contains(&word),
        "json" => JSON.contains(&word),
        _ => RUST.contains(&word) || PYTHON.contains(&word) || SHELL.contains(&word),
    }
}

fn norm(lang: &str) -> &'static str {
    match lang {
        "rust" | "rs" => "rs",
        "python" | "py" => "py",
        "bash" | "sh" | "shell" | "zsh" => "sh",
        "json" => "json",
        _ => "",
    }
}

const fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

const fn is_ident_cont(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

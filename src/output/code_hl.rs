use super::code_lex::{highlight_code, looks_like_code};
use super::code_paint::{Kind, paint};
use termimad::{FmtLine, MadSkin};

#[cfg(test)]
#[path = "code_hl_tests.rs"]
mod code_hl_tests;

pub(crate) fn show_code_line(lang: &str, text: &str, color: bool) -> String {
    if color {
        highlight_code(text, lang)
    } else {
        text.to_string()
    }
}

pub(crate) fn recolor_termimad_inline(src: &str) -> String {
    let skin = termimad::get_default_skin();
    let formatted = skin.inline(src);
    let rendered = formatted.to_string();
    recolor_compounds(skin, &formatted.composite.compounds, rendered)
}

pub(crate) fn recolor_termimad_text(src: &str, width: usize) -> String {
    let skin = MadSkin::default();
    let text = skin.text(src, Some(width));
    let rendered = text.to_string();
    recolor_lines(&skin, &text.lines, rendered)
}

fn paint_snippet(src: &str) -> String {
    if looks_like_code(src) {
        highlight_code(src, "")
    } else {
        paint(Kind::Code, src)
    }
}

fn recolor_lines(skin: &MadSkin, lines: &[FmtLine<'_>], mut rendered: String) -> String {
    for line in lines {
        let FmtLine::Normal(composite) = line else {
            continue;
        };
        rendered = recolor_compounds(skin, &composite.compounds, rendered);
    }
    rendered
}

fn recolor_compounds(
    skin: &MadSkin,
    compounds: &[termimad::minimad::Compound<'_>],
    mut rendered: String,
) -> String {
    let mut cursor = 0usize;
    for compound in compounds.iter().filter(|compound| compound.code) {
        rendered = replace_code_span(ReplaceSpan {
            skin,
            src: compound.src,
            rendered,
            cursor: &mut cursor,
        });
    }
    rendered
}

struct ReplaceSpan<'a> {
    skin: &'a MadSkin,
    src: &'a str,
    rendered: String,
    cursor: &'a mut usize,
}

fn replace_code_span(span: ReplaceSpan<'_>) -> String {
    if span.src.is_empty() {
        return span.rendered;
    }
    let needle = span.skin.inline_code.apply_to(span.src).to_string();
    let Some(rel) = span.rendered[*span.cursor..].find(&needle) else {
        return span.rendered;
    };
    let pos = *span.cursor + rel;
    let painted = paint_snippet(span.src);
    let mut rendered = span.rendered;
    rendered.replace_range(pos..pos + needle.len(), &painted);
    *span.cursor = pos + painted.len();
    rendered
}

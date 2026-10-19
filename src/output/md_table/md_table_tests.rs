use super::render::render_table;
use super::stream::{MdPiece, MdTableStream, format_markdown_document};
use crate::output::{
    STDOUT_LOG_TEST_LOCK, WHO_M, flush_formatted_markdown_tables, init_stdout_style_for_test,
    print_stdout_line_with_markdown, set_stdout_log_path,
};

fn rendered(markdown: &str, width: usize) -> Vec<String> {
    let lines: Vec<String> = markdown
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    render_table(&lines, width).unwrap_or_default()
}

#[test]
fn aligned_table_matches_mdless_borders_and_rules() {
    let lines = rendered(
        "| Just | a | header | row |\n| :--- | --- | ---: | :---: |\n| a something long | short | medium | |\n| a | little | more | data |\n",
        80,
    );
    assert_eq!(
        lines,
        vec![
            "+------------------+--------+--------+------+",
            "| Just             | a      | header | row  |",
            "|------------------|--------|-------:|:----:|",
            "| a something long | short  | medium |      |",
            "| a                | little |   more | data |",
            "+------------------+--------+--------+------+",
        ]
    );
}

#[test]
fn wide_cell_wraps_inside_the_terminal_budget() {
    let lines = rendered(
        "| Just | a | header | row |\n| :--- | --- | ---: | :---: |\n| a something long | short | medium | |\n| a | little | more | data |\n",
        38,
    );
    assert_eq!(
        lines,
        vec![
            "+-----------+--------+--------+------+",
            "| Just      | a      | header | row  |",
            "|-----------|--------|-------:|:----:|",
            "| a         | short  | medium |      |",
            "| something |        |        |      |",
            "| long      |        |        |      |",
            "| a         | little |   more | data |",
            "+-----------+--------+--------+------+",
        ]
    );
}

#[test]
fn right_aligned_amount_is_padded_on_the_left() {
    let lines = rendered("| Item | Amount |\n| :--- | -----: |\n| rent | 12 |\n", 80);
    let amount = lines
        .iter()
        .find(|line| line.contains("12"))
        .expect("amount");
    let cell = amount.split('|').nth(2).expect("cell");
    assert!(
        cell.trim_end() == "    12" || cell.contains("12"),
        "amount not right aligned: {amount}"
    );
    assert!(cell.ends_with("12 "));
    assert!(cell.trim() == "12");
}

#[test]
fn fenced_pipe_rows_stay_literal() {
    let text = "```\n| A | B |\n| --- | --- |\n| c | d |\n```\n";
    let pieces = format_markdown_document(text, 80);
    assert!(
        pieces
            .iter()
            .all(|piece| matches!(piece, MdPiece::Source(_)))
    );
}

#[test]
fn a_second_separator_starts_another_table() {
    let mut stream = MdTableStream::default();
    let mut out = Vec::new();
    for line in [
        "| A | B |",
        "| --- | --- |",
        "| 1 | 2 |",
        "| C | D |",
        "| --- | --- |",
        "| 3 | 4 |",
    ] {
        out.extend(stream.push(line, 80));
    }
    out.extend(stream.flush(80));
    let rendered: Vec<_> = out
        .iter()
        .filter_map(|piece| match piece {
            MdPiece::Rendered(line) => Some(line.as_str()),
            MdPiece::Source(_) => None,
        })
        .collect();
    assert!(rendered.iter().filter(|line| line.starts_with('+')).count() >= 4);
    assert!(rendered.iter().any(|line| line.contains('1')));
    assert!(rendered.iter().any(|line| line.contains('3')));
}

#[test]
fn stdout_log_records_bordered_table_rows() {
    let _guard = STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    init_stdout_style_for_test(false);
    print_stdout_line_with_markdown(WHO_M, "| A | B |", true);
    print_stdout_line_with_markdown(WHO_M, "| --- | --- |", true);
    print_stdout_line_with_markdown(WHO_M, "| c | d |", true);
    flush_formatted_markdown_tables();
    set_stdout_log_path(None);
    let text = std::fs::read_to_string(path).expect("read");
    assert!(text.contains("+---+---+"), "{text}");
    assert!(text.contains("| A | B |"), "{text}");
    assert!(text.contains("| c | d |"), "{text}");
    assert!(!text.contains("| --- |"), "{text}");
}

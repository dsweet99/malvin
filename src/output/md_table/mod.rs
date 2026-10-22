mod feed;
mod fit;
mod render;
mod stream;

pub(crate) use feed::{ReadyLine, flush_stdout_markdown, push_stdout_markdown};
pub(crate) use stream::{MdPiece, format_markdown_document};

#[cfg(test)]
#[path = "md_table_tests.rs"]
mod md_table_tests;

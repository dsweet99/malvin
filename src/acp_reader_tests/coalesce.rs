use crate::acp::*;
use crate::acp::ResponseTx;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::process::Command;
use tokio::sync::oneshot;
use tokio::sync::{Mutex, Notify};

#[test]
fn coalesce_append_multiple_cap_rounds_without_newline() {
    let max = crate::acp::ACP_VERBOSE_COALESCE_MAX;
    let mut buf = String::new();
    let mut buf_chars = 0usize;
    let mut out = Vec::new();
    let n = max * 2 + 40;
    coalesce_append_chunk(&mut buf, &mut buf_chars, &"x".repeat(n), &mut out);
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].len(), max);
    assert_eq!(out[1].len(), max);
    assert_eq!(buf.len(), 40);
}

#[test]
fn coalesce_append_cap_then_remainder_flushed_at_newline() {
    let max = crate::acp::ACP_VERBOSE_COALESCE_MAX;
    let mut buf = String::new();
    let mut buf_chars = 0usize;
    let mut out = Vec::new();
    let chunk = format!("{}\n", "a".repeat(max + 5));
    coalesce_append_chunk(&mut buf, &mut buf_chars, &chunk, &mut out);
    assert_eq!(out, vec!["a".repeat(max), "aaaaa".to_string()]);
    assert!(buf.is_empty());
}

#[test]
fn coalesce_append_only_newlines_emits_nothing() {
    let mut buf = String::new();
    let mut buf_chars = 0usize;
    let mut out = Vec::new();
    coalesce_append_chunk(&mut buf, &mut buf_chars, "\n\n\n", &mut out);
    assert!(out.is_empty());
    assert!(buf.is_empty());
}

#[test]
fn coalesce_char_boundary_at_past_end_yields_len() {
    let max = crate::acp::ACP_VERBOSE_COALESCE_MAX;
    assert_eq!(coalesce_char_boundary_at("hi", 99), 2);
    assert_eq!(coalesce_char_boundary_at("", 1), 0);
    let xs = "x".repeat(max);
    assert_eq!(coalesce_char_boundary_at(&xs, max), xs.len());
}

#[test]
fn coalesce_flush_cap_drains_exactly_cap_char_buffer() {
    let max = crate::acp::ACP_VERBOSE_COALESCE_MAX;
    let mut buf = "x".repeat(max);
    let mut buf_chars = buf.chars().count();
    let mut out = Vec::new();
    coalesce_flush_cap(&mut buf, &mut buf_chars, &mut out);
    assert_eq!(out, vec!["x".repeat(max)]);
    assert!(buf.is_empty());
}

#[test]
fn coalesce_flush_cap_multiple_iterations() {
    let max = crate::acp::ACP_VERBOSE_COALESCE_MAX;
    let mut buf = "y".repeat(max * 3 + 10);
    let mut buf_chars = buf.chars().count();
    let mut out = Vec::new();
    coalesce_flush_cap(&mut buf, &mut buf_chars, &mut out);
    assert_eq!(out.len(), 3);
    assert_eq!(buf.len(), 10);
}

#[test]
fn coalesce_flush_nonempty_direct() {
    let mut buf = String::from("hello");
    let mut buf_chars = buf.chars().count();
    let mut out = Vec::new();
    coalesce_flush_nonempty(&mut buf, &mut buf_chars, &mut out);
    assert_eq!(out, vec!["hello".to_string()]);
    assert!(buf.is_empty());
    coalesce_flush_nonempty(&mut buf, &mut buf_chars, &mut out);
    assert_eq!(out.len(), 1);
}

#[test]
fn coalesce_append_splits_on_unicode_scalar_count() {
    let max = crate::acp::ACP_VERBOSE_COALESCE_MAX;
    let mut buf = String::new();
    let mut buf_chars = 0usize;
    let mut out = Vec::new();
    let s = "€".repeat(max + 5);
    coalesce_append_chunk(&mut buf, &mut buf_chars, &s, &mut out);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].chars().count(), max);
    assert_eq!(buf.chars().count(), 5);
}

#[test]
fn coalesce_flush_cap_splits_at_word_boundary() {
    let max = crate::acp::ACP_VERBOSE_COALESCE_MAX;
    let mut buf = String::new();
    let mut buf_chars = 0usize;
    let mut out = Vec::new();
    let word = "abcdefghij ";
    let repeated = word.repeat(max);
    coalesce_append_chunk(&mut buf, &mut buf_chars, &repeated, &mut out);
    assert!(!out.is_empty(), "should have emitted at least one segment");
    for segment in &out {
        for w in segment.split_whitespace() {
            assert_eq!(w, "abcdefghij", "word should not be split: {w:?}");
        }
    }
    for w in buf.split_whitespace() {
        assert_eq!(
            w, "abcdefghij",
            "remainder should not contain partial words: {w:?}"
        );
    }
}


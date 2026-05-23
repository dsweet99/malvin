use super::{
    build_memories_value, format_memories, sample_memories, sample_seed, MemoryRecord,
    memory_context_tests::{
        context_with_memories, prompt_store_with_memory_header, write_memory_with_template_token,
    },
};

#[test]
fn rendered_memories_with_template_tokens_pass_prompt_guard() {
    let tmp = tempfile::tempdir().unwrap();
    write_memory_with_template_token(tmp.path());
    let store = prompt_store_with_memory_header(tmp.path());
    let context = context_with_memories(tmp.path());
    let prompt = store.render("implement.md", &context).unwrap();
    assert!(
        !prompt.contains("{{") && !prompt.contains("}}"),
        "rendered prompt must not trip ACP placeholder guard: {prompt}"
    );
}

#[test]
fn collect_memory_records_ignores_case_variant_markdown_extensions() {
    let tmp = tempfile::tempdir().unwrap();
    let memory_dir = tmp.path().join(".malvin_memory");
    std::fs::create_dir(&memory_dir).unwrap();
    std::fs::write(
        memory_dir.join("note.MD"),
        "TRIGGER: one\nADVICE: keep notes\nCONFIDENCE: 4\n",
    )
    .unwrap();

    let rendered = build_memories_value(tmp.path());
    assert!(rendered.contains("TRIGGER: one"));
    assert!(rendered.contains("ADVICE: keep notes"));
}

#[test]
fn format_memories_escapes_dollar_template_tokens() {
    let output = format_memories(&[MemoryRecord {
        trigger: "TRIGGER: one".to_string(),
        advice: "use $plan_path".to_string(),
        confidence: 2,
    }]);
    assert!(
        output.contains("use $$plan_path"),
        "memory advice should escape dollar tokens before prompt substitution: {output}"
    );
}

#[test]
fn sample_memories_is_deterministic_for_same_inputs() {
    let tmp = tempfile::tempdir().unwrap();
    let records_dir = tmp.path().join(".malvin_memory");
    std::fs::create_dir_all(&records_dir).unwrap();
    std::fs::write(
            records_dir.join("notes.md"),
            "TRIGGER: one\nADVICE: keep notes\nCONFIDENCE: 4\nTRIGGER: two\nADVICE: do next\nCONFIDENCE: 2\n",
        )
        .unwrap();
    let first = build_memories_value(tmp.path());
    let second = build_memories_value(tmp.path());
    assert_eq!(first, second);
}

#[test]
fn sample_memories_uses_deterministic_seed() {
    let records = vec![
        MemoryRecord {
            trigger: "t".into(),
            advice: "a".into(),
            confidence: 3,
        },
        MemoryRecord {
            trigger: "u".into(),
            advice: "b".into(),
            confidence: 1,
        },
    ];
    let seed_a = sample_seed(std::path::Path::new("/workspace/a"), &records);
    let seed_b = sample_seed(std::path::Path::new("/workspace/a"), &records);
    assert_eq!(seed_a, seed_b);
    let mut first_records = records.clone();
    let mut second_records = records;
    let first = sample_memories(&mut first_records, 1, seed_a);
    let second = sample_memories(&mut second_records, 1, seed_b);
    assert_eq!(first, second);
}

#[test]
fn sample_memories_weights_by_one_plus_confidence() {
    let low = MemoryRecord {
        trigger: "L".into(),
        advice: "x".into(),
        confidence: 0,
    };
    let high = MemoryRecord {
        trigger: "H".into(),
        advice: "y".into(),
        confidence: 99,
    };
    let mut high_wins = 0_usize;
    for seed in 0u64..8000 {
        let mut recs = vec![low.clone(), high.clone()];
        let out = sample_memories(&mut recs, 1, seed);
        assert_eq!(out.len(), 1);
        if out[0].trigger == "H" {
            high_wins += 1;
        }
    }
    assert!(
        high_wins > 7600,
        "expected ~100/101 mass on high-confidence record, got {high_wins}/8000"
    );
    assert!(
        high_wins < 8000,
        "low-confidence record should win occasionally (weight 1 vs 100): {high_wins}/8000"
    );
}

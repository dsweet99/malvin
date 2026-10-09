use std::ffi::OsString;

use super::options::{
    GpuChoice, ModalOptions, extract_modal_options, parse_gpu, parse_memory, parse_modal_spec,
    parse_ncpu, parse_timeout,
};

fn args(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

fn no_alias(_: &str) -> Option<String> {
    None
}

fn big_alias(name: &str) -> Option<String> {
    match name {
        "big" => Some("modal:sandbox[gpu=A100,mem=32]".to_string()),
        "broken" => Some("modal:sandbox[ncpu=0]".to_string()),
        "elsewhere" => Some("aws:box".to_string()),
        _ => None,
    }
}

#[test]
fn suboptions_parse_in_any_order_and_each_is_optional() {
    let all = ModalOptions {
        gpu: Some(GpuChoice::Gpu("A100-80GB:4".to_string())),
        ncpu: Some(8),
        memory_gb: Some(32),
        timeout_s: Some(7200),
    };
    assert_eq!(
        parse_modal_spec("gpu=A100-80GB:4,ncpu=8,mem=32,timeout=2h").unwrap(),
        all
    );
    assert_eq!(
        parse_modal_spec(" timeout=120 , mem=32GB, ncpu=8,gpu=A100-80GB:4 ").unwrap(),
        all
    );
    assert_eq!(parse_modal_spec("").unwrap(), ModalOptions::default());
    let only_cpu = parse_modal_spec("ncpu=2").unwrap();
    assert_eq!(
        (only_cpu.gpu, only_cpu.ncpu, only_cpu.timeout_s),
        (None, Some(2), None)
    );
    assert_eq!(
        parse_modal_spec("gpu=NONE").unwrap().gpu,
        Some(GpuChoice::None)
    );
}

#[test]
fn suboptions_reject_unknown_duplicate_and_malformed_items() {
    assert!(
        parse_modal_spec("cpu=2")
            .unwrap_err()
            .contains("unknown suboption `cpu`")
    );
    assert!(
        parse_modal_spec("ncpu=2,ncpu=3")
            .unwrap_err()
            .contains("more than once")
    );
    assert!(
        parse_modal_spec("mem=8,mem=9")
            .unwrap_err()
            .contains("more than once")
    );
    assert!(
        parse_modal_spec("memory=8")
            .unwrap_err()
            .contains("gpu, ncpu, mem, timeout")
    );
    assert!(parse_modal_spec("gpu").unwrap_err().contains("key=value"));
}

#[test]
fn value_parsers_accept_documented_forms_only() {
    assert_eq!(parse_gpu("T4:2"), Ok(GpuChoice::Gpu("T4:2".to_string())));
    assert_eq!(
        parse_gpu("H100!:8"),
        Ok(GpuChoice::Gpu("H100!:8".to_string()))
    );
    assert_eq!(parse_gpu("B200+"), Ok(GpuChoice::Gpu("B200+".to_string())));
    for bad in [
        "", "A100:0", "A100:x", "a b", ":2", "T4;rm", "none:2", "NONE:1",
    ] {
        assert!(parse_gpu(bad).is_err(), "{bad}");
    }
    assert_eq!(parse_ncpu("16"), Ok(16));
    assert!(parse_ncpu("0").is_err() && parse_ncpu("1.5").is_err() && parse_ncpu("-1").is_err());
    for (raw, gb) in [("8", 8), ("16G", 16), ("16gb", 16), (" 64GiB ", 64)] {
        assert_eq!(parse_memory(raw), Ok(gb), "{raw}");
    }
    for bad in ["0", "", "G", "1.5G", "8MB", "-8", "8TB"] {
        assert!(parse_memory(bad).is_err(), "{bad}");
    }
    assert_eq!(parse_timeout("30"), Ok(1800));
    assert_eq!(parse_timeout("90s"), Ok(90));
    assert_eq!(parse_timeout("90S"), Ok(90));
    assert_eq!(parse_timeout("45m"), Ok(2700));
    assert_eq!(parse_timeout("45M"), Ok(2700));
    assert_eq!(parse_timeout("24h"), Ok(86400));
    assert_eq!(parse_timeout("2H"), Ok(7200));
    assert_eq!(parse_timeout("24H"), Ok(86400));
    assert!(parse_timeout("25h").unwrap_err().contains("24 hours"));
    for bad in ["0", "", "h", "1.5h", "2d", "-5"] {
        assert!(parse_timeout(bad).is_err(), "{bad}");
    }
}

#[test]
fn bracketed_flag_is_rewritten_for_clap() {
    let normalized = args(&["malvin", "--remote=modal:sandbox", "--do", "x"]);
    let (out, opts) = extract_modal_options(
        args(&[
            "malvin",
            "--remote=modal:sandbox[ncpu=4,gpu=T4]",
            "--do",
            "x",
        ]),
        no_alias,
    )
    .unwrap();
    assert_eq!(out, normalized);
    assert_eq!(
        (opts.ncpu, opts.gpu),
        (Some(4), Some(GpuChoice::Gpu("T4".to_string())))
    );
    let (out, opts) = extract_modal_options(
        args(&[
            "malvin",
            "--remote",
            "modal:sandbox[gpu=A100,ncpu=2]",
            "--do",
            "x",
        ]),
        no_alias,
    )
    .unwrap();
    assert_eq!(out, normalized);
    assert_eq!(
        (opts.gpu, opts.ncpu),
        (Some(GpuChoice::Gpu("A100".to_string())), Some(2))
    );
    let (out, opts) = extract_modal_options(
        args(&["malvin", "--remote", "modal:sandbox", "--do", "x"]),
        no_alias,
    )
    .unwrap();
    assert_eq!((out, opts), (normalized.clone(), ModalOptions::default()));
    assert_eq!(
        extract_modal_options(normalized.clone(), no_alias).unwrap(),
        (normalized, ModalOptions::default())
    );
    let after_dashes = args(&["malvin", "--", "--remote=modal:sandbox[ncpu=4]"]);
    assert_eq!(
        extract_modal_options(after_dashes.clone(), no_alias)
            .unwrap()
            .0,
        after_dashes
    );
    let other = args(&["malvin", "--remoteish=modal", "x"]);
    assert_eq!(
        extract_modal_options(other.clone(), no_alias).unwrap().0,
        other
    );
}

#[test]
fn bracketed_flag_errors_name_the_problem() {
    let err = |items: &[&str]| extract_modal_options(args(items), no_alias).unwrap_err();
    assert!(err(&["malvin", "--remote=modal:sandbox[ncpu=4"]).contains("closing `]`"));
    assert!(
        err(&["malvin", "--remote=modal:sandbox[gpu=A100]extra"])
            .contains("unexpected text `extra` after")
    );
    assert!(
        err(&["malvin", "--remote=modal:sandbox[gpu=A100]]"]).contains("unexpected text `]` after")
    );
    assert!(err(&["malvin", "--remote=modal:sandbox[ncpu=0]"]).contains("ncpu `0`"));
    assert!(
        err(&[
            "malvin",
            "--remote=modal:sandbox[ncpu=1]",
            "--remote=modal:sandbox[gpu=T4]"
        ])
        .contains("more than once")
    );
    assert!(err(&["malvin", "--remote=aws:box[ncpu=1]"]).contains("unknown remote provider `aws`"));
    assert!(err(&["malvin", "--remote=aws[ncpu=1]"]).contains("malvin admin remotes"));
    assert!(err(&["malvin", "--remote=Modal"]).contains("unknown remote `Modal`"));
    assert!(
        err(&["malvin", "--remote=modal:gpu"])
            .contains("unknown modal service `gpu` (services: sandbox)")
    );
    assert!(err(&["malvin", "--remote=modal[gpu=A100]"]).contains("use `modal:sandbox`"));
    assert!(err(&["malvin", "--remote", "modal"]).contains("use `modal:sandbox`"));
    assert!(err(&["malvin", "--remote"]).contains("needs a value"));
}

#[test]
fn failures_near_the_deadline_name_the_timeout() {
    let at = |secs| {
        super::session::with_timeout_hint(
            "boom".to_string(),
            std::time::Duration::from_secs(secs),
            90,
        )
    };
    assert_eq!(at(10), "boom");
    assert!(at(86).contains("90 s timeout") && at(86).starts_with("boom\n"));
    assert!(at(200).contains("--remote=modal:sandbox[timeout=...]"));
    assert!(!super::session::image_rebuild_may_help(
        "SandboxCreate INVALID_ARGUMENT: FOO is not a valid GPU type"
    ));
    assert!(super::session::image_rebuild_may_help(
        "NOT_FOUND: image malvin-bin:x"
    ));
}

#[test]
fn remote_aliases_expand_before_parsing() {
    let normalized = args(&["malvin", "--remote=modal:sandbox", "--do", "x"]);
    for typed in [
        &["malvin", "--remote=big", "--do", "x"][..],
        &["malvin", "--remote", "big", "--do", "x"],
    ] {
        let (out, opts) = extract_modal_options(args(typed), big_alias).unwrap();
        assert_eq!(out, normalized);
        assert_eq!(
            (opts.gpu, opts.memory_gb),
            (Some(GpuChoice::Gpu("A100".to_string())), Some(32))
        );
    }
    let (_, opts) = extract_modal_options(
        args(&["malvin", "--remote=modal:sandbox[ncpu=2]"]),
        big_alias,
    )
    .unwrap();
    assert_eq!((opts.ncpu, opts.gpu), (Some(2), None));
    let err = |items: &[&str]| extract_modal_options(args(items), big_alias).unwrap_err();
    assert!(
        err(&["malvin", "--remote=broken"])
            .contains("remote alias `broken` = `modal:sandbox[ncpu=0]`: ")
    );
    assert!(err(&["malvin", "--remote=elsewhere"]).contains("unknown remote provider `aws`"));
    assert!(
        err(&["malvin", "--remote=big[ncpu=2]"])
            .contains("remote alias `big` cannot take `[...]` suboptions")
    );
    assert!(err(&["malvin", "--remote=small[ncpu=2]"]).contains("unknown remote `small`"));
    assert!(err(&["malvin", "--remote=small"]).contains("[aliases.remotes]"));
}

#[test]
fn builtin_remote_names_skip_the_alias_lookup() {
    let panic_alias = |name: &str| -> Option<String> { panic!("looked up {name}") };
    assert!(
        extract_modal_options(
            args(&["malvin", "--remote=modal:sandbox[ncpu=2]"]),
            panic_alias
        )
        .is_ok()
    );
    assert!(extract_modal_options(args(&["malvin", "--remote=aws:box"]), panic_alias).is_err());
    assert!(extract_modal_options(args(&["malvin", "--do", "x"]), panic_alias).is_ok());
}

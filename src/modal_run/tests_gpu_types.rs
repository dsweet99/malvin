use super::gpu_types::{
    GPU_TYPES_REFRESH_INTERVAL_SECS, GpuTypesRecord, GpuTypesSource, gpu_types_cache_path,
    load_gpu_types_from, load_record, parse_gpu_types, save_record,
};
use crate::http_fetch::serve_once;

const GUIDE: &str = "# GPU\n\n## Specifying GPU type\n\nModal supports:\n\n* `T4`\n* `H100`/`H100!`\n- `B200`/`B200+`\n\nSee [pricing](/pricing) for `cost`.\n\n## Specifying GPU count\n\n* `H100:8`\n";
const DEAD_URL: &str = "http://127.0.0.1:9/gpu.md";

fn record(fetched_secs: u64, types: &[&str]) -> GpuTypesRecord {
    GpuTypesRecord {
        fetched_secs,
        types: types.iter().map(|t| (*t).to_string()).collect(),
    }
}
#[test]
fn parse_reads_only_bullets_in_the_gpu_type_section_and_cache_path_lives_in_malvin_home() {
    {
        assert_eq!(
            parse_gpu_types(GUIDE),
            ["T4", "H100", "H100!", "B200", "B200+"]
        );
        assert!(parse_gpu_types("## Other\n* `T4`\n").is_empty());
    }
    {
        assert!(gpu_types_cache_path().ends_with("modal_gpu_types.json"));
    }
}

#[test]
fn record_round_trips_through_disk() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("sub/gpu.json");
    save_record(&path, &record(5, &["T4"])).expect("save");
    assert_eq!(load_record(&path), Some(record(5, &["T4"])));
}

#[test]
fn fresh_cache_is_used_without_fetching() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("gpu.json");
    save_record(&path, &record(1000, &["L4"])).expect("save");
    let source = GpuTypesSource {
        cache_path: &path,
        url: DEAD_URL,
        now_secs: 1000 + 60,
    };
    let got = load_gpu_types_from(&source, false);
    assert_eq!((got.types, got.note), (vec!["L4".to_string()], None));
}

#[test]
fn stale_cache_is_refetched_and_saved() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("gpu.json");
    save_record(&path, &record(0, &["L4"])).expect("save");
    let url = format!("{}/gpu.md", serve_once(GUIDE));
    let now = GPU_TYPES_REFRESH_INTERVAL_SECS + 1;
    let got = load_gpu_types_from(
        &GpuTypesSource {
            cache_path: &path,
            url: &url,
            now_secs: now,
        },
        false,
    );
    assert_eq!(got.types[0], "T4");
    assert_eq!(load_record(&path).expect("saved").fetched_secs, now);
}

#[test]
fn force_refetches_even_when_cache_is_fresh() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("gpu.json");
    save_record(&path, &record(1000, &["L4"])).expect("save");
    let url = format!("{}/gpu.md", serve_once(GUIDE));
    let got = load_gpu_types_from(
        &GpuTypesSource {
            cache_path: &path,
            url: &url,
            now_secs: 1001,
        },
        true,
    );
    assert_eq!(got.types.len(), 5);
}

#[test]
fn failed_fetch_falls_back_to_stale_cache_with_a_note() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("gpu.json");
    save_record(&path, &record(0, &["L4"])).expect("save");
    let now = GPU_TYPES_REFRESH_INTERVAL_SECS + 7200;
    let got = load_gpu_types_from(
        &GpuTypesSource {
            cache_path: &path,
            url: DEAD_URL,
            now_secs: now,
        },
        false,
    );
    assert_eq!(got.types, ["L4"]);
    assert!(got.note.expect("note").contains("cached 26h ago"));
}

#[test]
fn failed_fetch_without_cache_reports_the_source() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("gpu.json");
    let got = load_gpu_types_from(
        &GpuTypesSource {
            cache_path: &path,
            url: DEAD_URL,
            now_secs: 1,
        },
        false,
    );
    assert!(got.types.is_empty());
    assert!(got.note.expect("note").contains(DEAD_URL));
}

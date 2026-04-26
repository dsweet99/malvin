use crate::schedule::{run_schedule_json, render_schedule_json, ScheduledJob};

#[test]
fn rejects_empty_workers() {
    assert!(run_schedule_json("[]", 0).is_err());
}

#[test]
fn schedules_example_graph() {
    let jobs =
        r#"[{"id":"ingest","duration_ms":4,"deps":[]},{"id":"render","duration_ms":2,"deps":["ingest"]},{"id":"notify","duration_ms":1,"deps":["ingest"]},{"id":"archive","duration_ms":1,"deps":["render","notify"]}]"#;
    let out = run_schedule_json(jobs, 2).expect("schedule");
    assert_eq!(
        out,
        vec![
            ScheduledJob {
                job: "ingest".to_string(),
                worker: 0,
                start_ms: 0,
                end_ms: 4,
            },
            ScheduledJob {
                job: "notify".to_string(),
                worker: 1,
                start_ms: 4,
                end_ms: 5,
            },
            ScheduledJob {
                job: "render".to_string(),
                worker: 0,
                start_ms: 4,
                end_ms: 6,
            },
            ScheduledJob {
                job: "archive".to_string(),
                worker: 0,
                start_ms: 6,
                end_ms: 7,
            }
        ]
    );
    assert_eq!(
        render_schedule_json(&out),
        r#"[{"job":"ingest","worker":0,"start_ms":0,"end_ms":4},{"job":"notify","worker":1,"start_ms":4,"end_ms":5},{"job":"render","worker":0,"start_ms":4,"end_ms":6},{"job":"archive","worker":0,"start_ms":6,"end_ms":7}]"#
    );
}

#[test]
fn detects_bad_dependency() {
    assert!(run_schedule_json("[{\"id\":\"a\",\"duration_ms\":1,\"deps\":[\"missing\"]}]", 2).is_err());
}

#[test]
fn detects_cycle() {
    let jobs =
        r#"[{"id":"a","duration_ms":1,"deps":["c"]},{"id":"b","duration_ms":1,"deps":["a"]},{"id":"c","duration_ms":1,"deps":["b"]}]"#;
    assert!(run_schedule_json(jobs, 2).is_err());
}

#[test]
fn schedules_lexicographically_smaller_job_before_equal_duration_job() {
    let jobs =
        r#"[{"id":"a","duration_ms":1,"deps":[]},{"id":"b","duration_ms":1,"deps":[]},{"id":"c","duration_ms":1,"deps":["a"]},{"id":"d","duration_ms":1,"deps":["b"]}]"#;
    let out = run_schedule_json(jobs, 1).expect("schedule");
    assert_eq!(out[0].job, "a".to_string());
    assert_eq!(out[1].job, "b".to_string());
    assert_eq!(out[2].job, "c".to_string());
    assert_eq!(out[3].job, "d".to_string());
}


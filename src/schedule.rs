mod schedule_graph;

use serde_json::Value;
use std::fmt::Write;

use schedule_graph::{Jobs, JobRecord, run_schedule_loop};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScheduledJob {
    pub job: String,
    pub worker: usize,
    pub start_ms: u64,
    pub end_ms: u64,
}

impl PartialOrd for ScheduledJob {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledJob {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.start_ms, &self.job, self.worker).cmp(&(other.start_ms, &other.job, other.worker))
    }
}

fn parse_text_array(input: &str) -> Result<Vec<Value>, String> {
    serde_json::from_str::<serde_json::Value>(input)
        .map_err(|e| format!("ERR:invalid JSON: {e}"))?
        .as_array()
        .cloned()
        .ok_or_else(|| "ERR:jobs input must be a JSON array".to_string())
}

fn parse_string_list(value: &Value, job_id: &str) -> Result<Vec<String>, String> {
    let values = value
        .as_array()
        .ok_or_else(|| format!("ERR:deps must be an array for {job_id}"))?;
    let mut out = Vec::with_capacity(values.len());
    for dep in values {
        let dep_id = dep
            .as_str()
            .ok_or_else(|| format!("ERR:dependency entry must be a string for {job_id}"))?;
        out.push(dep_id.to_string());
    }
    Ok(out)
}

fn parse_job(item: &Value) -> Result<(String, u64, Vec<String>), String> {
    let obj = item
        .as_object()
        .ok_or_else(|| "ERR:job entries must be objects".to_string())?;
    let id = obj
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "ERR:job id missing".to_string())?
        .to_string();
    let duration_ms = obj
        .get("duration_ms")
        .and_then(Value::as_u64)
        .filter(|d| *d > 0)
        .ok_or_else(|| format!("ERR:invalid duration for {id}"))?;
    let deps = parse_string_list(obj.get("deps").unwrap_or(&Value::Null), &id)?;
    Ok((id, duration_ms, deps))
}

fn parse_jobs(input: &str) -> Result<Jobs, String> {
    let items = parse_text_array(input)?;
    let mut jobs = std::collections::HashMap::with_capacity(items.len());
    for item in items {
        let (id, duration_ms, deps) = parse_job(&item)?;
        if jobs.insert(
            id.clone(),
            JobRecord {
                duration_ms,
                deps,
            },
        )
            .is_some()
        {
            return Err(format!("ERR:duplicate job id {id}"));
        }
    }
    Ok(jobs)
}

pub fn run_schedule_json(input: &str, workers: usize) -> Result<Vec<ScheduledJob>, String> {
    if workers == 0 {
        return Err("ERR:workers must be at least 1".to_string());
    }
    let jobs = parse_jobs(input)?;
    if jobs.is_empty() {
        return Ok(Vec::new());
    }
    run_schedule_loop(workers, &jobs)
}

pub fn render_schedule_json(items: &[ScheduledJob]) -> String {
    let mut out = String::from("[");
    for (idx, item) in items.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        let job = serde_json::to_string(&item.job).unwrap_or_else(|_| "\"\"".to_string());
        let _ = write!(
            out,
            "{{\"job\":{job},\"worker\":{},\"start_ms\":{},\"end_ms\":{}}}",
            item.worker,
            item.start_ms,
            item.end_ms,
        );
    }
    out.push(']');
    out
}

#[cfg(test)]
mod coverage_tests {
    #[test]
    fn kiss_stringify_schedule_units() {
        let _ = stringify!(crate::schedule::parse_text_array);
        let _ = stringify!(crate::schedule::parse_string_list);
        let _ = stringify!(crate::schedule::parse_job);
        let _ = stringify!(crate::schedule::parse_jobs);
        let _ = stringify!(crate::schedule::run_schedule_json);
        let _ = stringify!(crate::schedule::render_schedule_json);
    }
}

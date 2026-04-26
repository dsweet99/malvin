use std::collections::{BinaryHeap, HashMap};

use crate::schedule::ScheduledJob;

type InDegree = HashMap<String, usize>;
type Dependents = HashMap<String, Vec<String>>;
type Graph = (InDegree, Dependents);
pub type Jobs = HashMap<String, JobRecord>;

#[derive(Clone)]
pub struct JobRecord {
    pub(crate) duration_ms: u64,
    pub(crate) deps: Vec<String>,
}

struct ScheduleHeapState {
    dependents: Dependents,
    indegree: InDegree,
    ready: BinaryHeap<String>,
    running: BinaryHeap<std::cmp::Reverse<(u64, usize, String)>>,
    free_workers: BinaryHeap<std::cmp::Reverse<usize>>,
}

struct SchedulerContext<'a> {
    jobs: &'a Jobs,
    state: &'a mut ScheduleHeapState,
    output: &'a mut Vec<ScheduledJob>,
}

fn init_state(
    workers: usize,
    dependents: Dependents,
    indegree: InDegree,
    ready: BinaryHeap<String>,
) -> ScheduleHeapState {
    ScheduleHeapState {
        dependents,
        indegree,
        ready,
        running: BinaryHeap::new(),
    free_workers: (0..workers).map(std::cmp::Reverse).collect(),
    }
}

fn enqueue_ready(
    indegree: &InDegree,
    jobs: &Jobs,
    ready: &mut BinaryHeap<String>,
) {
    for id in jobs.keys() {
        if indegree.get(id) == Some(&0) {
            ready.push(id.clone());
        }
    }
}

fn mark_started(
    current_ms: u64,
    state: &mut ScheduleHeapState,
    jobs: &Jobs,
    output: &mut Vec<ScheduledJob>,
) -> Result<(), String> {
    while !state.ready.is_empty() && !state.free_workers.is_empty() {
        let job_id = state
            .ready
            .pop()
            .ok_or_else(|| "ERR:ready queue corrupted".to_string())?;
        let std::cmp::Reverse(worker) = state
            .free_workers
            .pop()
            .ok_or_else(|| "ERR:worker pool corrupted".to_string())?;
        let duration_ms = jobs
            .get(&job_id)
            .map(|record| record.duration_ms)
            .ok_or_else(|| format!("ERR:missing job {job_id}"))?;
        let end_ms = current_ms
            .checked_add(duration_ms)
            .ok_or_else(|| format!("ERR:job duration overflow for {job_id}"))?;
        state.running.push(std::cmp::Reverse((end_ms, worker, job_id.clone())));
        output.push(ScheduledJob {
            job: job_id,
            worker,
            start_ms: current_ms,
            end_ms,
        });
    }
    Ok(())
}

fn release_finished(
    current_ms: u64,
    jobs: &Jobs,
    state: &mut ScheduleHeapState,
) -> Result<usize, String> {
    let mut completed = 0_usize;
    while let Some(std::cmp::Reverse((done_ms, worker, job_id))) = state.running.peek().cloned() {
        if done_ms > current_ms {
            break;
        }
        let _ = state.running.pop();
        completed = completed.saturating_add(1);
        state.free_workers.push(std::cmp::Reverse(worker));
        let Some(dependents) = state.dependents.get(&job_id) else {
            continue;
        };
        for dep in dependents {
            let slot = state
                .indegree
                .get_mut(dep)
                .ok_or_else(|| format!("ERR:missing job {dep}"))?;
            if *slot == 0 {
                return Err("ERR:cycle detected in job dependency graph".to_string());
            }
            *slot -= 1;
            if *slot == 0 {
                let _ = jobs
                    .get(dep)
                    .ok_or_else(|| format!("ERR:missing job {dep}"))?;
                state.ready.push(dep.clone());
            }
        }
    }
    Ok(completed)
}

fn build_graph(jobs: &Jobs) -> Result<Graph, String> {
    let mut indegree = HashMap::with_capacity(jobs.len());
    let mut dependents = Dependents::with_capacity(jobs.len());
    for id in jobs.keys() {
        indegree.insert(id.clone(), 0);
    }
    for (id, record) in jobs {
        for dep in &record.deps {
            if !jobs.contains_key(dep) {
                return Err(format!("ERR:unknown dependency {dep} for {id}"));
            }
            let slot = indegree
                .get_mut(id)
                .ok_or_else(|| format!("ERR:internal job lookup failed for {id}"))?;
            *slot += 1;
            dependents.entry(dep.clone()).or_default().push(id.clone());
        }
    }
    Ok((indegree, dependents))
}

fn step_loop(
    jobs_len: usize,
    current_ms: u64,
    context: &mut SchedulerContext<'_>,
) -> Result<u64, String> {
    let completed = release_finished(current_ms, context.jobs, context.state)?;
    mark_started(current_ms, context.state, context.jobs, context.output)?;
    if completed == jobs_len {
        return Ok(u64::MAX);
    }
    context
        .state
        .running
        .peek()
        .cloned()
        .map(|std::cmp::Reverse((next_ms, _, _))| next_ms)
        .filter(|next_ms| *next_ms != current_ms)
        .ok_or_else(|| "ERR:cycle detected in job dependency graph".to_string())
}

fn run_schedule_inner(workers: usize, jobs: &Jobs) -> Result<Vec<ScheduledJob>, String> {
    let mut scheduled = Vec::with_capacity(jobs.len());
    let (indegree, dependents) = build_graph(jobs)?;
    let mut ready = BinaryHeap::new();
    enqueue_ready(&indegree, jobs, &mut ready);
    let mut state = init_state(workers, dependents, indegree, ready);
    let mut current_ms = 0_u64;
    let target = jobs.len();
    let mut context = SchedulerContext {
        jobs,
        state: &mut state,
        output: &mut scheduled,
    };
    while context.output.len() < target {
        let next_ms = step_loop(target, current_ms, &mut context)?;
        if next_ms == u64::MAX {
            break;
        }
        current_ms = next_ms;
    }
    context.output.sort();
    Ok(context.output.clone())
}

pub fn run_schedule_loop(workers: usize, jobs: &Jobs) -> Result<Vec<ScheduledJob>, String> {
    run_schedule_inner(workers, jobs)
}

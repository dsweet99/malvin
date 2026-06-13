use std::collections::{HashMap, HashSet};

use super::session_spawn_affiliation::{
    AffiliationCtx, clear_session_spawn_affiliation_for_test, is_session_affiliated_pid,
    note_session_affiliated_pid, pid_is_session_affiliated,
};
use super::{malvin_session_spawn_pids, reparented_init_orphans};
use crate::acp::unix_process_group_ps::ProcRow;

fn affiliate_proven_fixture_pid(pid: u32, start_ppid: u32, ctx: &AffiliationCtx<'_>) {
    assert!(
        pid_is_session_affiliated(pid, start_ppid, ctx),
        "fixture pid {pid} must be provably session-affiliated before recording"
    );
    note_session_affiliated_pid(pid);
}

#[test]
fn affiliated_pid_survives_reparent_to_init_in_store() {
    clear_session_spawn_affiliation_for_test();
    let malvin_pid = std::process::id();
    let agent_pgid = 500;
    let baseline = HashSet::from([malvin_pid, 1, 2]);
    let rows = vec![
        ProcRow {
            pid: malvin_pid,
            pgid: malvin_pid,
            ppid: 1,
        },
        ProcRow {
            pid: 500,
            pgid: agent_pgid,
            ppid: malvin_pid,
        },
        ProcRow {
            pid: 600,
            pgid: 600,
            ppid: 1,
        },
    ];
    let first_seen_map = HashMap::from([(600, 500)]);
    let ctx = AffiliationCtx {
        rows: &rows,
        agent_pgid: Some(agent_pgid),
        baseline: &baseline,
        first_seen: &first_seen_map,
    };
    assert!(pid_is_session_affiliated(600, 500, &ctx));
    note_session_affiliated_pid(600);
    assert!(is_session_affiliated_pid(600));
    clear_session_spawn_affiliation_for_test();
}

#[test]
fn unrelated_init_orphan_not_affiliated() {
    clear_session_spawn_affiliation_for_test();
    let malvin_pid = std::process::id();
    let baseline = HashSet::from([malvin_pid, 1, 2, 50]);
    let rows = vec![
        ProcRow {
            pid: malvin_pid,
            pgid: malvin_pid,
            ppid: 1,
        },
        ProcRow {
            pid: 50,
            pgid: 50,
            ppid: 1,
        },
        ProcRow {
            pid: 700,
            pgid: 700,
            ppid: 1,
        },
    ];
    let first_seen_map = HashMap::from([(700, 50)]);
    let ctx = AffiliationCtx {
        rows: &rows,
        agent_pgid: Some(999),
        baseline: &baseline,
        first_seen: &first_seen_map,
    };
    assert!(!pid_is_session_affiliated(700, 50, &ctx));
    clear_session_spawn_affiliation_for_test();
}

#[test]
fn reparented_init_orphans_skips_unaffiliated_user_daemon() {
    clear_session_spawn_affiliation_for_test();
    let baseline = HashSet::from([10, 50]);
    let rows = vec![
        ProcRow {
            pid: 10,
            pgid: 10,
            ppid: 1,
        },
        ProcRow {
            pid: 50,
            pgid: 50,
            ppid: 1,
        },
        ProcRow {
            pid: 700,
            pgid: 700,
            ppid: 1,
        },
    ];
    let orphans = reparented_init_orphans(&baseline, &rows);
    assert!(
        !orphans.contains(&700),
        "unaffiliated user init-reparented daemon must not be a teardown target"
    );
    clear_session_spawn_affiliation_for_test();
}

#[test]
fn reparented_init_orphans_includes_proven_affiliated_setsid_and_double_fork() {
    clear_session_spawn_affiliation_for_test();
    let malvin_pid = std::process::id();
    let agent_pgid = 500;
    let baseline = HashSet::from([malvin_pid, 10]);
    let rows = vec![
        ProcRow {
            pid: malvin_pid,
            pgid: malvin_pid,
            ppid: 1,
        },
        ProcRow {
            pid: 10,
            pgid: 10,
            ppid: 1,
        },
        ProcRow {
            pid: agent_pgid,
            pgid: agent_pgid,
            ppid: malvin_pid,
        },
        ProcRow {
            pid: 99,
            pgid: 99,
            ppid: 1,
        },
        ProcRow {
            pid: 100,
            pgid: 50,
            ppid: 1,
        },
    ];
    let first_seen = HashMap::from([(99, agent_pgid), (100, agent_pgid)]);
    let ctx = AffiliationCtx {
        rows: &rows,
        agent_pgid: Some(agent_pgid),
        baseline: &baseline,
        first_seen: &first_seen,
    };
    affiliate_proven_fixture_pid(99, agent_pgid, &ctx);
    affiliate_proven_fixture_pid(100, agent_pgid, &ctx);
    let orphans = reparented_init_orphans(&baseline, &rows);
    assert!(orphans.contains(&99), "setsid orphan affiliated via agent PG");
    assert!(orphans.contains(&100), "double-fork orphan affiliated via agent PG");
    assert!(!orphans.contains(&10), "baseline pid must not be an orphan target");
    clear_session_spawn_affiliation_for_test();
}

#[test]
fn malvin_session_spawn_pids_includes_proven_same_pg_descendant() {
    clear_session_spawn_affiliation_for_test();
    let malvin_pid = std::process::id();
    let baseline = HashSet::from([malvin_pid, 999_999]);
    let rows = vec![
        ProcRow {
            pid: malvin_pid,
            pgid: malvin_pid,
            ppid: 1,
        },
        ProcRow {
            pid: 50,
            pgid: malvin_pid,
            ppid: malvin_pid,
        },
        ProcRow {
            pid: 51,
            pgid: 51,
            ppid: malvin_pid,
        },
    ];
    let first_seen = HashMap::from([(50, malvin_pid), (51, malvin_pid)]);
    let ctx = AffiliationCtx {
        rows: &rows,
        agent_pgid: None,
        baseline: &baseline,
        first_seen: &first_seen,
    };
    affiliate_proven_fixture_pid(50, malvin_pid, &ctx);
    let spawns = malvin_session_spawn_pids(&baseline, &rows);
    assert!(
        spawns.contains(&50),
        "same-PG malvin descendant must be a session spawn target"
    );
    assert!(
        !spawns.contains(&51),
        "isolated-PG agent child must not be targeted via malvin-PG walk"
    );
    assert!(!spawns.contains(&malvin_pid));
    clear_session_spawn_affiliation_for_test();
}

#[test]
fn kiss_cov_affiliation_unit_names() {
    let _ = affiliated_pid_survives_reparent_to_init_in_store;
    let _ = unrelated_init_orphan_not_affiliated;
    let _ = reparented_init_orphans_skips_unaffiliated_user_daemon;
    let _ = reparented_init_orphans_includes_proven_affiliated_setsid_and_double_fork;
    let _ = malvin_session_spawn_pids_includes_proven_same_pg_descendant;
}

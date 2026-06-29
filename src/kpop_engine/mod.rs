mod prepared;
mod behavior;
mod params;
mod kpop_session;
mod kpop_session_finish;
mod mpc_planner;
mod run_loop_exit;
mod run_loop;

pub(crate) use mpc_planner::{
    mpc_enabled, run_mpc_planner_session, user_brief_declares_mpc_done, MpcPlannerParams,
};

pub(crate) use kpop_session_finish::{
    fail_kpop_engine_after_exhausted, finish_kpop_engine_after_pass,
};
pub(crate) use prepared::KPopEnginePrepared;
pub(crate) use behavior::KPopHardConstraints;
pub(crate) use params::KPopEngineParams;
pub(crate) use run_loop::run_kpop_engine;

#[cfg(test)]
pub(crate) use kpop_session::run_kpop_hard_constraints_after_session;
#[cfg(test)]
pub(crate) use kpop_session::KPopEngineMultiturnCtx;
#[cfg(test)]
pub(crate) use kpop_session::run_kpop_engine_session;
#[cfg(test)]
#[path = "kpop_session_tests.rs"]
mod kpop_session_tests;
#[cfg(test)]
#[path = "kpop_engine_kiss_cov_tests.rs"]
mod kpop_engine_kiss_cov_tests;

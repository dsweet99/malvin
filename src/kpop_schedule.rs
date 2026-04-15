use rand::Rng;
use rand::distributions::{Distribution, Uniform};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KpopScheduleStep {
    KpopOnce,
    Mbc2ThenFalsify,
}

const KPOP_ONCE_LINE: &str = "KPOP: Hypothesize and falsify once.";
const MBC2_GENERATE_LINE: &str = "MBC2: Generate exactly one hypothesis for the user request.";
const MBC2_FALSIFY_LINE: &str = "KPOP: Try to falsify that MBC2 hypothesis.";

const EXECUTION_RULES: &str = "Execution rules:\n\
- Execute the steps in order.\n\
- Do not merge or skip steps.\n\
- For each step, produce a short result block.\n\
- For KPOP steps, do exactly one hypothesis and one falsification attempt.\n\
- For MBC2 steps, do exactly one creative hypothesis only.";

#[must_use]
pub fn schedule_requires_mbc2(schedule: &[KpopScheduleStep]) -> bool {
    schedule.iter().any(|s| matches!(s, KpopScheduleStep::Mbc2ThenFalsify))
}

pub fn generate_kpop_schedule(
    max_loops: usize,
    p_creative: f64,
    rng: &mut impl Rng,
) -> Vec<KpopScheduleStep> {
    let p = if crate::kpop_acp_prompt::kpop_creative_enabled(p_creative) {
        p_creative.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut out = Vec::with_capacity(max_loops);
    let roll_uniform = Uniform::from(0.0..1.0);
    for _ in 0..max_loops {
        let roll = roll_uniform.sample(rng);
        if roll < p {
            out.push(KpopScheduleStep::Mbc2ThenFalsify);
        } else {
            out.push(KpopScheduleStep::KpopOnce);
        }
    }
    out
}

#[must_use]
pub fn render_planned_schedule_lines(schedule: &[KpopScheduleStep]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for (idx, step) in schedule.iter().enumerate() {
        let n = idx + 1;
        match step {
            KpopScheduleStep::KpopOnce => lines.push(format!("{n}. {KPOP_ONCE_LINE}")),
            KpopScheduleStep::Mbc2ThenFalsify => {
                lines.push(format!("{n}a. {MBC2_GENERATE_LINE}"));
                lines.push(format!("{n}b. {MBC2_FALSIFY_LINE}"));
            }
        }
    }
    lines.join("\n")
}

#[must_use]
pub fn build_scheduled_kpop_prompt(
    kpop_definition: &str,
    mbc2_definition: &str,
    user_request: &str,
    schedule: &[KpopScheduleStep],
) -> String {
    let kdef = kpop_definition.trim_end();
    let ureq = user_request.trim_end();
    let sched_text = render_planned_schedule_lines(schedule);
    let with_mbc2 = schedule_requires_mbc2(schedule);
    if with_mbc2 {
        let mdef = mbc2_definition.trim();
        format!(
            "Define KPOP:\n{kdef}\n\n---\n\nDefine MBC2:\n{mdef}\n\n---\n\nUser request:\n{ureq}\n\nPlanned schedule:\n{sched_text}\n\n{EXECUTION_RULES}"
        )
    } else {
        format!(
            "Define KPOP:\n{kdef}\n\n---\n\nUser request:\n{ureq}\n\nPlanned schedule:\n{sched_text}\n\n{EXECUTION_RULES}"
        )
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::{
        KpopScheduleStep, build_scheduled_kpop_prompt, generate_kpop_schedule,
        render_planned_schedule_lines, schedule_requires_mbc2,
    };

    #[test]
    fn p_creative_zero_is_all_kpop_once() {
        let mut rng = StdRng::seed_from_u64(42);
        let s = generate_kpop_schedule(20, 0.0, &mut rng);
        assert!(s.iter().all(|x| matches!(x, KpopScheduleStep::KpopOnce)));
        assert!(!schedule_requires_mbc2(&s));
    }

    #[test]
    fn p_creative_one_is_all_mbc2_pairs() {
        let mut rng = StdRng::seed_from_u64(99);
        let s = generate_kpop_schedule(15, 1.0, &mut rng);
        assert!(s
            .iter()
            .all(|x| matches!(x, KpopScheduleStep::Mbc2ThenFalsify)));
        assert!(schedule_requires_mbc2(&s));
    }

    #[test]
    fn seeded_schedule_is_stable() {
        let mut a = StdRng::seed_from_u64(12345);
        let mut b = StdRng::seed_from_u64(12345);
        let sa = generate_kpop_schedule(30, 0.3, &mut a);
        let sb = generate_kpop_schedule(30, 0.3, &mut b);
        assert_eq!(sa, sb);
    }

    #[test]
    fn render_kpop_only_and_mbc2_pair() {
        let plain = [KpopScheduleStep::KpopOnce, KpopScheduleStep::KpopOnce];
        let t = render_planned_schedule_lines(&plain);
        assert!(t.contains("1. KPOP:"));
        assert!(t.contains("2. KPOP:"));
        assert!(!t.contains("1a."));

        let creative = [KpopScheduleStep::Mbc2ThenFalsify];
        let t2 = render_planned_schedule_lines(&creative);
        assert!(t2.contains("1a. MBC2:"));
        assert!(t2.contains("1b. KPOP:"));
    }

    #[test]
    fn prompt_orders_sections_with_and_without_mbc2() {
        let sched = [KpopScheduleStep::KpopOnce];
        let p = build_scheduled_kpop_prompt("KD", "", "REQ", &sched);
        assert!(p.starts_with("Define KPOP:\nKD"));
        let u = p.find("User request:").expect("user");
        let pl = p.find("Planned schedule:").expect("plan");
        assert!(u < pl);
        assert!(!p.contains("Define MBC2:"));

        let sched_m = [KpopScheduleStep::Mbc2ThenFalsify];
        let p2 = build_scheduled_kpop_prompt("KD", "MD", "REQ", &sched_m);
        assert!(p2.contains("Define MBC2:\nMD"));
        let d_mbc2 = p2.find("Define MBC2:").expect("mbc2");
        let u2 = p2.find("User request:").expect("user2");
        assert!(d_mbc2 < u2);
    }

    #[test]
    fn nonfinite_p_creative_yields_no_mbc2_steps() {
        let mut rng = StdRng::seed_from_u64(1);
        let s = generate_kpop_schedule(10, f64::NAN, &mut rng);
        assert!(s.iter().all(|x| matches!(x, KpopScheduleStep::KpopOnce)));
    }

    fn mbc2_dup_test_store(
        tmp: &std::path::Path,
    ) -> (
        crate::prompts::PromptStore,
        std::collections::HashMap<String, String>,
    ) {
        std::fs::write(tmp.join("header.md"), "hdr").unwrap();
        std::fs::write(tmp.join("coding_rules.md"), "CR_UNIQUE_MARKER_X").unwrap();
        std::fs::write(tmp.join("kpop.md"), "kline").unwrap();
        std::fs::write(tmp.join("mbc2.md"), "{{ coding_rules }}\nmbc2tail").unwrap();
        (
            crate::prompts::PromptStore::with_root(tmp.to_path_buf()),
            std::collections::HashMap::new(),
        )
    }

    #[test]
    fn scheduled_kpop_mbc2_body_does_not_duplicate_merged_coding_rules() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, context) = mbc2_dup_test_store(tmp.path());
        let rules = crate::prompts::merged_coding_rules(&store, &context);
        let kpop_core = store
            .render_prompt_only("kpop.md", &context)
            .expect("kp");
        let kpop_body = format!("{}\n\n{}", rules.trim_end(), kpop_core.trim_end());
        let mbc2_body = crate::prompts::render_mbc2_for_scheduled_kpop_block(&store, &context)
            .expect("mb");
        let combined = build_scheduled_kpop_prompt(
            &kpop_body,
            &mbc2_body,
            "u",
            &[KpopScheduleStep::Mbc2ThenFalsify],
        );
        assert_eq!(combined.matches("CR_UNIQUE_MARKER_X").count(), 1);
    }
}

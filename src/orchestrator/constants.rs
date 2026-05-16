pub const REVIEWER_FANOUT_CONCURRENCY: usize = 3;

pub const REVIEW_DESCRIPTIONS_FILE: &str = "review_descriptions.md";
pub const REVIEWER_TEMPLATE_FILE: &str = "reviewer_template.md";
pub const REVIEW_WRITE_FILE: &str = "review_write.md";

#[must_use]
pub const fn fanout_wave_count(job_count: usize) -> usize {
    if job_count == 0 {
        0
    } else {
        job_count.div_ceil(REVIEWER_FANOUT_CONCURRENCY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fanout_wave_count_is_ceil_division_by_concurrency() {
        assert_eq!(fanout_wave_count(0), 0);
        assert_eq!(fanout_wave_count(1), 1);
        assert_eq!(fanout_wave_count(3), 1);
        assert_eq!(fanout_wave_count(4), 2);
        let n = super::super::review_fanout_desc::embedded_review_description_job_count();
        assert_eq!(fanout_wave_count(n), n.div_ceil(REVIEWER_FANOUT_CONCURRENCY));
    }
}

use malvin::kpop_multiturn_prompts::KpopMultiturnPrompts;

pub const MBC2_SEEK_MAX_STEPS: usize = 10_000;

pub struct MtStubPrompts;

impl KpopMultiturnPrompts for MtStubPrompts {
    fn kpop_block(&mut self, want: usize, _: usize) -> Result<String, String> {
        Ok(format!("stub kpop want={want}"))
    }

    fn mbc2_pure(&mut self) -> Result<String, String> {
        Ok("stub mbc2".into())
    }
}

pub fn parse_kpop_want(prompt: &str) -> Option<usize> {
    prompt
        .trim()
        .strip_prefix("stub kpop want=")
        .and_then(|s| s.parse().ok())
}

pub fn append_kpop_line(path: &std::path::Path, step: usize) {
    let line = format!("## Step {step} — KPOP test\n");
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| {
            use std::io::Write;
            f.write_all(line.as_bytes())
        })
        .expect("append kpop");
}

pub fn append_mbc2_line(path: &std::path::Path, step: usize) {
    let line = format!("## Step {step} — MBC2 test\n");
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| {
            use std::io::Write;
            f.write_all(line.as_bytes())
        })
        .expect("append mbc2");
}

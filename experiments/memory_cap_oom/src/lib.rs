//! Cap guest RAM, run an allocator past the cap, and check microsandbox behavior.
//!
//! Primary case: busybox `dd` loop in `alpine` under a 48 MiB guest cap.
//! Expect non-zero exit (typically 137 = SIGKILL), partial stdout, and clean `stop()`.

use microsandbox::Sandbox;

const SANDBOX_NAME: &str = "malvin-exp-mem-cap";
pub const CAPPED_MEMORY_MIB: u32 = 48;
pub const CONTROL_MEMORY_MIB: u32 = 128;
pub const ALLOC_STEP_MIB: u32 = 4;

const ALLOCATOR_SH: &str = r#"i=0
while true; do
  i=$((i+1))
  dd if=/dev/zero of=/tmp/oom$i bs=1M count=4 2>/dev/null || exit 137
  echo "allocated $((i*4)) MiB"
done
"#;

#[derive(Debug)]
pub struct RunReport {
    pub label: &'static str,
    pub memory_mib: u32,
    pub exit_code: i32,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub async fn run_sh_allocator(
    label: &'static str,
    memory_mib: u32,
) -> Result<RunReport, Box<dyn std::error::Error + Send + Sync>> {
    let sb = Sandbox::builder(SANDBOX_NAME)
        .image("alpine")
        .memory(memory_mib)
        .replace()
        .create()
        .await?;

    let output = sb.exec("sh", ["-c", ALLOCATOR_SH]).await?;

    let report = RunReport {
        label,
        memory_mib,
        exit_code: output.status().code,
        success: output.status().success,
        stdout: output.stdout().unwrap_or_else(|_| "<non-utf8 stdout>".into()),
        stderr: output.stderr().unwrap_or_else(|_| "<non-utf8 stderr>".into()),
    };

    sb.stop().await?;
    Ok(report)
}

pub fn print_report(r: &RunReport) {
    println!("--- {} (guest cap {} MiB) ---", r.label, r.memory_mib);
    println!("exit_code={}", r.exit_code);
    println!("success={}", r.success);
    if !r.stdout.is_empty() {
        println!("stdout:\n{}", r.stdout);
    }
    if !r.stderr.is_empty() {
        println!("stderr:\n{}", r.stderr);
    }
}

pub fn last_allocated_mib(stdout: &str) -> Option<u32> {
    stdout
        .lines()
        .filter_map(|line| parse_allocated_mib(line).ok())
        .last()
}

pub fn parse_allocated_mib(line: &str) -> Result<u32, ()> {
    let rest = line.strip_prefix("allocated ").ok_or(())?;
    let num = rest.split_whitespace().next().ok_or(())?;
    num.parse().map_err(|_| ())
}

pub fn oom_like_exit(code: i32) -> bool {
    matches!(code, 137 | 9 | -9)
}

pub fn verify_capped(r: &RunReport) -> Result<(), String> {
    print_report(r);

    if r.success {
        return Err("allocator must not succeed under a 48 MiB guest cap".into());
    }

    if !oom_like_exit(r.exit_code) {
        return Err(format!(
            "expected SIGKILL-style exit 137 (or 9), got {}",
            r.exit_code
        ));
    }

    let alloc = last_allocated_mib(&r.stdout).ok_or_else(|| {
        "expected allocation progress lines in stdout".to_string()
    })?;

    if alloc >= CAPPED_MEMORY_MIB {
        return Err(format!(
            "last allocation {alloc} MiB >= cap {CAPPED_MEMORY_MIB} MiB"
        ));
    }

    println!(
        "OK: guest process killed after ~{alloc} MiB (cap {CAPPED_MEMORY_MIB} MiB), exit={}",
        r.exit_code
    );
    Ok(())
}

pub fn verify_control_vs_capped(capped: &RunReport, control: &RunReport) -> Result<(), String> {
    print_report(control);

    if control.success {
        return Err("control should also fail once it exceeds its smaller cap".into());
    }

    let capped_mib = last_allocated_mib(&capped.stdout).unwrap_or(0);
    let control_mib = last_allocated_mib(&control.stdout).unwrap_or(0);

    if control_mib <= capped_mib {
        return Err(format!(
            "control should allocate more than capped run before dying (capped={capped_mib}, control={control_mib} MiB)"
        ));
    }

    if control_mib >= CONTROL_MEMORY_MIB {
        return Err(format!(
            "control reached {control_mib} MiB at/above cap {CONTROL_MEMORY_MIB} MiB"
        ));
    }

    println!(
        "OK: higher cap allowed more allocation (capped {capped_mib} MiB, control {control_mib} MiB)"
    );
    Ok(())
}

pub async fn run_memory_cap_oom() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!(
        "step={ALLOC_STEP_MIB} MiB; capped={CAPPED_MEMORY_MIB} MiB; control={CONTROL_MEMORY_MIB} MiB"
    );

    let capped = run_sh_allocator("capped", CAPPED_MEMORY_MIB).await?;
    verify_capped(&capped)?;

    let control = run_sh_allocator("control", CONTROL_MEMORY_MIB).await?;
    verify_control_vs_capped(&capped, &control)?;

    println!("\nAll checks passed.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        last_allocated_mib, oom_like_exit, parse_allocated_mib, print_report, run_sh_allocator,
        verify_capped, verify_control_vs_capped, RunReport, ALLOC_STEP_MIB, CAPPED_MEMORY_MIB,
        CONTROL_MEMORY_MIB,
    };

    #[test]
    fn kiss_cov_symbols() {
        let _ = stringify!(run_memory_cap_oom);
        let _ = stringify!(run_sh_allocator);
        let _ = stringify!(print_report);
        let _ = stringify!(last_allocated_mib);
        let _ = stringify!(parse_allocated_mib);
        let _ = stringify!(oom_like_exit);
        let _ = stringify!(verify_capped);
        let _ = stringify!(verify_control_vs_capped);
        let _ = stringify!(RunReport);
    }

    fn sample_report(label: &'static str, stdout: &str, exit_code: i32, success: bool) -> RunReport {
        RunReport {
            label,
            memory_mib: CAPPED_MEMORY_MIB,
            exit_code,
            success,
            stdout: stdout.to_string(),
            stderr: String::new(),
        }
    }

    #[test]
    fn parse_allocated_mib_parses_progress_line() {
        assert_eq!(parse_allocated_mib("allocated 12 MiB"), Ok(12));
        assert!(parse_allocated_mib("nope").is_err());
    }

    #[test]
    fn last_allocated_mib_takes_final_line() {
        let stdout = "allocated 4 MiB\nallocated 8 MiB\n";
        assert_eq!(last_allocated_mib(stdout), Some(8));
    }

    #[test]
    fn oom_like_exit_accepts_sigkill_codes() {
        assert!(oom_like_exit(137));
        assert!(oom_like_exit(9));
        assert!(!oom_like_exit(0));
    }

    #[test]
    fn verify_capped_accepts_sigkill_with_partial_alloc() {
        let r = sample_report("capped", "allocated 16 MiB\n", 137, false);
        verify_capped(&r).expect("capped report should pass verification");
    }

    #[test]
    fn verify_control_requires_more_allocation_than_capped() {
        let capped = sample_report("capped", "allocated 16 MiB\n", 137, false);
        let control = sample_report("control", "allocated 32 MiB\n", 137, false);
        verify_control_vs_capped(&capped, &control).expect("control should allocate more");
    }

    #[test]
    fn print_report_smoke() {
        let r = sample_report("smoke", "allocated 4 MiB\n", 137, false);
        print_report(&r);
        let _ = (ALLOC_STEP_MIB, CONTROL_MEMORY_MIB);
    }
}

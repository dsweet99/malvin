use super::Commands;

pub fn ensure_malvin_checks_for_command(cmd: &Commands) -> Result<(), String> {
    match cmd {
        Commands::Do(_) | Commands::Models(_) => Ok(()),
        _ => {
            let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
            crate::repo_gates::ensure_default_malvin_checks_file(&cwd)?;
            crate::repo_gates::ensure_default_malvin_config_file(&cwd)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ensure_malvin_checks_for_command;
    use crate::cli::args::{DoArgs, ModelsArgs};
    use crate::cli::{CodeArgs, Commands};

    #[test]
    fn ensure_malvin_checks_for_command_writes_defaults_except_do_and_models() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let checks = tmp.path().join(".malvin/checks");
        let config = tmp.path().join(".malvin/config.toml");
        assert!(!checks.exists());
        assert!(!config.exists());

        ensure_malvin_checks_for_command(&Commands::Code(CodeArgs {
            max_loops: 1,
            max_hypotheses: 10,
            tenacious: false,
            trust_the_plan: false,
            dry_run: false,
            skip_pre_checks: false,
            fast: false,
            request: None,
        }))
        .expect("code should materialize checks");
        assert!(checks.is_file());
        assert!(config.is_file());
        assert!(
            std::fs::read_to_string(&config)
                .expect("read config")
                .contains("[agent]")
        );

        std::fs::remove_file(&checks).expect("remove checks");
        std::fs::remove_file(&config).expect("remove config");
        ensure_malvin_checks_for_command(&Commands::Do(DoArgs {
            repo_gates: false,
            thoughts: false,
            request: None,
        }))
        .expect("do must not create checks");
        assert!(!checks.exists());
        assert!(!config.exists());

        ensure_malvin_checks_for_command(&Commands::Models(ModelsArgs {}))
            .expect("models must not create checks");
        assert!(!checks.exists());
        assert!(!config.exists());

        std::env::set_current_dir(cwd).expect("restore cwd");
    }
}

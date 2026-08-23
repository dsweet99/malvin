use super::Commands;

pub const fn ensure_malvin_checks_for_command(_cmd: &Commands) {}

pub fn ensure_malvin_checks_for_gates_only_route() -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    crate::repo_gates::ensure_default_malvin_config_file(&cwd)
}

pub fn ensure_malvin_checks_for_do_workflow() -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    crate::repo_gates::ensure_default_malvin_config_file(&cwd)
}

pub fn ensure_malvin_checks_for_default_route() -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    crate::repo_gates::ensure_default_malvin_config_file(&cwd)
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_malvin_checks_for_command, ensure_malvin_checks_for_do_workflow,
        ensure_malvin_checks_for_gates_only_route,
    };
    use crate::cli::Commands;
    use crate::cli::args::ModelsArgs;

    #[test]
    fn ensure_malvin_checks_for_command_writes_config_not_checks() {
        crate::test_utils::with_isolated_home(|work| {
            assert!(
                std::process::Command::new("git")
                    .args(["init"])
                    .current_dir(work)
                    .status()
                    .expect("git init")
                    .success()
            );
            let cwd = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(work).expect("chdir");
            let checks = crate::malvin_checks_path(work);
            let config = crate::malvin_config_path(work);
            assert!(!checks.exists());
            assert!(!config.exists());

            ensure_malvin_checks_for_gates_only_route().expect("gates-only route should materialize config only");
            assert!(!checks.is_file());
            assert!(config.is_file());
            assert!(
                std::fs::read_to_string(&config)
                    .expect("read config")
                    .contains("[agent]")
            );

            std::fs::remove_file(&config).expect("remove config");
            ensure_malvin_checks_for_do_workflow().expect("do must materialize home config");
            assert!(!checks.exists());
            assert!(config.is_file());
            assert!(
                std::fs::read_to_string(&config)
                    .expect("read config")
                    .contains("[agent]")
            );

            std::fs::remove_file(&config).expect("remove config for models test");

            ensure_malvin_checks_for_command(&Commands::Models(ModelsArgs::default()));
            assert!(!checks.exists());
            assert!(!config.exists());

            std::env::set_current_dir(cwd).expect("restore cwd");
        });
    }
}

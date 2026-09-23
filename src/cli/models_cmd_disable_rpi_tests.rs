use super::models_cmd::{ModelsArgs, run_models, test_hooks};
use malvin::output::{enable_stdout_capture, take_captured_stdout};
use malvin::test_utils::with_isolated_home;
use malvin::workspace_paths::malvin_config_path;

#[test]
fn run_models_omits_rpi_when_disable_rpi() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(
            &path,
            "disable_rpi = true\n[agent]\nmodel = \"cursor:auto\"\n",
        )
        .expect("write");
        // Avoid catalog refresh (can hang without network) in this unit test.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        test_hooks::save_last_refresh_secs(now).expect("stamp refresh");
        enable_stdout_capture();
        run_models(
            ModelsArgs {
                refresh: false,
                words: vec!["rpi:".into()],
            },
            malvin::config::DEFAULT_CLI_MODEL,
        )
        .expect("models");
        let out = take_captured_stdout();
        assert!(
            !out.contains("rpi:"),
            "disable_rpi must hide rpi listings: {out}"
        );
        assert!(out.contains("Current:"), "footer remains: {out}");
    });
}

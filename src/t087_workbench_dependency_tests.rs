use crate::workbench::{InertWorkbench, backend_contract_name, event_contract_name};

const CARGO_TOML: &str = include_str!("../Cargo.toml");
const CARGO_LOCK: &str = include_str!("../Cargo.lock");
const WORKBENCH_SOURCE: &str = include_str!("workbench.rs");

#[test]
fn t087_inert_workbench_render_is_deterministic_and_non_authoritative() {
    let first = InertWorkbench::new(32, 5).render();
    let second = InertWorkbench::new(32, 5).render();

    assert_eq!(first, second);
    assert_eq!(first.area, InertWorkbench::new(32, 5).area());
    assert_eq!(first[(0, 0)].symbol(), "┌");
    assert_eq!(first[(31, 0)].symbol(), "┐");
    assert_eq!(first[(0, 4)].symbol(), "└");
    assert_eq!(first[(31, 4)].symbol(), "┘");

    assert!(backend_contract_name().contains("CrosstermBackend"));
    assert!(event_contract_name().ends_with("crossterm::event::Event"));

    for forbidden in [
        "portable_pty",
        "TerminalSession",
        "std::process",
        "Command::new",
        "event::read",
        "enable_raw_mode",
        "disable_raw_mode",
        "tokio",
    ] {
        assert!(
            !WORKBENCH_SOURCE.contains(forbidden),
            "T087 inert workbench unexpectedly contains forbidden runtime surface: {forbidden}"
        );
    }
}

#[test]
fn t087_dependency_features_are_exact_and_fail_closed() {
    assert!(CARGO_TOML.contains(
        "crossterm = { version = \"=0.29.0\", default-features = false, features = [\"bracketed-paste\", \"events\", \"windows\", \"derive-more\"] }"
    ));
    assert!(CARGO_TOML.contains(
        "ratatui = { version = \"=0.30.2\", default-features = false, features = [\"crossterm\"] }"
    ));

    assert!(!CARGO_TOML.contains("osc52"));
    assert!(!CARGO_TOML.contains("event-stream"));
    assert!(!CARGO_TOML.contains("ratatui-textarea"));
    assert!(!CARGO_TOML.contains("vt100"));

    for required in [
        "name = \"crossterm\"\nversion = \"0.29.0\"",
        "name = \"ratatui\"\nversion = \"0.30.2\"",
        "name = \"ratatui-core\"\nversion = \"0.1.2\"",
        "name = \"ratatui-crossterm\"\nversion = \"0.1.2\"",
        "name = \"ratatui-widgets\"\nversion = \"0.3.2\"",
    ] {
        assert!(CARGO_LOCK.contains(required), "missing lock entry: {required}");
    }

    for forbidden in [
        "name = \"base64\"",
        "name = \"futures-core\"",
        "name = \"tokio\"",
        "name = \"ratatui-macros\"",
        "name = \"time\"",
    ] {
        assert!(
            !CARGO_LOCK.contains(forbidden),
            "T087 resolved graph unexpectedly contains forbidden package: {forbidden}"
        );
    }
}

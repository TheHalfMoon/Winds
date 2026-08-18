use std::fs;

fn workflow(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn assert_exact_head_contract(path: &str, contents: &str) {
    assert!(
        contents.contains("github.event.pull_request.head.sha || github.sha"),
        "{path} must derive candidate identity from the pull-request head SHA"
    );
    assert!(
        contents.contains("Verify checkout identity"),
        "{path} must fail closed if checkout identity differs from the candidate SHA"
    );
    assert!(
        contents.contains("git rev-parse HEAD"),
        "{path} must verify the checked-out Git commit"
    );
}

#[test]
fn t068_ci_workflows_bind_evidence_to_exact_candidate_head() {
    for path in [
        ".github/workflows/quality.yml",
        ".github/workflows/windows-terminal.yml",
        ".github/workflows/release-candidate.yml",
    ] {
        let contents = workflow(path);
        assert_exact_head_contract(path, &contents);
    }
}

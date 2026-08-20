use std::fs;

fn workflow(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn assert_exact_head_contract(path: &str, contents: &str) {
    let candidate_line = contents
        .lines()
        .find(|line| line.trim_start().starts_with("CANDIDATE_SHA:"))
        .unwrap_or_else(|| panic!("{path} must define CANDIDATE_SHA"));
    let pull_head = "github.event.pull_request.head.sha";
    let pull_head_index = candidate_line.find(pull_head).unwrap_or_else(|| {
        panic!("{path} must derive candidate identity from the pull-request head SHA")
    });
    let github_sha_index = candidate_line
        .find("github.sha")
        .unwrap_or_else(|| panic!("{path} must retain a non-PR SHA fallback"));
    assert!(
        pull_head_index < github_sha_index,
        "{path} must prefer the pull-request head SHA over fallback candidate identity"
    );
    assert!(
        contents.contains("ref: ${{ env.CANDIDATE_SHA }}"),
        "{path} must checkout the exact candidate SHA rather than a mutable branch/ref"
    );
    assert!(
        contents.contains("Verify checkout identity"),
        "{path} must fail closed if checkout identity differs from the candidate SHA"
    );
    assert!(
        contents.contains("test \"$(git rev-parse HEAD)\" = \"$CANDIDATE_SHA\"")
            || (contents.contains("$actual = (git rev-parse HEAD).Trim()")
                && contents.contains("$actual -cne $env:CANDIDATE_SHA")),
        "{path} must compare the actual checked-out Git commit to the exact candidate SHA"
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

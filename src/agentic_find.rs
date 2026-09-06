use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum FindProvenance {
    ExactName,
    PrefixName,
    SubstringName,
    OrderedSubsequence,
    Changed,
    Recent,
    TestDerived,
    SymbolDerived,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SessionFindInput {
    pub session_id: String,
    pub display_name: String,
    pub changed: bool,
    pub recent: bool,
    pub test_derived: bool,
    pub symbol_derived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SessionFindCandidate {
    pub session_id: String,
    pub display_name: String,
    pub score: u8,
    pub provenance: Vec<FindProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SessionSelection {
    None,
    Unique(SessionFindCandidate),
    Ambiguous(Vec<SessionFindCandidate>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SemanticAvailability {
    Unavailable,
}

pub(crate) fn select_session(query: &str, sessions: &[SessionFindInput]) -> SessionSelection {
    let candidates = rank_sessions(query, sessions);
    match candidates.len() {
        0 => SessionSelection::None,
        1 => SessionSelection::Unique(candidates.into_iter().next().expect("one candidate")),
        _ => SessionSelection::Ambiguous(candidates),
    }
}

pub(crate) fn rank_sessions(
    query: &str,
    sessions: &[SessionFindInput],
) -> Vec<SessionFindCandidate> {
    let normalized_query = normalize(query);
    if normalized_query.is_empty() {
        return Vec::new();
    }

    let mut candidates: Vec<_> = sessions
        .iter()
        .filter_map(|session| {
            let normalized_name = normalize(&session.display_name);
            let normalized_id = normalize(&session.session_id);
            let (score, mut provenance) =
                name_score(&normalized_query, &normalized_name, &normalized_id)?;
            if session.changed {
                provenance.push(FindProvenance::Changed);
            }
            if session.recent {
                provenance.push(FindProvenance::Recent);
            }
            if session.test_derived {
                provenance.push(FindProvenance::TestDerived);
            }
            if session.symbol_derived {
                provenance.push(FindProvenance::SymbolDerived);
            }
            provenance.sort();
            provenance.dedup();
            Some(SessionFindCandidate {
                session_id: session.session_id.clone(),
                display_name: session.display_name.clone(),
                score,
                provenance,
            })
        })
        .collect();

    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| normalize(&left.display_name).cmp(&normalize(&right.display_name)))
            .then_with(|| left.session_id.cmp(&right.session_id))
    });
    candidates
}

pub(crate) fn canonical_existing_path(root: &Path, requested: &Path) -> Result<PathBuf, String> {
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("workspace root cannot be canonicalized: {error}"))?;
    let candidate = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        canonical_root.join(requested)
    };
    let canonical_candidate = candidate
        .canonicalize()
        .map_err(|error| format!("selected path cannot be canonicalized: {error}"))?;
    if !canonical_candidate.starts_with(&canonical_root) {
        return Err("selected path escapes the canonical workspace root".to_owned());
    }
    Ok(canonical_candidate)
}

pub(crate) fn semantic_availability() -> SemanticAvailability {
    SemanticAvailability::Unavailable
}

fn name_score(query: &str, name: &str, session_id: &str) -> Option<(u8, Vec<FindProvenance>)> {
    if name == query || session_id == query {
        return Some((100, vec![FindProvenance::ExactName]));
    }
    if name.starts_with(query) || session_id.starts_with(query) {
        return Some((80, vec![FindProvenance::PrefixName]));
    }
    if name.contains(query) || session_id.contains(query) {
        return Some((60, vec![FindProvenance::SubstringName]));
    }
    if ordered_subsequence(query, name) || ordered_subsequence(query, session_id) {
        return Some((40, vec![FindProvenance::OrderedSubsequence]));
    }
    None
}

fn ordered_subsequence(query: &str, candidate: &str) -> bool {
    let mut candidate_chars = candidate.chars();
    for query_char in query.chars() {
        if !candidate_chars.any(|candidate_char| candidate_char == query_char) {
            return false;
        }
    }
    true
}

fn normalize(value: &str) -> String {
    value.chars().flat_map(char::to_lowercase).collect()
}

#[cfg(test)]
#[path = "t084_agentic_findability_tests.rs"]
mod tests;

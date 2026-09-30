#![forbid(unsafe_code)]

//! Compile-time, data-only agent detection catalog for the 24 families Spec 012
//! pins as ledger rows A01-A24.
//!
//! The catalog is the whole authority surface: static borrowed data plus pure
//! matching. It holds no executable it can act on, no configuration it can
//! write, and no provider session it can attach to. Nothing here executes,
//! reloads, or refreshes anything, and the only way to change what it matches
//! is to build a different value in Rust. Terminal prose, user labels, and shell
//! titles are not detection inputs at all, so they cannot promote a
//! classification.
//!
//! Catalog presence is detection-only support. No family in this file has an
//! admitted execution seam, so the support state of every family is
//! [`AgentSupport::DetectionOnly`] and there is deliberately no variant that
//! could express launch, install, prompt, or execution authority.
//!
//! # What the pinned catalog asserts
//!
//! Each family records one accepted structured-metadata namespace token. The
//! token is the exact per-family detector-manifest stem that the Spec 012
//! capability ledger pins in the source-evidence column of rows A01-A24, for
//! example `github-copilot` for `GithubCopilot` and `qodercli` for `Qodercli`.
//! The namespace is a Winds-owned classification token, not a vendor
//! filesystem layout claim.
//!
//! # What the pinned catalog deliberately does not assert
//!
//! - **No executable basename.** The repository holds no accepted evidence of
//!   any vendor executable basename for these families, so the pinned catalog
//!   records none. Asserting an unsubstantiated basename would let Winds report
//!   a false family for an unrelated program that happens to share a name,
//!   which FR-033 forbids. [`AgentCatalogEntry::executable_basenames`] is empty
//!   on every pinned entry; adding one is a data-only change that requires the
//!   exact basename to be independently substantiated first.
//! - **No platform restriction.** Winds holds no accepted evidence that any of
//!   the 24 families is unavailable on Linux, macOS, native Windows, or WSL, so
//!   every pinned entry records all four. A restriction that cannot be
//!   substantiated would manufacture a false `Unavailable` classification.
//!   [`AgentCatalogEntry::platforms`] still exists so that a genuine restriction
//!   fails closed rather than silently dropping a family when one is accepted.
//!
//! Both nonclaims are enforced by focused tests, so neither can be dropped by
//! accident.

/// The exact pinned family set, in Spec 012 ledger row order A01-A24.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum AgentFamily {
    Pi,
    Claude,
    Codex,
    Gemini,
    Cursor,
    Devin,
    Antigravity,
    Cline,
    Omp,
    Mastracode,
    OpenCode,
    GithubCopilot,
    Kimi,
    Kiro,
    Droid,
    Amp,
    Grok,
    Hermes,
    Kilo,
    Qodercli,
    Qwen,
    Letta,
    Maki,
    Muse,
}

impl AgentFamily {
    /// Every pinned family, in pinned order.
    pub(crate) const ALL: [AgentFamily; 24] = [
        AgentFamily::Pi,
        AgentFamily::Claude,
        AgentFamily::Codex,
        AgentFamily::Gemini,
        AgentFamily::Cursor,
        AgentFamily::Devin,
        AgentFamily::Antigravity,
        AgentFamily::Cline,
        AgentFamily::Omp,
        AgentFamily::Mastracode,
        AgentFamily::OpenCode,
        AgentFamily::GithubCopilot,
        AgentFamily::Kimi,
        AgentFamily::Kiro,
        AgentFamily::Droid,
        AgentFamily::Amp,
        AgentFamily::Grok,
        AgentFamily::Hermes,
        AgentFamily::Kilo,
        AgentFamily::Qodercli,
        AgentFamily::Qwen,
        AgentFamily::Letta,
        AgentFamily::Maki,
        AgentFamily::Muse,
    ];

    /// The exact pinned family name, for presentation and evidence.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            AgentFamily::Pi => "Pi",
            AgentFamily::Claude => "Claude",
            AgentFamily::Codex => "Codex",
            AgentFamily::Gemini => "Gemini",
            AgentFamily::Cursor => "Cursor",
            AgentFamily::Devin => "Devin",
            AgentFamily::Antigravity => "Antigravity",
            AgentFamily::Cline => "Cline",
            AgentFamily::Omp => "Omp",
            AgentFamily::Mastracode => "Mastracode",
            AgentFamily::OpenCode => "OpenCode",
            AgentFamily::GithubCopilot => "GithubCopilot",
            AgentFamily::Kimi => "Kimi",
            AgentFamily::Kiro => "Kiro",
            AgentFamily::Droid => "Droid",
            AgentFamily::Amp => "Amp",
            AgentFamily::Grok => "Grok",
            AgentFamily::Hermes => "Hermes",
            AgentFamily::Kilo => "Kilo",
            AgentFamily::Qodercli => "Qodercli",
            AgentFamily::Qwen => "Qwen",
            AgentFamily::Letta => "Letta",
            AgentFamily::Maki => "Maki",
            AgentFamily::Muse => "Muse",
        }
    }
}

/// Detection support is the only support state this catalog can express. There
/// is no admitted execution seam in this program, so a family can never be
/// reported as launchable, installable, or promptable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentSupport {
    DetectionOnly,
}

impl AgentSupport {
    /// The reported support token. This match is exhaustive, so admitting a
    /// second support state is a build error until a separately governed
    /// authority decision says how that state must be reported.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            AgentSupport::DetectionOnly => "DETECTION_ONLY",
        }
    }

    /// The honest limitation the UI must state: detection is not execution.
    pub(crate) const fn truth_statement(self) -> &'static str {
        "Detection only. Catalog presence proves an accepted structured-name \
         match only, never provider execution, session attachment, or launch, \
         install, or prompt authority."
    }
}

/// Host platforms the catalog records applicability for. Applicability only
/// narrows which families are plausible on a host; it never asserts that a
/// family is installed, running, or executable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CatalogPlatform {
    Windows,
    MacOs,
    Linux,
    Wsl,
}

impl CatalogPlatform {
    pub(crate) const ALL: [CatalogPlatform; 4] = [
        CatalogPlatform::Windows,
        CatalogPlatform::MacOs,
        CatalogPlatform::Linux,
        CatalogPlatform::Wsl,
    ];
}

/// Structured signal classes the catalog matches. Both are structured inputs
/// produced by a host process or runtime boundary, never free-form display text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DetectionSource {
    /// An exact agent-native configuration namespace component.
    StructuredMetadataNamespace,
    /// The exact, normalized file name of a launched executable.
    ExecutableFileName,
}

/// One family in the catalog. Every field is borrowed `'static` data, so an
/// accepted entry can never change under a reader that already holds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AgentCatalogEntry {
    pub(crate) family: AgentFamily,
    pub(crate) support: AgentSupport,
    /// The exact accepted structured-metadata namespace token for the family.
    pub(crate) namespace: &'static str,
    /// Accepted executable basenames. Empty on every pinned entry because no
    /// vendor basename is substantiated; see the module nonclaims.
    pub(crate) executable_basenames: &'static [&'static str],
    /// The platforms the family is recorded as applicable to.
    pub(crate) platforms: &'static [CatalogPlatform],
}

impl AgentCatalogEntry {
    /// Whether this entry claims `normalized` for `source`.
    fn claims(&self, source: DetectionSource, normalized: &str) -> bool {
        match source {
            DetectionSource::StructuredMetadataNamespace => self.namespace == normalized,
            DetectionSource::ExecutableFileName => self.executable_basenames.contains(&normalized),
        }
    }

    /// Whether `normalized` is a proper prefix, in either direction, of a token
    /// this entry claims for `source`. A near match carries no family, so it can
    /// never be mistaken for an observation.
    fn is_near(&self, source: DetectionSource, normalized: &str) -> bool {
        match source {
            DetectionSource::StructuredMetadataNamespace => {
                is_near_token(self.namespace, normalized)
            }
            DetectionSource::ExecutableFileName => self
                .executable_basenames
                .iter()
                .any(|basename| is_near_token(basename, normalized)),
        }
    }
}

/// One family's catalog data, validated before it may classify anything, so
/// malformed data never partially activates.
fn validate_entry(entry: &AgentCatalogEntry) -> Result<(), CatalogRuleError> {
    if entry.namespace.is_empty() {
        return Err(CatalogRuleError::EmptyNamespace);
    }
    if !is_lower_namespace(entry.namespace) {
        return Err(CatalogRuleError::MalformedNamespace);
    }
    if entry.platforms.is_empty() {
        return Err(CatalogRuleError::EmptyPlatformSet);
    }
    let mut index = 0;
    while index < entry.executable_basenames.len() {
        let basename = entry.executable_basenames[index];
        if !is_lower_basename(basename) {
            return Err(CatalogRuleError::MalformedExecutableBasename);
        }
        // Normalization removes these suffixes before matching, so a rule value
        // carrying one could never fire. Rejecting it keeps every accepted rule
        // reachable instead of silently dead.
        if EXECUTABLE_SUFFIXES
            .iter()
            .any(|suffix| basename.ends_with(suffix))
        {
            return Err(CatalogRuleError::MalformedExecutableBasename);
        }
        if entry.executable_basenames[..index].contains(&basename) {
            return Err(CatalogRuleError::DuplicateExecutableBasename);
        }
        index += 1;
    }
    Ok(())
}

/// The platform executable suffixes that normalization removes before matching.
/// Rule validation and normalization share this one list, so an accepted
/// basename is always a name the normalizer can actually produce.
const EXECUTABLE_SUFFIXES: [&str; 4] = [".exe", ".cmd", ".bat", ".ps1"];

/// Whether the token begins and ends with an alphanumeric character.
fn is_alphanumeric_ended(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes
        .first()
        .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes
            .last()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

/// A namespace is a lowercase ASCII token that may additionally contain `-`
/// inside it and may neither start nor end with one. This rejects `Pi`, `pi/`,
/// `pi-`, and `-pi`, so an accepted namespace is always a single exact word that
/// a normalization step cannot silently reshape.
fn is_lower_namespace(value: &str) -> bool {
    is_alphanumeric_ended(value)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

/// A basename is a lowercase ASCII token that may additionally contain `-`, `_`,
/// and `.` inside it and may neither start nor end with one. This rejects
/// `.claude`, `claude.`, `_claude`, and `Claude`, so a rule value can never be a
/// bare separator or carry a leading or trailing one.
fn is_lower_basename(value: &str) -> bool {
    is_alphanumeric_ended(value)
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.')
        })
}

fn is_near_token(accepted: &str, normalized: &str) -> bool {
    accepted != normalized && (normalized.starts_with(accepted) || accepted.starts_with(normalized))
}

const NO_BASENAMES: &[&str] = &[];
const ALL_HOSTS: &[CatalogPlatform] = &[
    CatalogPlatform::Windows,
    CatalogPlatform::MacOs,
    CatalogPlatform::Linux,
    CatalogPlatform::Wsl,
];

macro_rules! entry {
    ($family:ident, $namespace:literal) => {
        AgentCatalogEntry {
            family: AgentFamily::$family,
            support: AgentSupport::DetectionOnly,
            namespace: $namespace,
            executable_basenames: NO_BASENAMES,
            platforms: ALL_HOSTS,
        }
    };
}

/// The pinned compile-time catalog, one entry per required family.
///
/// The namespace of each entry is the exact detector-manifest stem pinned by
/// the corresponding Spec 012 ledger row A01-A24 in
/// `docs/research/021-herdr-exhaustive-capability-ledger.md`.
const PINNED_CATALOG: [AgentCatalogEntry; 24] = [
    entry!(Pi, "pi"),
    entry!(Claude, "claude"),
    entry!(Codex, "codex"),
    entry!(Gemini, "gemini"),
    entry!(Cursor, "cursor"),
    entry!(Devin, "devin"),
    entry!(Antigravity, "antigravity"),
    entry!(Cline, "cline"),
    entry!(Omp, "omp"),
    entry!(Mastracode, "mastracode"),
    entry!(OpenCode, "opencode"),
    entry!(GithubCopilot, "github-copilot"),
    entry!(Kimi, "kimi"),
    entry!(Kiro, "kiro"),
    entry!(Droid, "droid"),
    entry!(Amp, "amp"),
    entry!(Grok, "grok"),
    entry!(Hermes, "hermes"),
    entry!(Kilo, "kilo"),
    entry!(Qodercli, "qodercli"),
    entry!(Qwen, "qwen"),
    entry!(Letta, "letta"),
    entry!(Maki, "maki"),
    entry!(Muse, "muse"),
];

/// The catalog revision this build classified against. It is a compile-time
/// constant so a detection result is always attributable to exact catalog data.
const DETECTOR_REVISION: &str = "t172-pinned-1";

/// A borrowed, immutable view of a set of catalog entries.
///
/// The catalog is a value over borrowed data. There is no interior mutability,
/// no refresh, and no reload, so an accepted catalog cannot change under an
/// observer while it is being read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AgentCatalog {
    entries: &'static [AgentCatalogEntry],
}

impl AgentCatalog {
    /// The compile-time pinned catalog.
    pub(crate) const fn pinned() -> Self {
        Self {
            entries: &PINNED_CATALOG,
        }
    }

    /// The exact catalog revision a classification is attributable to.
    pub(crate) const fn revision(&self) -> &'static str {
        DETECTOR_REVISION
    }

    pub(crate) const fn entries(&self) -> &'static [AgentCatalogEntry] {
        self.entries
    }

    pub(crate) fn entry(&self, family: AgentFamily) -> Option<&'static AgentCatalogEntry> {
        self.entries.iter().find(|entry| entry.family == family)
    }

    /// Classifies one structured input on one host.
    pub(crate) fn detect(&self, input: DetectionInput, host: CatalogPlatform) -> AgentDetection {
        detect_in(self.entries, input, host)
    }

    /// Re-checks a classification that was previously reported as observed.
    ///
    /// T172 holds no live state, so it cannot tell a replaced pane from a live
    /// one, and it does not pretend to. What it can prove is the boundary T173
    /// depends on: a classification the current accepted catalog no longer
    /// reproduces for the same structured input is reported as
    /// [`AgentDetection::Stale`] and is never silently retargeted to whatever
    /// the input now matches.
    pub(crate) fn revalidate(
        &self,
        previous: &AgentDetection,
        input: DetectionInput,
        host: CatalogPlatform,
    ) -> AgentDetection {
        revalidate_in(self.entries, previous, input, host)
    }

    /// Every accepted token two or more families claim. The pinned catalog has
    /// none; the query exists so that adding one is a visible build-boundary
    /// failure rather than a silent ambiguity at detection time.
    pub(crate) fn cross_family_claims(&self) -> Vec<ClaimedToken> {
        cross_family_claims_in(self.entries)
    }

    /// Validates the whole catalog at the build/test boundary.
    pub(crate) fn validate(&self) -> Result<(), CatalogRuleError> {
        validate_entries(self.entries)
    }
}

/// Validates a catalog at the build/test boundary. Every required family must be
/// present exactly once, every rule must be well formed, and no token may be
/// claimed by more than one family.
///
/// This is a pure function of the borrowed entries, so a rejected catalog fails
/// closed on exactly the data it was handed. The checks run in a fixed order:
/// rule shape first, so a malformed rule is reported as malformed whatever the
/// catalog size, then family uniqueness, then catalog completeness, then
/// cross-family ambiguity.
pub(crate) fn validate_entries(entries: &[AgentCatalogEntry]) -> Result<(), CatalogRuleError> {
    for entry in entries {
        validate_entry(entry)?;
    }
    for (index, entry) in entries.iter().enumerate() {
        if entries[..index]
            .iter()
            .any(|earlier| earlier.family == entry.family)
        {
            return Err(CatalogRuleError::DuplicateFamily);
        }
    }
    // A catalog that does not carry exactly one entry per required family cannot
    // classify anything. This check also subsumes a missing family: any catalog
    // that omits one necessarily carries a different count, and any catalog with
    // the right count and a missing family necessarily carries a duplicate,
    // which the uniqueness check above has already rejected.
    if entries.len() != AgentFamily::ALL.len() {
        return Err(CatalogRuleError::FamilyCountMismatch);
    }
    if !cross_family_claims_in(entries).is_empty() {
        return Err(CatalogRuleError::CrossFamilyClaim);
    }
    Ok(())
}

/// An accepted token that more than one family claims.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClaimedToken {
    pub(crate) source: DetectionSource,
    pub(crate) token: &'static str,
    pub(crate) families: Vec<AgentFamily>,
}

pub(crate) fn cross_family_claims_in(entries: &[AgentCatalogEntry]) -> Vec<ClaimedToken> {
    let mut claims: Vec<ClaimedToken> = Vec::new();
    for entry in entries {
        for (source, token) in claimed_tokens(entry) {
            if let Some(existing) = claims
                .iter_mut()
                .find(|claim| claim.source == source && claim.token == token)
            {
                if !existing.families.contains(&entry.family) {
                    existing.families.push(entry.family);
                }
                continue;
            }
            claims.push(ClaimedToken {
                source,
                token,
                families: vec![entry.family],
            });
        }
    }
    claims
        .into_iter()
        .filter(|claim| claim.families.len() > 1)
        .collect()
}

fn claimed_tokens(entry: &AgentCatalogEntry) -> Vec<(DetectionSource, &'static str)> {
    let mut tokens = vec![(
        DetectionSource::StructuredMetadataNamespace,
        entry.namespace,
    )];
    for basename in entry.executable_basenames {
        tokens.push((DetectionSource::ExecutableFileName, *basename));
    }
    tokens
}

/// Classifies one structured input against one borrowed set of entries.
///
/// The classifier is a pure function of its arguments, so the same inputs always
/// produce the same classification and no refresh can change an answer. Every
/// loop is bounded by the entry count of the catalog it was handed, so the
/// classification cost is bounded by accepted compile-time data.
pub(crate) fn detect_in(
    entries: &[AgentCatalogEntry],
    input: DetectionInput,
    host: CatalogPlatform,
) -> AgentDetection {
    let source = match input {
        DetectionInput::StructuredMetadataNamespace { namespace } => {
            match normalize_namespace(namespace) {
                Some(normalized) => (DetectionSource::StructuredMetadataNamespace, normalized),
                None => {
                    return AgentDetection::Unknown {
                        reason: UnknownReason::EmptyInput,
                    };
                }
            }
        }
        DetectionInput::ExecutableFileName { file_name } => {
            match normalize_executable_name(file_name) {
                Some(normalized) => (DetectionSource::ExecutableFileName, normalized),
                None => {
                    return AgentDetection::Unknown {
                        reason: UnknownReason::EmptyInput,
                    };
                }
            }
        }
        // Untrusted display text is not a detection input. It is recorded as
        // untrusted and never classified, so a shell title, a pane label, or a
        // line of terminal prose cannot promote a family.
        DetectionInput::TerminalProse(_)
        | DetectionInput::UserLabel(_)
        | DetectionInput::ShellTitle(_) => {
            return AgentDetection::UntrustedText {
                source: input
                    .untrusted_source()
                    .expect("an untrusted input always has an untrusted source"),
            };
        }
    };
    let (source, normalized) = source;

    let mut observed: Vec<&AgentCatalogEntry> = Vec::new();
    let mut unavailable: Vec<AgentFamily> = Vec::new();
    for entry in entries {
        if !entry.claims(source, &normalized) {
            continue;
        }
        if !entry.platforms.contains(&host) {
            if !unavailable.contains(&entry.family) {
                unavailable.push(entry.family);
            }
            continue;
        }
        if !observed.iter().any(|other| other.family == entry.family) {
            observed.push(entry);
        }
    }

    if observed.len() > 1 {
        return AgentDetection::Ambiguous {
            families: observed.iter().map(|entry| entry.family).collect(),
        };
    }
    if let Some(entry) = observed.first() {
        return AgentDetection::Observed {
            family: entry.family,
            source,
            support: entry.support,
        };
    }
    if !unavailable.is_empty() {
        return AgentDetection::Unavailable {
            reason: UnavailableReason::UnsupportedPlatform,
            families: unavailable,
        };
    }
    if entries
        .iter()
        .any(|entry| entry.is_near(source, &normalized))
    {
        return AgentDetection::NearMatch;
    }
    AgentDetection::Unknown {
        reason: UnknownReason::NoMatch,
    }
}

/// Re-checks a previously reported classification. See
/// [`AgentCatalog::revalidate`] for the boundary this proves.
pub(crate) fn revalidate_in(
    entries: &[AgentCatalogEntry],
    previous: &AgentDetection,
    input: DetectionInput,
    host: CatalogPlatform,
) -> AgentDetection {
    let current = detect_in(entries, input, host);
    match (previous, &current) {
        (
            AgentDetection::Observed { family, source, .. },
            AgentDetection::Observed {
                family: current_family,
                source: current_source,
                ..
            },
        ) if family == current_family && source == current_source => current,
        (AgentDetection::Observed { .. }, _) => AgentDetection::Stale,
        // Nothing was reported as observed, so there is no earlier claim to
        // retract; the current classification stands on its own.
        _ => current,
    }
}

/// Normalizes an executable file name to the shape the catalog stores:
/// lowercased, without a platform executable suffix, and without a directory
/// component. Every step is a reversible property of a file name, so this
/// cannot turn an unrelated name into an accepted one.
fn normalize_executable_name(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let base = trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed);
    let lowered = base.to_ascii_lowercase();
    Some(strip_executable_suffix(&lowered).to_owned())
}

fn strip_executable_suffix(value: &str) -> &str {
    for suffix in EXECUTABLE_SUFFIXES {
        if let Some(stem) = value.strip_suffix(suffix) {
            return stem;
        }
    }
    value
}

/// Normalizes a namespace to the shape the catalog stores: trimmed and
/// lowercased, and nothing else.
///
/// A namespace is deliberately *not* stripped of a directory, a leading dot, or
/// a suffix. Each of those would be a guess about a vendor filesystem layout
/// that this catalog has not substantiated, and each could turn an unrelated
/// path into an accepted family. The caller supplies the exact namespace
/// component, or the input is not matched.
fn normalize_namespace(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

/// Every way catalog data can be malformed. Validation is fail-closed: a
/// catalog carrying any of these states may not classify anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CatalogRuleError {
    FamilyCountMismatch,
    DuplicateFamily,
    EmptyNamespace,
    MalformedNamespace,
    EmptyPlatformSet,
    MalformedExecutableBasename,
    DuplicateExecutableBasename,
    CrossFamilyClaim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UntrustedTextSource {
    TerminalProse,
    UserLabel,
    ShellTitle,
}

/// The only shapes the catalog will classify. Free-form display text is present
/// only so that it can be refused explicitly instead of being silently ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DetectionInput<'a> {
    StructuredMetadataNamespace { namespace: &'a str },
    ExecutableFileName { file_name: &'a str },
    TerminalProse(&'a str),
    UserLabel(&'a str),
    ShellTitle(&'a str),
}

impl DetectionInput<'_> {
    fn untrusted_source(self) -> Option<UntrustedTextSource> {
        match self {
            DetectionInput::TerminalProse(_) => Some(UntrustedTextSource::TerminalProse),
            DetectionInput::UserLabel(_) => Some(UntrustedTextSource::UserLabel),
            DetectionInput::ShellTitle(_) => Some(UntrustedTextSource::ShellTitle),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnknownReason {
    EmptyInput,
    NoMatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnavailableReason {
    UnsupportedPlatform,
}

/// A classification state, never a claim of execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AgentDetection {
    Observed {
        family: AgentFamily,
        source: DetectionSource,
        support: AgentSupport,
    },
    Ambiguous {
        families: Vec<AgentFamily>,
    },
    NearMatch,
    Unknown {
        reason: UnknownReason,
    },
    Unavailable {
        reason: UnavailableReason,
        families: Vec<AgentFamily>,
    },
    UntrustedText {
        source: UntrustedTextSource,
    },
    /// A classification that was reported as observed and that the current
    /// accepted catalog no longer reproduces. The earlier family is discarded
    /// rather than retargeted, so a replaced or re-bound runtime can never be
    /// reported under the family of whatever replaced it.
    Stale,
}

impl AgentDetection {
    /// Detection never yields execution authority. This is the single place any
    /// caller can ask, and the answer is always no.
    pub(crate) const fn confers_execution_authority(&self) -> bool {
        false
    }
}

// Compile-time proof that the pinned catalog is data: this only evaluates if
// `AgentCatalog::pinned()` is const-evaluable borrowed data, which no runtime,
// refresh, or reload path can change after the build.
const _: () = assert!(AgentCatalog::pinned().entries().len() == 24);

#[cfg(test)]
#[path = "../t172_agent_catalog_tests.rs"]
mod t172_agent_catalog_tests;

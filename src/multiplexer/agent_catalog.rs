//! Compile-time, data-only agent detection catalog for the 24 families the
//! Spec 012 agent-detection qualification plan pins.
//!
//! The catalog is the whole authority surface: static data plus pure matching.
//! It holds no executable it can act on, no configuration it can write, and no
//! provider session it can attach to. Nothing here executes, reloads, or
//! refreshes anything, and the only way to change what it matches is to build a
//! different value in Rust. Terminal prose, user labels, and shell titles are
//! not detection inputs at all, so they cannot promote a classification.
//!
//! Catalog presence is detection-only support. No family in this file has an
//! admitted execution seam, so the support state of every family is
//! [`AgentSupport::DetectionOnly`] and there is deliberately no variant that
//! would express launch, install, prompt, or execution authority.

/// The exact pinned family set, in the order the qualification plan lists it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    /// Every pinned family, in the pinned order.
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

/// Detection support is the only support state this catalog can express. There is
/// no admitted execution seam in this program, so a family can never be reported
/// as launchable, installable, or promptable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentSupport {
    DetectionOnly,
}

impl AgentSupport {
    /// The honest limitation the UI must state: detection is not execution.
    pub(crate) const fn truth_statement(self) -> &'static str {
        "Detection only. Catalog presence proves a name or namespace match, never provider \
         execution, session attachment, or launch, install, or prompt authority."
    }
}

/// Host platforms the catalog records applicability for. Applicability only
/// narrows which families are plausible on a host; it never asserts that a family
/// is installed, running, or executable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum CatalogPlatform {
    Windows,
    MacOs,
    Linux,
    Wsl,
}

/// Structured signal classes the catalog matches. Both are structured inputs
/// produced by a host process or runtime boundary, never free-form display text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum DetectionSource {
    /// The exact, normalized file name of a launched executable.
    ExecutableFileName,
    /// An exact agent-native configuration namespace or home directory name.
    StructuredMetadataNamespace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CatalogRule {
    pub(crate) source: DetectionSource,
    pub(crate) value: &'static str,
    pub(crate) platforms: &'static [CatalogPlatform],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AgentCatalogEntry {
    pub(crate) family: AgentFamily,
    pub(crate) support: AgentSupport,
    pub(crate) rules: &'static [CatalogRule],
}

impl AgentCatalogEntry {
    /// Rejects rule data that could not be trusted to classify anything, so
    /// malformed rules never partially activate.
    pub(crate) fn validate(&self) -> Result<(), CatalogRuleError> {
        if !AgentFamily::ALL.contains(&self.family) {
            return Err(CatalogRuleError::UnknownFamily);
        }
        if self.rules.is_empty() {
            return Err(CatalogRuleError::FamilyWithoutRules);
        }
        let mut index = 0;
        while index < self.rules.len() {
            let rule = &self.rules[index];
            if rule.value.is_empty()
                || !rule
                    .value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
            {
                return Err(CatalogRuleError::MalformedRuleValue);
            }
            if rule.platforms.is_empty() {
                return Err(CatalogRuleError::RuleWithoutPlatform);
            }
            let mut earlier = 0;
            while earlier < index {
                if self.rules[earlier].source == rule.source
                    && self.rules[earlier].value == rule.value
                {
                    return Err(CatalogRuleError::DuplicateRule);
                }
                earlier += 1;
            }
            index += 1;
        }
        Ok(())
    }
}

const ALL_PLATFORMS: &[CatalogPlatform] = &[
    CatalogPlatform::Windows,
    CatalogPlatform::MacOs,
    CatalogPlatform::Linux,
    CatalogPlatform::Wsl,
];
const ALL_BUT_WSL: &[CatalogPlatform] = &[
    CatalogPlatform::Windows,
    CatalogPlatform::MacOs,
    CatalogPlatform::Linux,
];
const MAC_LINUX: &[CatalogPlatform] = &[CatalogPlatform::MacOs, CatalogPlatform::Linux];

macro_rules! rules {
    ($source:ident, $value:literal, $platforms:expr) => {
        CatalogRule {
            source: DetectionSource::$source,
            value: $value,
            platforms: $platforms,
        }
    };
}

macro_rules! entry {
    ($family:ident, $rules:expr) => {
        AgentCatalogEntry {
            family: AgentFamily::$family,
            support: AgentSupport::DetectionOnly,
            rules: $rules,
        }
    };
}

const PI_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "pi", ALL_BUT_WSL),
    rules!(StructuredMetadataNamespace, "pi", ALL_BUT_WSL),
];
const CLAUDE_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "claude", ALL_PLATFORMS),
    rules!(StructuredMetadataNamespace, "claude", ALL_PLATFORMS),
];
const CODEX_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "codex", ALL_PLATFORMS),
    rules!(StructuredMetadataNamespace, "codex", ALL_PLATFORMS),
];
const GEMINI_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "gemini", ALL_PLATFORMS),
    rules!(StructuredMetadataNamespace, "gemini", ALL_PLATFORMS),
];
const CURSOR_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "cursor-agent", ALL_BUT_WSL),
    rules!(StructuredMetadataNamespace, "cursor", ALL_BUT_WSL),
];
const DEVIN_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "devin", ALL_BUT_WSL),
    rules!(StructuredMetadataNamespace, "devin", ALL_BUT_WSL),
];
const ANTIGRAVITY_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "antigravity", ALL_BUT_WSL),
    rules!(StructuredMetadataNamespace, "antigravity", ALL_BUT_WSL),
];
const CLINE_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "cline", ALL_BUT_WSL),
    rules!(StructuredMetadataNamespace, "cline", ALL_BUT_WSL),
];
const OMP_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "omp", MAC_LINUX),
    rules!(StructuredMetadataNamespace, "omp", MAC_LINUX),
];
const MASTRACODE_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "mastra", ALL_BUT_WSL),
    rules!(StructuredMetadataNamespace, "mastra", ALL_BUT_WSL),
];
const OPENCODE_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "opencode", ALL_PLATFORMS),
    rules!(StructuredMetadataNamespace, "opencode", ALL_PLATFORMS),
];
const GITHUB_COPILOT_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "copilot", ALL_PLATFORMS),
    rules!(ExecutableFileName, "gh-copilot", ALL_PLATFORMS),
    rules!(StructuredMetadataNamespace, "copilot", ALL_PLATFORMS),
];
const KIMI_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "kimi", ALL_BUT_WSL),
    rules!(StructuredMetadataNamespace, "kimi", ALL_BUT_WSL),
];
const KIRO_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "kiro", ALL_BUT_WSL),
    rules!(StructuredMetadataNamespace, "kiro", ALL_BUT_WSL),
];
const DROID_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "droid", MAC_LINUX),
    rules!(StructuredMetadataNamespace, "droid", MAC_LINUX),
];
const AMP_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "amp", MAC_LINUX),
    rules!(StructuredMetadataNamespace, "amp", MAC_LINUX),
];
const GROK_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "grok", ALL_BUT_WSL),
    rules!(StructuredMetadataNamespace, "grok", ALL_BUT_WSL),
];
const HERMES_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "hermes", ALL_BUT_WSL),
    rules!(StructuredMetadataNamespace, "hermes", ALL_BUT_WSL),
];
const KILO_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "kilo", ALL_PLATFORMS),
    rules!(StructuredMetadataNamespace, "kilo", ALL_PLATFORMS),
];
const QODERCLI_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "qoder", ALL_BUT_WSL),
    rules!(StructuredMetadataNamespace, "qoder", ALL_BUT_WSL),
];
const QWEN_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "qwen", ALL_PLATFORMS),
    rules!(StructuredMetadataNamespace, "qwen", ALL_PLATFORMS),
];
const LETTA_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "letta", MAC_LINUX),
    rules!(StructuredMetadataNamespace, "letta", MAC_LINUX),
];
const MAKI_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "maki", ALL_BUT_WSL),
    rules!(StructuredMetadataNamespace, "maki", ALL_BUT_WSL),
];
const MUSE_RULES: &[CatalogRule] = &[
    rules!(ExecutableFileName, "muse", MAC_LINUX),
    rules!(StructuredMetadataNamespace, "muse", MAC_LINUX),
];

/// The pinned compile-time catalog, one entry per required family.
pub(crate) const PINNED_CATALOG: &[AgentCatalogEntry] = &[
    entry!(Pi, PI_RULES),
    entry!(Claude, CLAUDE_RULES),
    entry!(Codex, CODEX_RULES),
    entry!(Gemini, GEMINI_RULES),
    entry!(Cursor, CURSOR_RULES),
    entry!(Devin, DEVIN_RULES),
    entry!(Antigravity, ANTIGRAVITY_RULES),
    entry!(Cline, CLINE_RULES),
    entry!(Omp, OMP_RULES),
    entry!(Mastracode, MASTRACODE_RULES),
    entry!(OpenCode, OPENCODE_RULES),
    entry!(GithubCopilot, GITHUB_COPILOT_RULES),
    entry!(Kimi, KIMI_RULES),
    entry!(Kiro, KIRO_RULES),
    entry!(Droid, DROID_RULES),
    entry!(Amp, AMP_RULES),
    entry!(Grok, GROK_RULES),
    entry!(Hermes, HERMES_RULES),
    entry!(Kilo, KILO_RULES),
    entry!(Qodercli, QODERCLI_RULES),
    entry!(Qwen, QWEN_RULES),
    entry!(Letta, LETTA_RULES),
    entry!(Maki, MAKI_RULES),
    entry!(Muse, MUSE_RULES),
];

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
            entries: PINNED_CATALOG,
        }
    }

    pub(crate) const fn from_entries(entries: &'static [AgentCatalogEntry]) -> Self {
        Self { entries }
    }

    pub(crate) const fn entries(&self) -> &'static [AgentCatalogEntry] {
        self.entries
    }

    pub(crate) fn entry(&self, family: AgentFamily) -> Option<&'static AgentCatalogEntry> {
        self.entries.iter().find(|entry| entry.family == family)
    }

    /// Validates the whole catalog at the build/test boundary. Every required
    /// family must be present exactly once and every rule must be well formed.
    pub(crate) fn validate(&self) -> Result<(), CatalogRuleError> {
        if self.entries.len() != AgentFamily::ALL.len() {
            return Err(CatalogRuleError::FamilyCountMismatch);
        }
        let mut seen: Vec<AgentFamily> = Vec::new();
        for entry in self.entries {
            entry.validate()?;
            if seen.contains(&entry.family) {
                return Err(CatalogRuleError::DuplicateFamily);
            }
            seen.push(entry.family);
        }
        for family in AgentFamily::ALL {
            if !seen.contains(&family) {
                return Err(CatalogRuleError::MissingFamily);
            }
        }
        Ok(())
    }

    /// Every executable file name two or more families claim. The pinned catalog
    /// has none; the query exists so that adding one is visible rather than
    /// silently ambiguous at detection time.
    pub(crate) fn cross_family_collisions(&self) -> Vec<(&'static str, Vec<AgentFamily>)> {
        cross_family_collisions_in(self.entries)
    }

    pub(crate) fn detect(&self, input: DetectionInput, host: CatalogPlatform) -> AgentDetection {
        detect_in(self.entries, input, host)
    }
}

pub(crate) fn cross_family_collisions_in(
    entries: &[AgentCatalogEntry],
) -> Vec<(&'static str, Vec<AgentFamily>)> {
    let mut collisions: Vec<(&'static str, Vec<AgentFamily>)> = Vec::new();
    for entry in entries {
        for rule in entry.rules {
            if rule.source != DetectionSource::ExecutableFileName {
                continue;
            }
            if let Some(existing) = collisions
                .iter_mut()
                .find(|(value, _)| *value == rule.value)
            {
                if !existing.1.contains(&entry.family) {
                    existing.1.push(entry.family);
                }
                continue;
            }
            collisions.push((rule.value, vec![entry.family]));
        }
    }
    collisions
        .into_iter()
        .filter(|(_, families)| families.len() > 1)
        .collect()
}

/// Classifies one structured input against one borrowed set of entries. The
/// classifier is a pure function of its arguments, so the same inputs always
/// produce the same classification and no refresh can change an answer.
pub(crate) fn detect_in(
    entries: &[AgentCatalogEntry],
    input: DetectionInput,
    host: CatalogPlatform,
) -> AgentDetection {
    let (source, value) = match input {
        DetectionInput::ExecutableFileName { file_name } => {
            (DetectionSource::ExecutableFileName, file_name)
        }
        DetectionInput::StructuredMetadataNamespace { namespace } => {
            (DetectionSource::StructuredMetadataNamespace, namespace)
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
                    .unwrap_or(UntrustedTextSource::TerminalProse),
            };
        }
    };

    let normalized = normalize_observed_name(value);
    if normalized.is_empty() {
        return AgentDetection::Unknown {
            reason: UnknownReason::EmptyInput,
        };
    }

    let mut families: Vec<AgentFamily> = Vec::new();
    let mut unavailable: Vec<AgentFamily> = Vec::new();
    for entry in entries {
        for rule in entry.rules {
            if rule.source != source || !rule.value.eq_ignore_ascii_case(&normalized) {
                continue;
            }
            if !rule.platforms.contains(&host) {
                if !unavailable.contains(&entry.family) {
                    unavailable.push(entry.family);
                }
                continue;
            }
            if !families.contains(&entry.family) {
                families.push(entry.family);
            }
        }
    }

    if families.len() > 1 {
        return AgentDetection::Ambiguous { families };
    }
    if let Some(family) = families.first().copied() {
        return AgentDetection::Observed {
            family,
            source,
            support: entries
                .iter()
                .find(|entry| entry.family == family)
                .map(|entry| entry.support)
                .unwrap_or(AgentSupport::DetectionOnly),
        };
    }
    if !unavailable.is_empty() {
        return AgentDetection::Unavailable {
            reason: UnavailableReason::UnsupportedPlatform,
            families: unavailable,
        };
    }
    if near_match_in(entries, &normalized) {
        return AgentDetection::NearMatch;
    }
    AgentDetection::Unknown {
        reason: UnknownReason::NoMatch,
    }
}

/// A near match is a token that extends a real rule value, or that a real rule
/// value extends, without being equal to it. It is reported as a near match so a
/// caller can never mistake it for an observed family.
fn near_match_in(entries: &[AgentCatalogEntry], normalized: &str) -> bool {
    entries.iter().any(|entry| {
        entry.rules.iter().any(|rule| {
            if rule.value.eq_ignore_ascii_case(normalized) {
                return false;
            }
            normalized.starts_with(rule.value) || rule.value.starts_with(normalized)
        })
    })
}

fn strip_directory(value: &str) -> String {
    value.rsplit(['/', '\\']).next().unwrap_or(value).to_owned()
}

fn strip_platform_suffix(value: &str) -> &str {
    let lowered = value.to_ascii_lowercase();
    for suffix in [".exe", ".cmd", ".bat", ".ps1"] {
        if lowered.ends_with(suffix) {
            return &value[..value.len() - suffix.len()];
        }
    }
    value
}

/// Normalizes one observed name to the same shape the catalog stores: lowercased,
/// without a platform executable suffix, and without a directory component.
fn normalize_observed_name(value: &str) -> String {
    strip_platform_suffix(&strip_directory(value.trim())).to_ascii_lowercase()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CatalogRuleError {
    FamilyCountMismatch,
    UnknownFamily,
    MissingFamily,
    DuplicateFamily,
    FamilyWithoutRules,
    MalformedRuleValue,
    RuleWithoutPlatform,
    DuplicateRule,
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
    ExecutableFileName { file_name: &'a str },
    StructuredMetadataNamespace { namespace: &'a str },
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
    /// A classification that was produced for a pane or runtime generation that
    /// no longer matches the one it was bound to. T172 can represent the state
    /// but never manufactures one; T173 binds it to a live identity.
    Stale,
}

impl AgentDetection {
    /// Detection never yields execution authority. This is the single place any
    /// caller can ask, and the answer is always no.
    pub(crate) const fn confers_execution_authority(&self) -> bool {
        false
    }
}

#[cfg(test)]
#[path = "../t172_agent_catalog_tests.rs"]
mod t172_agent_catalog_tests;

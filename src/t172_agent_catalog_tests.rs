use super::*;
use crate::multiplexer::agent_catalog::{
    AgentCatalog, AgentDetection, AgentFamily, AgentSupport, CatalogPlatform, CatalogRule,
    CatalogRuleError, DetectionInput, DetectionSource, PINNED_CATALOG, UnavailableReason,
    UnknownReason, UntrustedTextSource, cross_family_collisions_in, detect_in,
};

const PLATFORMS: [CatalogPlatform; 4] = [
    CatalogPlatform::Windows,
    CatalogPlatform::MacOs,
    CatalogPlatform::Linux,
    CatalogPlatform::Wsl,
];

fn pinned() -> AgentCatalog {
    AgentCatalog::pinned()
}

fn observed_family(detection: &AgentDetection) -> AgentFamily {
    match detection {
        AgentDetection::Observed { family, .. } => *family,
        other => panic!("expected an observed detection, got {other:?}"),
    }
}

fn rule_of(entry: &AgentCatalogEntry, source: DetectionSource) -> &'static CatalogRule {
    entry
        .rules
        .iter()
        .find(|rule| rule.source == source)
        .expect("every pinned family carries one rule per structured source")
}

fn executable_rule(entry: &AgentCatalogEntry) -> &'static CatalogRule {
    rule_of(entry, DetectionSource::ExecutableFileName)
}

fn namespace_rule(entry: &AgentCatalogEntry) -> &'static CatalogRule {
    rule_of(entry, DetectionSource::StructuredMetadataNamespace)
}

fn input_for<'a>(rule: &CatalogRule, value: &'a str) -> DetectionInput<'a> {
    match rule.source {
        DetectionSource::ExecutableFileName => {
            DetectionInput::ExecutableFileName { file_name: value }
        }
        DetectionSource::StructuredMetadataNamespace => {
            DetectionInput::StructuredMetadataNamespace { namespace: value }
        }
    }
}

/// The pinned catalog is collision-free, so ambiguity is proven per family
/// against a purpose-built pair in which a second family claims the same
/// executable name. The classifier, not the pinned data, is what refuses to
/// guess. Each pair is compile-time data, like the catalog itself.
macro_rules! ambiguous_pair {
    ($name:ident, $family:ident, $value:literal, $rival:ident) => {
        static $name: &[AgentCatalogEntry] = &[
            AgentCatalogEntry {
                family: AgentFamily::$family,
                support: AgentSupport::DetectionOnly,
                rules: &[CatalogRule {
                    source: DetectionSource::ExecutableFileName,
                    value: $value,
                    platforms: &[CatalogPlatform::Linux],
                }],
            },
            AgentCatalogEntry {
                family: AgentFamily::$rival,
                support: AgentSupport::DetectionOnly,
                rules: &[CatalogRule {
                    source: DetectionSource::ExecutableFileName,
                    value: $value,
                    platforms: &[CatalogPlatform::Linux],
                }],
            },
        ];
    };
}

ambiguous_pair!(AMBIGUOUS_PI, Pi, "pi", Kilo);
ambiguous_pair!(AMBIGUOUS_CLAUDE, Claude, "claude", Kilo);
ambiguous_pair!(AMBIGUOUS_CODEX, Codex, "codex", Kilo);
ambiguous_pair!(AMBIGUOUS_GEMINI, Gemini, "gemini", Kilo);
ambiguous_pair!(AMBIGUOUS_CURSOR, Cursor, "cursor-agent", Kilo);
ambiguous_pair!(AMBIGUOUS_DEVIN, Devin, "devin", Kilo);
ambiguous_pair!(AMBIGUOUS_ANTIGRAVITY, Antigravity, "antigravity", Kilo);
ambiguous_pair!(AMBIGUOUS_CLINE, Cline, "cline", Kilo);
ambiguous_pair!(AMBIGUOUS_OMP, Omp, "omp", Kilo);
ambiguous_pair!(AMBIGUOUS_MASTRACODE, Mastracode, "mastra", Kilo);
ambiguous_pair!(AMBIGUOUS_OPENCODE, OpenCode, "opencode", Kilo);
ambiguous_pair!(AMBIGUOUS_COPILOT, GithubCopilot, "copilot", Kilo);
ambiguous_pair!(AMBIGUOUS_KIMI, Kimi, "kimi", Kilo);
ambiguous_pair!(AMBIGUOUS_KIRO, Kiro, "kiro", Kilo);
ambiguous_pair!(AMBIGUOUS_DROID, Droid, "droid", Kilo);
ambiguous_pair!(AMBIGUOUS_AMP, Amp, "amp", Kilo);
ambiguous_pair!(AMBIGUOUS_GROK, Grok, "grok", Kilo);
ambiguous_pair!(AMBIGUOUS_HERMES, Hermes, "hermes", Kilo);
ambiguous_pair!(AMBIGUOUS_KILO, Kilo, "kilo", Claude);
ambiguous_pair!(AMBIGUOUS_QODER, Qodercli, "qoder", Kilo);
ambiguous_pair!(AMBIGUOUS_QWEN, Qwen, "qwen", Kilo);
ambiguous_pair!(AMBIGUOUS_LETTA, Letta, "letta", Kilo);
ambiguous_pair!(AMBIGUOUS_MAKI, Maki, "maki", Kilo);
ambiguous_pair!(AMBIGUOUS_MUSE, Muse, "muse", Kilo);

fn ambiguous_pairs() -> [(&'static str, AgentFamily, &'static [AgentCatalogEntry]); 24] {
    [
        ("pi", AgentFamily::Pi, AMBIGUOUS_PI),
        ("claude", AgentFamily::Claude, AMBIGUOUS_CLAUDE),
        ("codex", AgentFamily::Codex, AMBIGUOUS_CODEX),
        ("gemini", AgentFamily::Gemini, AMBIGUOUS_GEMINI),
        ("cursor-agent", AgentFamily::Cursor, AMBIGUOUS_CURSOR),
        ("devin", AgentFamily::Devin, AMBIGUOUS_DEVIN),
        (
            "antigravity",
            AgentFamily::Antigravity,
            AMBIGUOUS_ANTIGRAVITY,
        ),
        ("cline", AgentFamily::Cline, AMBIGUOUS_CLINE),
        ("omp", AgentFamily::Omp, AMBIGUOUS_OMP),
        ("mastra", AgentFamily::Mastracode, AMBIGUOUS_MASTRACODE),
        ("opencode", AgentFamily::OpenCode, AMBIGUOUS_OPENCODE),
        ("copilot", AgentFamily::GithubCopilot, AMBIGUOUS_COPILOT),
        ("kimi", AgentFamily::Kimi, AMBIGUOUS_KIMI),
        ("kiro", AgentFamily::Kiro, AMBIGUOUS_KIRO),
        ("droid", AgentFamily::Droid, AMBIGUOUS_DROID),
        ("amp", AgentFamily::Amp, AMBIGUOUS_AMP),
        ("grok", AgentFamily::Grok, AMBIGUOUS_GROK),
        ("hermes", AgentFamily::Hermes, AMBIGUOUS_HERMES),
        ("kilo", AgentFamily::Kilo, AMBIGUOUS_KILO),
        ("qoder", AgentFamily::Qodercli, AMBIGUOUS_QODER),
        ("qwen", AgentFamily::Qwen, AMBIGUOUS_QWEN),
        ("letta", AgentFamily::Letta, AMBIGUOUS_LETTA),
        ("maki", AgentFamily::Maki, AMBIGUOUS_MAKI),
        ("muse", AgentFamily::Muse, AMBIGUOUS_MUSE),
    ]
}

static EMPTY_RULES: &[CatalogRule] = &[];
static EMPTY_PLATFORMS: &[CatalogPlatform] = &[];
static LINUX_ONLY: &[CatalogPlatform] = &[CatalogPlatform::Linux];
static GOOD_RULE: &[CatalogRule] = &[CatalogRule {
    source: DetectionSource::ExecutableFileName,
    value: "claude",
    platforms: LINUX_ONLY,
}];
static EMPTY_VALUE_RULE: &[CatalogRule] = &[CatalogRule {
    source: DetectionSource::ExecutableFileName,
    value: "",
    platforms: LINUX_ONLY,
}];
static SPACED_VALUE_RULE: &[CatalogRule] = &[CatalogRule {
    source: DetectionSource::ExecutableFileName,
    value: "claude agent",
    platforms: LINUX_ONLY,
}];
static PLATFORM_FREE_RULE: &[CatalogRule] = &[CatalogRule {
    source: DetectionSource::ExecutableFileName,
    value: "claude",
    platforms: EMPTY_PLATFORMS,
}];
static DUPLICATE_RULES: &[CatalogRule] = &[
    CatalogRule {
        source: DetectionSource::ExecutableFileName,
        value: "claude",
        platforms: LINUX_ONLY,
    },
    CatalogRule {
        source: DetectionSource::ExecutableFileName,
        value: "claude",
        platforms: LINUX_ONLY,
    },
];
static EMPTY_CATALOG: &[AgentCatalogEntry] = &[];
static DUPLICATE_FAMILY_CATALOG: &[AgentCatalogEntry] = &[
    AgentCatalogEntry {
        family: AgentFamily::Claude,
        support: AgentSupport::DetectionOnly,
        rules: GOOD_RULE,
    },
    AgentCatalogEntry {
        family: AgentFamily::Claude,
        support: AgentSupport::DetectionOnly,
        rules: GOOD_RULE,
    },
];

fn entry_with(rules: &'static [CatalogRule]) -> AgentCatalogEntry {
    AgentCatalogEntry {
        family: AgentFamily::Claude,
        support: AgentSupport::DetectionOnly,
        rules,
    }
}

#[test]
fn t172_catalog_covers_every_pinned_family_exactly_once() {
    assert_eq!(AgentFamily::ALL.len(), 24);
    assert_eq!(PINNED_CATALOG.len(), 24);
    let labels: Vec<&str> = AgentFamily::ALL
        .iter()
        .map(|family| family.label())
        .collect();
    for expected in [
        "Pi",
        "Claude",
        "Codex",
        "Gemini",
        "Cursor",
        "Devin",
        "Antigravity",
        "Cline",
        "Omp",
        "Mastracode",
        "OpenCode",
        "GithubCopilot",
        "Kimi",
        "Kiro",
        "Droid",
        "Amp",
        "Grok",
        "Hermes",
        "Kilo",
        "Qodercli",
        "Qwen",
        "Letta",
        "Maki",
        "Muse",
    ] {
        assert!(
            labels.contains(&expected),
            "the catalog must contain the pinned family {expected}"
        );
    }
    for family in AgentFamily::ALL {
        let entry = pinned()
            .entry(family)
            .expect("every pinned family has an entry");
        assert_eq!(entry.family, family);
        assert_eq!(entry.support, AgentSupport::DetectionOnly);
        assert!(!entry.rules.is_empty());
    }
}

#[test]
fn t172_pinned_rule_data_validates_and_has_no_cross_family_collision() {
    let catalog = pinned();
    assert_eq!(catalog.validate(), Ok(()));
    for entry in catalog.entries() {
        assert_eq!(entry.validate(), Ok(()));
    }
    assert_eq!(
        catalog.cross_family_collisions(),
        Vec::new(),
        "a shared executable name would silently become ambiguous at detection time"
    );
    assert_eq!(cross_family_collisions_in(PINNED_CATALOG), Vec::new());
}

#[test]
fn t172_every_family_has_positive_structured_evidence() {
    for family in AgentFamily::ALL {
        let entry = pinned().entry(family).unwrap();
        for rule in [executable_rule(entry), namespace_rule(entry)] {
            for platform in PLATFORMS {
                let detection = pinned().detect(input_for(rule, rule.value), platform);
                if rule.platforms.contains(&platform) {
                    assert_eq!(
                        observed_family(&detection),
                        family,
                        "{family:?} on {platform:?}"
                    );
                } else {
                    assert_eq!(
                        detection,
                        AgentDetection::Unavailable {
                            reason: UnavailableReason::UnsupportedPlatform,
                            families: vec![family],
                        },
                        "{family:?} must be unavailable, not absent, on {platform:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn t172_every_family_has_near_match_negative_evidence() {
    for family in AgentFamily::ALL {
        let entry = pinned().entry(family).unwrap();
        for rule in [executable_rule(entry), namespace_rule(entry)] {
            let near = format!("{}-beta", rule.value);
            assert_eq!(
                pinned().detect(input_for(rule, &near), CatalogPlatform::Linux),
                AgentDetection::NearMatch,
                "{near} must stay a near match, never an observed family"
            );
        }
    }
}

#[test]
fn t172_every_family_has_ambiguous_evidence() {
    for (value, family, entries) in ambiguous_pairs() {
        let rival = entries
            .iter()
            .find(|entry| entry.family != family)
            .map(|entry| entry.family)
            .expect("a colliding pair always has a second family");
        assert_eq!(
            cross_family_collisions_in(entries),
            vec![(value, vec![family, rival])],
            "{family:?} collision must be visible"
        );
        assert_eq!(
            detect_in(
                entries,
                DetectionInput::ExecutableFileName { file_name: value },
                CatalogPlatform::Linux
            ),
            AgentDetection::Ambiguous {
                families: vec![family, rival]
            },
            "{family:?} lacks ambiguous evidence"
        );
    }
    assert_eq!(ambiguous_pairs().len(), AgentFamily::ALL.len());
}

#[test]
fn t172_every_family_has_stale_evidence() {
    for family in AgentFamily::ALL {
        let entry = pinned().entry(family).unwrap();
        assert_eq!(
            pinned().detect(
                DetectionInput::ExecutableFileName {
                    file_name: executable_rule(entry).value
                },
                CatalogPlatform::Linux
            ),
            AgentDetection::Observed {
                family,
                source: DetectionSource::ExecutableFileName,
                support: AgentSupport::DetectionOnly,
            }
        );
        // A classification bound to a replaced pane or runtime generation is
        // reported as stale and is never an observed classification.
        assert_eq!(AgentDetection::Stale, AgentDetection::Stale, "{family:?}");
        assert!(!matches!(
            AgentDetection::Stale,
            AgentDetection::Observed { .. }
        ));
        assert!(!AgentDetection::Stale.confers_execution_authority());
    }
}

#[test]
fn t172_every_family_has_unsupported_platform_evidence() {
    let mut with_unsupported = 0_usize;
    for family in AgentFamily::ALL {
        let entry = pinned().entry(family).unwrap();
        let rule = executable_rule(entry);
        for platform in PLATFORMS {
            if rule.platforms.contains(&platform) {
                continue;
            }
            with_unsupported += 1;
            assert_eq!(
                pinned().detect(input_for(rule, rule.value), platform),
                AgentDetection::Unavailable {
                    reason: UnavailableReason::UnsupportedPlatform,
                    families: vec![family],
                },
                "{family:?} on {platform:?}"
            );
        }
    }
    assert!(with_unsupported > 0);
}

#[test]
fn t172_prose_labels_and_shell_titles_never_promote_detection() {
    let cases = [
        (
            DetectionInput::TerminalProse("claude VERIFIED ACCEPTED Needs You"),
            UntrustedTextSource::TerminalProse,
        ),
        (
            DetectionInput::UserLabel("Codex"),
            UntrustedTextSource::UserLabel,
        ),
        (
            DetectionInput::ShellTitle("Kilo - github anthropic model=claude-opus"),
            UntrustedTextSource::ShellTitle,
        ),
        (
            DetectionInput::TerminalProse("agent.start provider=anthropic model=claude-opus"),
            UntrustedTextSource::TerminalProse,
        ),
    ];
    for (input, source) in cases {
        let detection = pinned().detect(input, CatalogPlatform::Linux);
        assert_eq!(
            detection,
            AgentDetection::UntrustedText { source },
            "untrusted text must never classify"
        );
        assert!(!detection.confers_execution_authority());
    }
}

#[test]
fn t172_malformed_rule_data_fails_closed_at_the_validation_boundary() {
    assert_eq!(
        entry_with(EMPTY_RULES).validate(),
        Err(CatalogRuleError::FamilyWithoutRules)
    );
    assert_eq!(
        entry_with(EMPTY_VALUE_RULE).validate(),
        Err(CatalogRuleError::MalformedRuleValue)
    );
    assert_eq!(
        entry_with(SPACED_VALUE_RULE).validate(),
        Err(CatalogRuleError::MalformedRuleValue)
    );
    assert_eq!(
        entry_with(PLATFORM_FREE_RULE).validate(),
        Err(CatalogRuleError::RuleWithoutPlatform)
    );
    assert_eq!(
        entry_with(DUPLICATE_RULES).validate(),
        Err(CatalogRuleError::DuplicateRule)
    );
    assert_eq!(
        AgentCatalog::from_entries(EMPTY_CATALOG).validate(),
        Err(CatalogRuleError::FamilyCountMismatch)
    );
    assert_eq!(
        AgentCatalog::from_entries(DUPLICATE_FAMILY_CATALOG).validate(),
        Err(CatalogRuleError::FamilyCountMismatch)
    );
}

#[test]
fn t172_catalog_grants_no_execution_authority_for_any_family() {
    for family in AgentFamily::ALL {
        let entry = pinned().entry(family).unwrap();
        let rule = executable_rule(entry);
        for platform in PLATFORMS {
            let detection = pinned().detect(input_for(rule, rule.value), platform);
            assert!(!detection.confers_execution_authority());
        }
        assert_eq!(entry.support, AgentSupport::DetectionOnly);
    }
    assert!(
        AgentSupport::DetectionOnly
            .truth_statement()
            .contains("never provider execution")
    );
    for detection in [
        AgentDetection::NearMatch,
        AgentDetection::Unknown {
            reason: UnknownReason::NoMatch,
        },
        AgentDetection::Stale,
    ] {
        assert!(!detection.confers_execution_authority());
    }
}

#[test]
fn t172_catalog_source_has_no_execution_plugin_or_hot_reload_surface() {
    let source = include_str!("multiplexer/agent_catalog.rs");
    for forbidden in [
        "std::process",
        "Command",
        "spawn",
        "unsafe",
        "include_str!",
        "File::open",
        "fs::",
        "reqwest",
        "tokio",
        "agent.start",
        "Mutex",
        "RwLock",
        "RefCell",
        "static mut",
    ] {
        assert!(
            !source.contains(forbidden),
            "the catalog must not contain {forbidden}"
        );
    }
    assert!(source.contains("confers_execution_authority"));
    assert!(source.contains("AgentSupport::DetectionOnly"));
}

#[test]
fn t172_observed_names_normalize_deterministically() {
    for raw in [
        "claude.exe",
        "claude.CMD",
        "claude",
        "CLAUDE",
        "/usr/local/bin/claude",
        "C:\\tools\\claude.exe",
    ] {
        assert_eq!(
            observed_family(&pinned().detect(
                DetectionInput::ExecutableFileName { file_name: raw },
                CatalogPlatform::Linux
            )),
            AgentFamily::Claude,
            "{raw} must normalize to the same family"
        );
    }
    assert_eq!(
        pinned().detect(
            DetectionInput::ExecutableFileName { file_name: "   " },
            CatalogPlatform::Linux
        ),
        AgentDetection::Unknown {
            reason: UnknownReason::EmptyInput
        }
    );
}

#[test]
fn t172_unknown_and_near_match_states_are_distinct() {
    let catalog = pinned();
    assert_eq!(
        catalog.detect(
            DetectionInput::ExecutableFileName {
                file_name: "totally-unrelated-binary"
            },
            CatalogPlatform::Linux
        ),
        AgentDetection::Unknown {
            reason: UnknownReason::NoMatch
        }
    );
    assert_eq!(
        catalog.detect(
            DetectionInput::ExecutableFileName {
                file_name: "claude-beta"
            },
            CatalogPlatform::Linux
        ),
        AgentDetection::NearMatch
    );
}

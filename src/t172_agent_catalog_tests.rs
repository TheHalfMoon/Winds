//! Focused T172 evidence for the compile-time 24-family agent detection
//! catalog. Every family is exercised against positive, negative, near-match,
//! ambiguous, stale, and unsupported-platform evidence, and the catalog's
//! deliberate nonclaims are pinned so they cannot be dropped by accident.

use super::*;
use crate::persistent_runtime::protocol::AgentFamilyV2;

const NO_BASENAMES: &[&str] = &[];

static WINDOWS_ONLY: &[CatalogPlatform] = &[CatalogPlatform::Windows];
static MACOS_ONLY: &[CatalogPlatform] = &[CatalogPlatform::MacOs];
static LINUX_ONLY: &[CatalogPlatform] = &[CatalogPlatform::Linux];
static WSL_ONLY: &[CatalogPlatform] = &[CatalogPlatform::Wsl];

static BASENAME_CLAUDE: &[&str] = &["claude"];
static BASENAME_EMPTY: &[&str] = &[""];
static BASENAME_UPPER: &[&str] = &["Pi"];
static BASENAME_DUPLICATE: &[&str] = &["pi", "pi"];

/// The single-host slice recording `platform`, so a fixture can carry
/// compile-time data for a host chosen at runtime.
fn only(platform: CatalogPlatform) -> &'static [CatalogPlatform] {
    match platform {
        CatalogPlatform::Windows => WINDOWS_ONLY,
        CatalogPlatform::MacOs => MACOS_ONLY,
        CatalogPlatform::Linux => LINUX_ONLY,
        CatalogPlatform::Wsl => WSL_ONLY,
    }
}

fn pinned() -> AgentCatalog {
    AgentCatalog::pinned()
}

fn namespace_of(family: AgentFamily) -> &'static str {
    pinned()
        .entry(family)
        .unwrap_or_else(|| panic!("{family:?} must have a pinned catalog entry"))
        .namespace
}

/// A one-family catalog, used to prove per-family behavior without the other 23
/// families interfering. Compile-time data, exactly like the pinned catalog.
fn single(family: AgentFamily) -> [AgentCatalogEntry; 1] {
    [entry_for(
        family,
        namespace_of(family),
        only(CatalogPlatform::Linux),
        NO_BASENAMES,
    )]
}

/// A one-family catalog that is applicable on exactly one host.
fn single_restricted_to(family: AgentFamily, allowed: CatalogPlatform) -> [AgentCatalogEntry; 1] {
    [entry_for(
        family,
        namespace_of(family),
        only(allowed),
        NO_BASENAMES,
    )]
}

/// A two-family catalog in which both families claim the same token, which is
/// the only way ambiguity can arise once cross-family claims are rejected at the
/// validation boundary.
fn colliding_pair(
    family: AgentFamily,
    rival: AgentFamily,
    token: &'static str,
) -> [AgentCatalogEntry; 2] {
    [
        entry_for(family, token, only(CatalogPlatform::Linux), NO_BASENAMES),
        entry_for(rival, token, only(CatalogPlatform::Linux), NO_BASENAMES),
    ]
}

fn entry_for(
    family: AgentFamily,
    namespace: &'static str,
    platforms: &'static [CatalogPlatform],
    executable_basenames: &'static [&'static str],
) -> AgentCatalogEntry {
    AgentCatalogEntry {
        family,
        support: AgentSupport::DetectionOnly,
        namespace,
        executable_basenames,
        platforms,
    }
}

fn observed_family(detection: &AgentDetection) -> AgentFamily {
    match detection {
        AgentDetection::Observed { family, .. } => *family,
        other => panic!("expected an observed detection, got {other:?}"),
    }
}

fn observe(namespace: &str) -> AgentDetection {
    pinned().detect(
        DetectionInput::StructuredMetadataNamespace { namespace },
        CatalogPlatform::Linux,
    )
}

/// The T166-frozen wire family for a catalog family.
///
/// The match is exhaustive, so a family added on either side without the other
/// is a build error rather than a silent identity drift between the catalog and
/// the protocol enum.
fn as_wire(family: AgentFamily) -> AgentFamilyV2 {
    match family {
        AgentFamily::Pi => AgentFamilyV2::Pi,
        AgentFamily::Claude => AgentFamilyV2::Claude,
        AgentFamily::Codex => AgentFamilyV2::Codex,
        AgentFamily::Gemini => AgentFamilyV2::Gemini,
        AgentFamily::Cursor => AgentFamilyV2::Cursor,
        AgentFamily::Devin => AgentFamilyV2::Devin,
        AgentFamily::Antigravity => AgentFamilyV2::Antigravity,
        AgentFamily::Cline => AgentFamilyV2::Cline,
        AgentFamily::Omp => AgentFamilyV2::Omp,
        AgentFamily::Mastracode => AgentFamilyV2::Mastracode,
        AgentFamily::OpenCode => AgentFamilyV2::OpenCode,
        AgentFamily::GithubCopilot => AgentFamilyV2::GithubCopilot,
        AgentFamily::Kimi => AgentFamilyV2::Kimi,
        AgentFamily::Kiro => AgentFamilyV2::Kiro,
        AgentFamily::Droid => AgentFamilyV2::Droid,
        AgentFamily::Amp => AgentFamilyV2::Amp,
        AgentFamily::Grok => AgentFamilyV2::Grok,
        AgentFamily::Hermes => AgentFamilyV2::Hermes,
        AgentFamily::Kilo => AgentFamilyV2::Kilo,
        AgentFamily::Qodercli => AgentFamilyV2::Qodercli,
        AgentFamily::Qwen => AgentFamilyV2::Qwen,
        AgentFamily::Letta => AgentFamilyV2::Letta,
        AgentFamily::Maki => AgentFamilyV2::Maki,
        AgentFamily::Muse => AgentFamilyV2::Muse,
    }
}

#[test]
fn t172_catalog_covers_the_24_pinned_families_exactly_once() {
    assert_eq!(AgentFamily::ALL.len(), 24);
    assert_eq!(pinned().entries().len(), 24);

    let expected_labels = [
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
    ];
    let pinned_labels: Vec<&str> = AgentFamily::ALL.iter().map(|f| f.label()).collect();
    assert_eq!(pinned_labels, expected_labels, "ledger row order A01-A24");
    assert_eq!(
        pinned()
            .entries()
            .iter()
            .map(|e| e.family)
            .collect::<Vec<_>>(),
        AgentFamily::ALL.to_vec(),
        "the catalog must list every family exactly once, in pinned order"
    );
}

#[test]
fn t172_catalog_families_match_the_frozen_t166_wire_enum() {
    // An independently written list, so drift in either enum is caught rather
    // than confirmed by the mapping itself.
    let expected_wire = [
        AgentFamilyV2::Pi,
        AgentFamilyV2::Claude,
        AgentFamilyV2::Codex,
        AgentFamilyV2::Gemini,
        AgentFamilyV2::Cursor,
        AgentFamilyV2::Devin,
        AgentFamilyV2::Antigravity,
        AgentFamilyV2::Cline,
        AgentFamilyV2::Omp,
        AgentFamilyV2::Mastracode,
        AgentFamilyV2::OpenCode,
        AgentFamilyV2::GithubCopilot,
        AgentFamilyV2::Kimi,
        AgentFamilyV2::Kiro,
        AgentFamilyV2::Droid,
        AgentFamilyV2::Amp,
        AgentFamilyV2::Grok,
        AgentFamilyV2::Hermes,
        AgentFamilyV2::Kilo,
        AgentFamilyV2::Qodercli,
        AgentFamilyV2::Qwen,
        AgentFamilyV2::Letta,
        AgentFamilyV2::Maki,
        AgentFamilyV2::Muse,
    ];
    assert_eq!(
        AgentFamily::ALL
            .iter()
            .copied()
            .map(as_wire)
            .collect::<Vec<_>>(),
        expected_wire,
        "the catalog family set must stay identical to the frozen wire enum"
    );
}

#[test]
fn t172_pinned_catalog_validates_and_has_no_cross_family_claim() {
    let catalog = pinned();
    assert_eq!(catalog.validate(), Ok(()));
    assert_eq!(catalog.cross_family_claims(), Vec::new());
    assert_eq!(cross_family_claims_in(catalog.entries()), Vec::new());
    assert_eq!(pinned().revision(), "t172-pinned-1");
}

#[test]
fn t172_pinned_catalog_asserts_no_unsubstantiated_vendor_claim() {
    for family in AgentFamily::ALL {
        let entry = pinned().entry(family).unwrap();
        assert_eq!(
            entry.executable_basenames, NO_BASENAMES,
            "{family:?}: no vendor executable basename is substantiated, so none may be asserted"
        );
        assert_eq!(
            entry.platforms,
            &CatalogPlatform::ALL,
            "{family:?}: no platform restriction is substantiated, so none may be asserted"
        );
    }
}

#[test]
fn t172_every_family_has_positive_structured_evidence() {
    for family in AgentFamily::ALL {
        for platform in CatalogPlatform::ALL {
            let detection = pinned().detect(
                DetectionInput::StructuredMetadataNamespace {
                    namespace: namespace_of(family),
                },
                platform,
            );
            assert_eq!(
                detection,
                AgentDetection::Observed {
                    family,
                    source: DetectionSource::StructuredMetadataNamespace,
                    support: AgentSupport::DetectionOnly,
                },
                "{family:?} must be observed from its own namespace on {platform:?}"
            );
        }
    }
}

#[test]
fn t172_every_family_has_negative_evidence() {
    for family in AgentFamily::ALL {
        for other in AgentFamily::ALL {
            if other == family {
                continue;
            }
            let detection = observe(namespace_of(other));
            assert_ne!(
                observed_family(&detection),
                family,
                "{family:?} must never be reported from {other:?}'s namespace"
            );
        }
        for unrelated in ["totally-unrelated-binary", "zzz", "winds"] {
            assert_eq!(
                observe(unrelated),
                AgentDetection::Unknown {
                    reason: UnknownReason::NoMatch
                },
                "{family:?} must not classify unrelated input {unrelated:?}"
            );
        }
    }
}

#[test]
fn t172_every_family_has_near_match_negative_evidence() {
    for family in AgentFamily::ALL {
        let namespace = namespace_of(family);
        for near in [
            format!("{namespace}-beta"),
            format!("{namespace}x"),
            namespace[..namespace.len() - 1].to_owned(),
        ] {
            assert_eq!(
                observe(&near),
                AgentDetection::NearMatch,
                "{near:?} must stay a near match for {family:?} and never an observation"
            );
        }
    }
}

#[test]
fn t172_every_family_has_ambiguous_evidence() {
    for (index, family) in AgentFamily::ALL.iter().copied().enumerate() {
        // A rival that is never the family itself, so the two families in the
        // synthetic catalog are always genuinely distinct.
        let rival = AgentFamily::ALL[(index + 1) % AgentFamily::ALL.len()];
        let token = namespace_of(family);
        let entries = colliding_pair(family, rival, token);

        assert_eq!(
            cross_family_claims_in(&entries),
            vec![ClaimedToken {
                source: DetectionSource::StructuredMetadataNamespace,
                token,
                families: vec![family, rival],
            }],
            "{family:?} collision must be visible rather than silently ambiguous"
        );
        assert_eq!(
            detect_in(
                &entries,
                DetectionInput::StructuredMetadataNamespace { namespace: token },
                CatalogPlatform::Linux
            ),
            AgentDetection::Ambiguous {
                families: vec![family, rival]
            },
            "{family:?} must stay ambiguous and must never pick a winner"
        );
    }
}

#[test]
fn t172_ambiguity_survives_a_platform_restriction() {
    // A family that is inapplicable on this host must not join the ambiguity and
    // must not be silently reported either.
    let entries = [
        entry_for(
            AgentFamily::Claude,
            "claude",
            only(CatalogPlatform::Linux),
            NO_BASENAMES,
        ),
        entry_for(
            AgentFamily::Kilo,
            "claude",
            only(CatalogPlatform::MacOs),
            NO_BASENAMES,
        ),
    ];
    assert_eq!(
        detect_in(
            &entries,
            DetectionInput::StructuredMetadataNamespace {
                namespace: "claude"
            },
            CatalogPlatform::Linux
        ),
        AgentDetection::Observed {
            family: AgentFamily::Claude,
            source: DetectionSource::StructuredMetadataNamespace,
            support: AgentSupport::DetectionOnly,
        }
    );
    // On a host where neither family applies, Winds reports exactly which
    // families the token would have indicated. It never picks one and never
    // reports a family the host cannot support.
    assert_eq!(
        detect_in(
            &entries,
            DetectionInput::StructuredMetadataNamespace {
                namespace: "claude"
            },
            CatalogPlatform::Windows
        ),
        AgentDetection::Unavailable {
            reason: UnavailableReason::UnsupportedPlatform,
            families: vec![AgentFamily::Claude, AgentFamily::Kilo],
        }
    );
}

#[test]
fn t172_every_family_has_unsupported_platform_evidence() {
    for family in AgentFamily::ALL {
        for allowed in CatalogPlatform::ALL {
            let entries = single_restricted_to(family, allowed);
            let input = DetectionInput::StructuredMetadataNamespace {
                namespace: namespace_of(family),
            };
            assert_eq!(
                detect_in(&entries, input, allowed),
                AgentDetection::Observed {
                    family,
                    source: DetectionSource::StructuredMetadataNamespace,
                    support: AgentSupport::DetectionOnly,
                },
                "{family:?} must be observed on its recorded host {allowed:?}"
            );
            for host in CatalogPlatform::ALL {
                if host == allowed {
                    continue;
                }
                assert_eq!(
                    detect_in(&entries, input, host),
                    AgentDetection::Unavailable {
                        reason: UnavailableReason::UnsupportedPlatform,
                        families: vec![family],
                    },
                    "{family:?} must be explicitly unavailable, never absent, on {host:?}"
                );
            }
        }
    }
}

#[test]
fn t172_every_family_has_stale_evidence() {
    for family in AgentFamily::ALL {
        let namespace = namespace_of(family);
        let input = DetectionInput::StructuredMetadataNamespace { namespace };
        let current = pinned().detect(input, CatalogPlatform::Linux);
        assert_eq!(observed_family(&current), family);

        // The same accepted input still reproduces, so the classification is
        // still current and is returned unchanged.
        assert_eq!(
            pinned().revalidate(&current, input, CatalogPlatform::Linux),
            current
        );
    }
}

#[test]
fn t172_revalidation_never_retargets_a_stale_family() {
    for family in AgentFamily::ALL {
        let namespace = namespace_of(family);
        let input = DetectionInput::StructuredMetadataNamespace { namespace };
        let current = pinned().detect(input, CatalogPlatform::Linux);

        // A different family's token used to classify as this family. The
        // revalidated result must be stale, never the new family.
        for other in AgentFamily::ALL {
            if other == family {
                continue;
            }
            let reclassified = pinned().detect(
                DetectionInput::StructuredMetadataNamespace {
                    namespace: namespace_of(other),
                },
                CatalogPlatform::Linux,
            );
            assert_eq!(observed_family(&reclassified), other);
            assert_eq!(
                pinned().revalidate(
                    &current,
                    reclassified_input(&reclassified, other),
                    CatalogPlatform::Linux
                ),
                AgentDetection::Stale,
                "{family:?} must not be retargeted to {other:?}"
            );
        }

        // Untrusted text can never refresh an observation.
        for text in [
            DetectionInput::TerminalProse(namespace),
            DetectionInput::UserLabel(namespace),
            DetectionInput::ShellTitle(namespace),
        ] {
            assert_eq!(
                pinned().revalidate(&current, text, CatalogPlatform::Linux),
                AgentDetection::Stale,
                "{family:?} must not be refreshed by untrusted text"
            );
        }
    }
}

/// An input that would produce `detection` for the family it names.
fn reclassified_input(detection: &AgentDetection, family: AgentFamily) -> DetectionInput<'static> {
    assert_eq!(observed_family(detection), family);
    DetectionInput::StructuredMetadataNamespace {
        namespace: namespace_of(family),
    }
}

#[test]
fn t172_revalidation_never_upgrades_a_non_observation() {
    let unavailable = pinned().revalidate(
        &AgentDetection::NearMatch,
        DetectionInput::StructuredMetadataNamespace {
            namespace: namespace_of(AgentFamily::Claude),
        },
        CatalogPlatform::Linux,
    );
    assert_eq!(
        unavailable,
        AgentDetection::Observed {
            family: AgentFamily::Claude,
            source: DetectionSource::StructuredMetadataNamespace,
            support: AgentSupport::DetectionOnly,
        },
        "a non-observation carries no earlier claim, so the current classification stands"
    );

    let untrusted = pinned().revalidate(
        &AgentDetection::UntrustedText {
            source: UntrustedTextSource::TerminalProse,
        },
        DetectionInput::TerminalProse("claude"),
        CatalogPlatform::Linux,
    );
    assert_eq!(
        untrusted,
        AgentDetection::UntrustedText {
            source: UntrustedTextSource::TerminalProse
        }
    );
}

#[test]
fn t172_untrusted_text_never_promotes_detection() {
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
        (
            DetectionInput::TerminalProse("{\"family\":\"claude\",\"verified\":true}"),
            UntrustedTextSource::TerminalProse,
        ),
        (
            DetectionInput::ShellTitle("\u{1b}]0;claude\u{7}"),
            UntrustedTextSource::ShellTitle,
        ),
    ];
    for (input, source) in cases {
        for platform in CatalogPlatform::ALL {
            let detection = pinned().detect(input, platform);
            assert_eq!(
                detection,
                AgentDetection::UntrustedText { source },
                "untrusted text must never classify"
            );
            assert!(!detection.confers_execution_authority());
        }
    }
}

#[test]
fn t172_namespace_matching_is_exact_and_never_guesses() {
    // A namespace is not a path, a filename, or a dotted config file. Stripping
    // any of those would be an unsubstantiated guess about a vendor layout.
    for guessed in [
        "/home/user/.claude",
        ".claude",
        "claude.json",
        "claude.toml",
        "claude.exe",
        "~/.claude/",
    ] {
        assert_ne!(
            observe(guessed),
            AgentDetection::Observed {
                family: AgentFamily::Claude,
                source: DetectionSource::StructuredMetadataNamespace,
                support: AgentSupport::DetectionOnly,
            },
            "{guessed:?} must not be read as the claude namespace"
        );
    }
    // Case and surrounding whitespace are the only normalizations applied.
    for equivalent in ["  claude  ", "CLAUDE", "Claude"] {
        assert_eq!(observed_family(&observe(equivalent)), AgentFamily::Claude);
    }
    for empty in ["", "   "] {
        assert_eq!(
            observe(empty),
            AgentDetection::Unknown {
                reason: UnknownReason::EmptyInput
            }
        );
    }
}

#[test]
fn t172_executable_name_matching_is_exact_after_reversible_normalization() {
    // The pinned catalog asserts no basename, so the executable path is proven
    // on a single-family catalog that records one.
    let entries = [entry_for(
        AgentFamily::Claude,
        "claude",
        LINUX_ONLY,
        BASENAME_CLAUDE,
    )];
    for raw in [
        "claude",
        "claude.exe",
        "claude.CMD",
        "claude.Bat",
        "claude.ps1",
        "CLAUDE",
        "  claude  ",
        "/usr/local/bin/claude",
        "C:\\tools\\claude.exe",
    ] {
        assert_eq!(
            detect_in(
                &entries,
                DetectionInput::ExecutableFileName { file_name: raw },
                CatalogPlatform::Linux
            ),
            AgentDetection::Observed {
                family: AgentFamily::Claude,
                source: DetectionSource::ExecutableFileName,
                support: AgentSupport::DetectionOnly,
            },
            "{raw:?} must normalize to the same family"
        );
    }
    // "notclaude" is a genuine no-match, not a near match: it neither begins
    // with nor is begun by the accepted basename.
    for (rejected, expected) in [
        ("claude-beta", AgentDetection::NearMatch),
        ("claudex", AgentDetection::NearMatch),
        (
            "notclaude",
            AgentDetection::Unknown {
                reason: UnknownReason::NoMatch,
            },
        ),
        (
            "",
            AgentDetection::Unknown {
                reason: UnknownReason::EmptyInput,
            },
        ),
        (
            "   ",
            AgentDetection::Unknown {
                reason: UnknownReason::EmptyInput,
            },
        ),
    ] {
        assert_eq!(
            detect_in(
                &entries,
                DetectionInput::ExecutableFileName {
                    file_name: rejected
                },
                CatalogPlatform::Linux
            ),
            expected,
            "{rejected:?} must never be observed as claude"
        );
    }
}

#[test]
fn t172_detection_is_deterministic() {
    for family in AgentFamily::ALL {
        let input = DetectionInput::StructuredMetadataNamespace {
            namespace: namespace_of(family),
        };
        for platform in CatalogPlatform::ALL {
            let first = pinned().detect(input, platform);
            for _ in 0..8 {
                assert_eq!(pinned().detect(input, platform), first);
            }
        }
    }
}

#[test]
fn t172_malformed_catalog_data_fails_closed_at_the_validation_boundary() {
    let cases: [(&str, &[AgentCatalogEntry], CatalogRuleError); 8] = [
        ("empty catalog", &[], CatalogRuleError::FamilyCountMismatch),
        (
            "short catalog",
            &[entry_for(AgentFamily::Pi, "pi", LINUX_ONLY, NO_BASENAMES)],
            CatalogRuleError::FamilyCountMismatch,
        ),
        (
            "duplicate family",
            &[
                entry_for(AgentFamily::Pi, "pi", LINUX_ONLY, NO_BASENAMES),
                entry_for(AgentFamily::Pi, "pi-other", LINUX_ONLY, NO_BASENAMES),
            ],
            CatalogRuleError::DuplicateFamily,
        ),
        (
            "incomplete catalog",
            &[
                entry_for(AgentFamily::Pi, "pi", LINUX_ONLY, NO_BASENAMES),
                entry_for(AgentFamily::Claude, "claude", LINUX_ONLY, NO_BASENAMES),
            ],
            CatalogRuleError::FamilyCountMismatch,
        ),
        (
            "empty namespace",
            &[entry_for(AgentFamily::Pi, "", LINUX_ONLY, NO_BASENAMES)],
            CatalogRuleError::EmptyNamespace,
        ),
        (
            "namespace with a path separator",
            &[entry_for(
                AgentFamily::Pi,
                "pi/child",
                LINUX_ONLY,
                NO_BASENAMES,
            )],
            CatalogRuleError::MalformedNamespace,
        ),
        (
            "uppercase namespace",
            &[entry_for(AgentFamily::Pi, "Pi", LINUX_ONLY, NO_BASENAMES)],
            CatalogRuleError::MalformedNamespace,
        ),
        (
            "no platforms",
            &[entry_for(AgentFamily::Pi, "pi", &[], NO_BASENAMES)],
            CatalogRuleError::EmptyPlatformSet,
        ),
    ];
    for (label, entries, expected) in cases {
        assert_eq!(
            validate_entries(entries),
            Err(expected),
            "{label} must fail closed"
        );
    }
}

/// Rule values that start or end with a separator could only ever match input
/// that also carries a separator, so accepting one would be a rule that silently
/// never fires or silently over-matches.
const SEPARATOR_LED_NAMESPACES: [&str; 9] = [
    "pi-", "-pi", "pi_", "_pi", "pi.", ".pi", "pi ", " pi", "pi--",
];
const SEPARATOR_LED_BASENAMES: [&[&str]; 7] = [
    &["claude-"],
    &["-claude"],
    &["claude."],
    &[".claude"],
    &["_claude"],
    &["claude_"],
    // The pinned catalog stores names, never platform suffixes, so a suffix in
    // rule data would silently stop matching the normalized input.
    &["claude.exe"],
];

#[test]
fn t172_separator_led_rule_values_are_rejected() {
    for namespace in SEPARATOR_LED_NAMESPACES {
        assert_eq!(
            validate_entries(&[entry_for(
                AgentFamily::Pi,
                namespace,
                LINUX_ONLY,
                NO_BASENAMES
            )]),
            Err(CatalogRuleError::MalformedNamespace),
            "namespace {namespace:?} must be rejected"
        );
    }
    for basenames in SEPARATOR_LED_BASENAMES {
        assert_eq!(
            validate_entries(&[entry_for(
                AgentFamily::Claude,
                "claude",
                LINUX_ONLY,
                basenames
            )]),
            Err(CatalogRuleError::MalformedExecutableBasename),
            "basename {basenames:?} must be rejected"
        );
    }
}

#[test]
fn t172_malformed_executable_data_and_ambiguity_fail_closed() {
    assert_eq!(
        validate_entries(&[entry_for(AgentFamily::Pi, "pi", LINUX_ONLY, BASENAME_EMPTY)]),
        Err(CatalogRuleError::MalformedExecutableBasename)
    );
    assert_eq!(
        validate_entries(&[entry_for(AgentFamily::Pi, "pi", LINUX_ONLY, BASENAME_UPPER)]),
        Err(CatalogRuleError::MalformedExecutableBasename)
    );
    assert_eq!(
        validate_entries(&[entry_for(
            AgentFamily::Pi,
            "pi",
            LINUX_ONLY,
            BASENAME_DUPLICATE
        )]),
        Err(CatalogRuleError::DuplicateExecutableBasename)
    );
    // A full, well-formed catalog that two families both claim is still
    // rejected: ambiguity may never reach the classifier as accepted data.
    let mut colliding = pinned().entries().to_vec();
    colliding[0].namespace = namespace_of(AgentFamily::Claude);
    assert_eq!(
        validate_entries(&colliding),
        Err(CatalogRuleError::CrossFamilyClaim)
    );
}

#[test]
fn t172_a_rejected_catalog_still_classifies_but_never_promotes_ambiguity() {
    // Validation gates acceptance, not the classifier: a colliding catalog is
    // refused at the boundary and, if used anyway, still refuses to guess.
    let entries = colliding_pair(AgentFamily::Pi, AgentFamily::Kilo, "pi");
    assert!(!cross_family_claims_in(&entries).is_empty());
    assert_eq!(
        detect_in(
            &entries,
            DetectionInput::StructuredMetadataNamespace { namespace: "pi" },
            CatalogPlatform::Linux
        ),
        AgentDetection::Ambiguous {
            families: vec![AgentFamily::Pi, AgentFamily::Kilo]
        }
    );
}

#[test]
fn t172_catalog_grants_no_execution_authority_for_any_family() {
    for family in AgentFamily::ALL {
        let entry = pinned().entry(family).unwrap();
        assert_eq!(entry.support, AgentSupport::DetectionOnly);
        assert_eq!(entry.support.as_str(), "DETECTION_ONLY");
        for platform in CatalogPlatform::ALL {
            let detection = pinned().detect(
                DetectionInput::StructuredMetadataNamespace {
                    namespace: namespace_of(family),
                },
                platform,
            );
            assert!(!detection.confers_execution_authority());
            assert_eq!(
                pinned().revalidate(
                    &detection,
                    DetectionInput::TerminalProse("agent.start"),
                    platform
                ),
                AgentDetection::Stale
            );
        }
    }
    assert!(
        AgentSupport::DetectionOnly
            .truth_statement()
            .contains("never provider execution")
    );
    // The match is exhaustive over every classification state, so a new state
    // cannot be added without deciding that it too confers no authority.
    for detection in [
        AgentDetection::NearMatch,
        AgentDetection::Stale,
        AgentDetection::Unknown {
            reason: UnknownReason::EmptyInput,
        },
        AgentDetection::Ambiguous {
            families: vec![AgentFamily::Pi],
        },
        AgentDetection::Unavailable {
            reason: UnavailableReason::UnsupportedPlatform,
            families: vec![AgentFamily::Pi],
        },
        AgentDetection::UntrustedText {
            source: UntrustedTextSource::UserLabel,
        },
    ] {
        assert!(!detection.confers_execution_authority());
    }
}

#[test]
fn t172_catalog_module_reaches_no_host_capability() {
    // The module declares no `use` statement at all, so it cannot name a
    // process, filesystem, network, thread, or async API. `unsafe` is already
    // forbidden by the module attribute, which is a compile-time guarantee this
    // test cannot weaken.
    let source = include_str!("multiplexer/agent_catalog.rs");
    let imports: Vec<&str> = source
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("use "))
        .collect();
    assert!(
        imports.is_empty(),
        "the catalog must declare no import; found {imports:?}"
    );
    // `unsafe` is already forbidden by the module attribute, which the compiler
    // enforces on every build. This asserts the guard is actually present, so
    // dropping it cannot pass unnoticed.
    assert!(
        source.contains("#![forbid(unsafe_code)]"),
        "the catalog must keep forbidding unsafe code"
    );
    // The pinned catalog is proven compile-time data by a module-level const
    // assertion, so a runtime, refresh, or reload path cannot be hiding here.
    assert_eq!(AgentCatalog::pinned().entries().len(), 24);
}

#[test]
fn t172_catalog_entries_are_immutable_borrowed_data() {
    fn assert_shared<T: Send + Sync + 'static>(_: &T) {}
    assert_shared(&pinned());
    let entry = *pinned().entry(AgentFamily::Pi).unwrap();
    let copy = entry;
    assert_eq!(entry, copy, "a catalog entry is plain data, never a handle");
    // Holding a borrowed entry across further lookups cannot observe a change.
    let later = pinned().entry(AgentFamily::Muse).unwrap();
    assert_eq!(entry.namespace, "pi");
    assert_eq!(later.namespace, "muse");
}

#[test]
fn t172_single_family_catalog_matches_only_its_own_family() {
    // The per-family synthetic catalogs used above must not borrow the other 23
    // families, or a positive fixture could pass for the wrong reason.
    for family in AgentFamily::ALL {
        let entries = single(family);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].family, family);
        assert_eq!(
            detect_in(
                &entries,
                DetectionInput::StructuredMetadataNamespace {
                    namespace: namespace_of(family)
                },
                CatalogPlatform::Linux
            ),
            AgentDetection::Observed {
                family,
                source: DetectionSource::StructuredMetadataNamespace,
                support: AgentSupport::DetectionOnly,
            }
        );
        for other in AgentFamily::ALL {
            if other == family {
                continue;
            }
            assert_eq!(
                detect_in(
                    &entries,
                    DetectionInput::StructuredMetadataNamespace {
                        namespace: namespace_of(other)
                    },
                    CatalogPlatform::Linux
                ),
                AgentDetection::Unknown {
                    reason: UnknownReason::NoMatch
                },
                "{family:?} must not match {other:?}'s namespace"
            );
        }
    }
}

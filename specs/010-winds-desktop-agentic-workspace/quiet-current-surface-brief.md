# Spec 010 — Quiet Current Surface Brief

Status: T129 static presentation candidate.

This brief binds the first Winds desktop visual anatomy to Spec 010 without creating live Store, Git, PTY, workflow, provider, filesystem, or verification authority.

## Shell regions

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ quiet top chrome                                                            │
├───────────────┬──────────────────────────────────────────────┬───────────────┤
│ left dock     │ center workspace                             │ right dock    │
│               │                                              │               │
│ Projects      │ Session pane A │ Session pane B              │ Files         │
│ Sessions      │ Work Stream    │ Work Stream                 │ Changes       │
│ runtime marks │ Composer       │ Composer                    │ Evidence      │
│               │                                              │ Context       │
│               │                                              │ Artifacts     │
├───────────────┴──────────────────────────────────────────────┴───────────────┤
│ quiet status rail                                                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Left dock

The left dock is organizational presentation. Fixture Projects demonstrate nested Sessions, rename-friendly labels, pinned/attention states, and runtime marks. A Project is not a second canonical Workspace model, and a Session label never rewrites canonical identity.

Session rows are compact and information-dense. Runtime marks are Winds-authored and accessible. Codex/Claude treatments are textual monograms rather than copied proprietary logo assets.

## Center workspace

The center demonstrates the intended one-or-two Session mental model. Two panes have independent headers, focus treatment, Work Streams, and composers. There is no broadcast composer and no shared input surface. Static fixture events mix agent prose, command/result blocks, and attention state without using oversized chat bubbles.

## Right dock

The right dock makes Files primary while keeping Changes, Evidence, Context, and Artifacts adjacent. T129 fixture content is inert. Later tasks must identity-bind all asynchronous projections and reject stale results before render.

## Theme and accessibility fixtures

The static shell must render from the same component tree under deterministic dark, light, and high-contrast tokens. Keyboard focus remains visible. Reduced motion is honored. A 125% scaled-text fixture must retain core actions without clipping.

## Runtime identity fixture states

```text
CODEX
CLAUDE
SHELL
UNKNOWN
UNAVAILABLE
CONFLICTING
STALE
```

Presentation state is not evidence. A future runtime projection must carry source-labelled identity and mismatch semantics from Rust-owned authority.

## Anti-clone boundary

The shell intentionally avoids a browser-tab metaphor, VS Code activity-bar clone, terminal multiplexer chrome, Claude/Codex proprietary marks, dashboard card grids, glass panels, AI-purple gradients, and neon terminal styling. Reference products set a quality floor only.

## T129 nonclaims

- no live Project/Session projection;
- no presentation persistence;
- no Tauri product command;
- no filesystem access;
- no Git access;
- no PTY input or rendering;
- no provider/model access;
- no runtime identity proof;
- no workflow action authority;
- no evidence verification authority.

# Winds Current Spectrum Design System

Status: T142A presentation identity candidate built on the canonically closed T129–T142 desktop foundation.

Current Spectrum is the Winds desktop visual language. It is designed for long-running, high-attention coding-agent work: compact enough for engineering reality, energetic enough to have a recognizable Winds identity, and precise enough that presentation never obscures authority.

## Principles

- **IDE anatomy before dashboard composition.** A narrow activity rail, bounded tool windows, central Workbench, contextual Inspector, and compact status rail define the shell.
- **Chat is a tool window, not the whole product.** The selected exact Session conversation belongs at left; execution belongs in the Workbench.
- **Quiet foundations, spectral identity.** Deep graphite/ink surfaces carry most of the interface; vivid spectrum colors are sparse identity and focus cues.
- **Information before decoration.** Every visible accent helps identify focus, navigation, runtime, state, or action.
- **Compact-professional density.** Controls are small enough for serious desktop work without becoming cramped.
- **Typography carries hierarchy.** Use UI sans for navigation, labels, and prose; use monospace for terminal, code, and exact identities only.
- **State never depends on color alone.** Pair color with text, symbols, position, or shape.
- **Motion explains continuity.** Default transitions are short and disappear under reduced-motion preferences.
- **Independent identity.** External IDEs are quality references only. Winds owns its geometry, tokens, mark, spacing, typography, composition, and visual voice.

## Foundations

### Color

Dark is the default first-program presentation. Light and high-contrast remain first-class deterministic themes.

Dark Current Spectrum foundation:

```text
canvas            #090A0F
surface-1         #101117
surface-2         #151721
surface-3         #1B1D29
border            #2D3040
text              #F5F6FB
text-muted        #B5B8C8
accent            #AAB0FF
spectrum-violet   #A78BFA
spectrum-azure    #63B3FF
spectrum-magenta  #E879F9
spectrum-coral    #FB7185
```

Light keeps neutral paper/graphite surfaces with darker accessible spectrum equivalents. High contrast uses black/white foundations, stronger borders, and system-safe focus colors. Normal text/status pairs continue to meet WCAG 2.2 AA against intended surfaces.

Spectrum colors are not status authority. Success, warning, danger, and runtime-proof semantics retain separate tokens.

### Current Mark

The Winds Current Mark is three authored flowing current lines. The geometry is deliberately non-letterform and does not reproduce a vendor monogram, app tile, or logo. Its lines may use separate Current Spectrum accents; no gradient is required.

### Type

Primary UI stack: system sans (`Inter` when already available, then platform system fonts). Terminal/code/identity stack: platform monospace. Controls do not use decorative display faces.

```text
9–10px  provenance and compact metadata
11–12px navigation and work labels
13px    default UI text
14px    emphasized UI text
16px    major Session/workspace title moments only
```

### Spacing and radius

The base unit is 4 px. Primary steps remain 4, 6, 8, 10, 12, 16, 20, 24, and 32 px. Use 4–8 px radii. The application shell is planar; radius appears on controls, selected rows, inputs, and bounded transient surfaces only.

### Elevation

Depth is rare. Prefer border plus tonal separation. Focused surfaces may use a restrained inset structural cue; no glass layer, glow halo, or floating dashboard card treatment is part of Current Spectrum.

### Focus

Every keyboard-reachable action receives a visible 2 px focus ring with offset. Focus is visible in dark, light, and high-contrast themes and is never replaced by hover styling.

## Desktop anatomy

### Top chrome

A compact chrome row shows Winds identity, current repository presentation context, and bounded global controls. It does not imitate browser tabs or competitor title chrome.

### Activity rail

Nominal width: 44 px, 40 px at compact density. The far-left rail contains the Current Mark and Winds-owned tool glyph geometry. Chat and Projects are explicit sibling tool-window actions. Selection combines shape, fill, and a narrow structural spectrum indicator rather than color alone.

### Left Chat tool window

Nominal width: 276 px at full density. Chat is the default wide/standard desktop tool window. It shows:

- exact selected canonical Session and Project ownership;
- runtime mark and proof wording;
- bounded conversation/work-stream projection;
- truthful composer availability and exact target identity;
- explicit presentation/trust boundary language where the source is fixture-only.

A disabled composer remains visibly disabled when direct runtime input is unavailable. Chat never invents dispatch authority.

### Left Projects tool window

Projects is a sibling tool window reached from the activity rail. It preserves canonical Project/Session search, selection, presentation mutation, creation, attention, and keyboard navigation behavior. Selecting an active Session changes only the requested exact Session target and returns presentation focus to Chat.

### Center Workbench

The center is an operational Workbench for one or two exact Session identities. Terminal opens as the primary sub-surface. Activity remains available for user-visible work events and exact Files/Changes intents. The Workbench does not render the primary Chat composer, preventing a second competing conversation surface.

Dual layout retains exact identities, independent focus, split persistence, maximize/restore, shared-worktree warning, and narrow single-focused-Slot fallback. Dual view never means broadcast.

### Contextual Inspector

Nominal width: 284 px. Files, Changes, Evidence, Context, Artifacts, and Needs You stay adjacent contextual surfaces bound to the focused exact Session. Visual prominence never upgrades evidence or approval authority.

### Status rail

The bottom rail remains compact and explicitly presentation-oriented. Current Spectrum identity text is non-authoritative.

## State grammar

Primary primitives visibly support default, hover, focus-visible, active/selected, disabled, loading, error, empty, and attention states.

- Hover changes tone/border, never geometry.
- Selected rows combine tonal fill, text emphasis, and a narrow structural indicator.
- Disabled controls reduce contrast but remain readable.
- Loading uses bounded text/skeleton rhythm rather than decorative indefinite motion.
- Errors pair danger color with explicit text.
- Empty states explain the missing object and next valid action without illustration-heavy filler.

## Motion

Default transition budget remains 150–220 ms. Motion is reserved for tool-window state, pane focus, single/dual transitions, contextual reveal, and attention continuity. `prefers-reduced-motion: reduce` removes non-essential motion while preserving state cues.

## Responsive desktop behavior

- `>= 1500px`: activity rail + full left tool window + dual Workbench + full Inspector.
- `1280–1499px`: activity rail + compact left/right widths + dual Workbench.
- `1080–1279px`: Inspector may collapse while rail, left tool window, and Workbench remain.
- `< 1080px`: focused single-Slot Workbench fallback preserves both canonical identities and offers explicit focus switching.
- 200% text scaling follows the same structural fallback rather than squeezing unreadable dual panes.

Desktop is the target. No mobile-web layout is claimed.

## Deterministic visual fixtures

The retained visual matrix still covers 1280/1440/1920 dark, light, high contrast, keyboard focus, 125% and 200% scaling, reduced motion, compact density, and narrow layout. T142A adds explicit Current Spectrum fixtures for:

```text
1440x900  dark   Chat tool window
1440x900  dark   Projects tool window
1440x900  light  Chat tool window
1440x900  light  Projects tool window
```

These fixtures are presentation-only. Screenshot evidence must be captured from the exact candidate when required; a hand-authored mockup is never equivalent to the app output.

## Prohibited visual patterns

Do not introduce gradient text, glassmorphism/backdrop blur, neon terminal glow, fake AI aura, dashboard card grids, oversized rounded chat bubbles, copied competitor layout/branding assets, exact competitor color sampling, proprietary typography/icons, or vendor logo files without separate qualification.

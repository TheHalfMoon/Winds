# Winds Quiet Current Design System

Status: T129 static visual system candidate.

Quiet Current is the Winds desktop visual language. It is designed for long-running, high-attention agentic work: calm enough for hours of use, dense enough for engineering reality, and precise enough that state changes are obvious without theatrical effects.

## Principles

- **Quiet surfaces, explicit structure.** Use tonal steps and separators before elevation.
- **Information before decoration.** Every visible accent must help identify focus, state, runtime, or action.
- **Compact-professional density.** Controls are small enough for serious desktop work without becoming cramped.
- **Typography carries hierarchy.** Use a primary UI sans for navigation, labels, and prose; use monospace for terminal/code/identities only.
- **State never depends on color alone.** Pair color with text, symbols, position, or shape.
- **Motion explains continuity.** Default transitions are 150–220 ms and disappear under reduced-motion preferences.
- **No cloned identity.** Herdr, Codex, Claude, and Impeccable are quality/process references only. Winds owns its geometry, tokens, marks, spacing, and visual voice.

## Foundations

### Color

Dark is the default first-program presentation; light and high-contrast are first-class, deterministic themes.

Dark foundation:

```text
canvas      #0B0D0E
surface-1   #101315
surface-2   #15191B
surface-3   #1C2124
border      #293034
text        #F4F6F7
text-muted  #AAB2B6
accent      #77A7FF
success     #71D39A
warning     #F0C674
danger      #FF8A8A
```

Light foundation:

```text
canvas      #F6F7F7
surface-1   #FFFFFF
surface-2   #EFF1F1
surface-3   #E6E9EA
border      #D6DBDD
text        #151719
text-muted  #5F676C
accent      #245BDB
success     #1B6B3A
warning     #7B5500
danger      #A52A2A
```

The primary/muted/accent/status foreground pairs are selected to meet WCAG 2.2 AA against their intended foundations. High-contrast uses black/white foundations, stronger borders, and system-safe focus colors.

### Type

Primary UI stack: system sans (`Inter` when already available, then platform system fonts). Terminal/code/identity stack: platform monospace. Controls never use decorative display faces.

Scale:

```text
11px micro metadata
12px compact labels
13px default UI text
14px emphasized UI text
16px section/Session titles
20px product/workspace title moments only
```

### Spacing

The base unit is 4 px. Primary steps: 4, 6, 8, 10, 12, 16, 20, 24, 32. Dock rows normally use 28–34 px heights; primary controls normally use 30–34 px heights.

### Radius

Use 4–8 px radii. Large floating-card radii are not part of Quiet Current. The application shell itself is planar; radius appears on controls, selected rows, inputs, and bounded transient surfaces only.

### Elevation

Depth is rare. Prefer border + tonal separation. The static shell uses one subtle inset/top-layer shadow only where a focused surface must lift from neighboring work.

### Focus

Every keyboard-reachable action receives a 2 px focus ring with a 2 px offset. Focus is visible in dark, light, and contrast themes and is not replaced by hover styling.

## Desktop anatomy

### Top chrome

A 42 px quiet chrome row shows Winds identity, current Project/repository presentation context, and bounded global controls. It must not resemble browser tabs or a terminal title bar.

### Left dock

Nominal width: 236 px. It owns Project navigation and nested Session rows. Sessions include a Winds-authored runtime mark, renameable presentation label, and quiet status text. Pinned/active/attention states remain distinguishable without large badges.

### Center workspace

The center is the product. At wide/standard desktop sizes it can host two independent Session panes with a 1 px divider. Each pane has a compact header, Work Stream, and anchored composer. Work events are rows/blocks, not oversized chat bubbles.

### Right dock

Nominal width: 284 px. Files is the first tab. Changes, Evidence, Context, and Artifacts remain adjacent contextual surfaces. Tree density is compact and selected-file styling is quiet.

### Status rail

A 24 px bottom rail shows static fixture context in T129. Later tasks may bind authoritative data; T129 must not imply it is live.

## Runtime marks

Runtime identity uses Winds-authored marks:

```text
Codex       CX
Claude      CL
Shell       $
Unknown     ?
Unavailable —
Conflicting !!
Stale       ·
```

Marks always have accessible text. They are presentation labels only and never prove runtime identity.

## State grammar

Primary primitives must visibly support: default, hover, focus-visible, active/selected, disabled, loading, error, empty, and attention states.

- Hover changes tone/border, never geometry.
- Selected rows combine tonal fill, text emphasis, and a narrow structural indicator.
- Disabled controls reduce contrast but remain readable.
- Loading uses text/skeleton rhythm; avoid infinite decorative spinners where a bounded state label works.
- Error states pair danger color with explicit text/iconography.
- Empty states explain the missing object and the next valid action without illustration-heavy filler.

## Motion

Default transition budget: 150–220 ms. Use motion for dock state, pane focus, single/dual transitions, contextual reveal, and attention transitions only. `prefers-reduced-motion: reduce` removes non-essential transitions/animation and preserves equivalent state cues.

## Responsive desktop behavior

- `>= 1500px`: full left dock + dual center + full right dock.
- `1280–1499px`: compact left/right docks + dual center.
- `1080–1279px`: right dock may collapse; dual panes keep minimum readable widths.
- `< 1080px`: single Session center is the safe fallback; dual mode must not squeeze unreadable panes.

Desktop is the target. No mobile-web layout is claimed.

## Deterministic visual fixtures

T129 defines fixture modes for:

```text
1280x800   dark
1440x900   dark
1920x1080  dark
1440x900   light
1440x900   contrast
1440x900   keyboard-focus
1440x900   scaled-text-125
```

These fixtures are presentation-only. Screenshot capture evidence must be generated from the exact candidate head; hand-authored mockups do not substitute for captured app output.

## Prohibited visual patterns

Do not introduce gradient text, decorative glass/backdrop blur, neon terminal glow, fake AI aura, dashboard card grids, oversized rounded chat bubbles, copied competitor layout/branding assets, or vendor logo files without separate qualification.

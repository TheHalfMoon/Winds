import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const repositoryRoot = join(root, "..");
const tokens = readFileSync(join(root, "src/tokens.css"), "utf8");
const fixtureModes = readFileSync(join(root, "src/fixture-modes.css"), "utf8");
const main = readFileSync(join(root, "src/main.tsx"), "utf8");
const rightDock = readFileSync(join(root, "src/rightDock/RightDock.tsx"), "utf8");
const sessionSurface = readFileSync(join(root, "src/sessionSurface/SessionSurface.tsx"), "utf8");
const leftDock = readFileSync(join(root, "src/leftDock/LeftDock.tsx"), "utf8");
const dualSession = readFileSync(join(root, "src/dualSession/DualSessionWorkspace.tsx"), "utf8");
const commandPalette = readFileSync(join(root, "src/commandPalette/CommandPalette.tsx"), "utf8");
const tauriHost = readFileSync(join(root, "src-tauri/src/main.rs"), "utf8");
const fixtureManifest = JSON.parse(readFileSync(join(root, "tests/fixtures/visual-fixtures.json"), "utf8"));

function themeVariables(marker) {
  const start = tokens.indexOf(marker);
  assert.notEqual(start, -1, `missing theme marker ${marker}`);
  const open = tokens.indexOf("{", start);
  const end = tokens.indexOf("\n}", open);
  assert.ok(open > start && end > open, `malformed theme block ${marker}`);
  const variables = {};
  for (const match of tokens.slice(open + 1, end).matchAll(/--([a-z0-9-]+):\s*(#[0-9a-f]{6});/gi)) {
    variables[match[1]] = match[2].toLowerCase();
  }
  return variables;
}

function channelLuminance(value) {
  const normalized = value / 255;
  return normalized <= 0.04045
    ? normalized / 12.92
    : ((normalized + 0.055) / 1.055) ** 2.4;
}

function luminance(hex) {
  const channels = [1, 3, 5].map((offset) => Number.parseInt(hex.slice(offset, offset + 2), 16));
  const [red, green, blue] = channels.map(channelLuminance);
  return 0.2126 * red + 0.7152 * green + 0.0722 * blue;
}

function contrastRatio(first, second) {
  const brighter = Math.max(luminance(first), luminance(second));
  const darker = Math.min(luminance(first), luminance(second));
  return (brighter + 0.05) / (darker + 0.05);
}

const normalTextTokens = [
  "text", "text-muted", "text-faint", "accent", "success", "warning", "danger",
  "runtime-codex", "runtime-claude", "runtime-shell", "runtime-muted",
];
const surfaceTokens = ["canvas", "surface-1", "surface-2", "surface-3"];

test("T140 dark light and contrast normal-text tokens meet WCAG AA", () => {
  for (const marker of [':root[data-theme="dark"]', ':root[data-theme="light"]', ':root[data-theme="contrast"]']) {
    const variables = themeVariables(marker);
    for (const foregroundName of normalTextTokens) {
      for (const backgroundName of surfaceTokens) {
        const ratio = contrastRatio(variables[foregroundName], variables[backgroundName]);
        assert.ok(ratio >= 4.5, `${marker} ${foregroundName}/${backgroundName} ratio ${ratio.toFixed(2)}`);
      }
    }
  }
});

test("T140 deterministic appearance fixtures cover system light compact reduced motion 200 percent and narrow", () => {
  assert.match(tokens, /prefers-color-scheme: light/);
  assert.match(tokens, /data-density="compact"/);
  assert.match(tokens, /data-motion="reduced"/);
  assert.match(main, /delete documentRoot\.dataset\.theme/);
  assert.match(main, /scale === "125" \|\| scale === "200"/);
  assert.match(main, /dataset\.motion = "reduced"/);
  assert.match(main, /dataset\.density = "compact"/);
  assert.match(main, /dataset\.layout = "narrow"/);
  assert.match(fixtureModes, /data-scale="200"/);
  assert.match(fixtureModes, /:root\[data-scale="200"\] \.narrow-dual-fallback/);
  assert.match(fixtureModes, /:root\[data-scale="200"\] \.dual-session-layout\[data-mode="DUAL"\]/);
  assert.match(fixtureModes, /:root\[data-scale="200"\][\s\S]*\.session-slot\[data-focused="false"\]/);
  assert.match(fixtureModes, /data-layout="narrow"/);
  assert.match(fixtureModes, /data-motion="reduced"/);

  const queries = new Set(fixtureManifest.fixtures.map(({ query }) => query));
  for (const query of [
    "?theme=light",
    "?theme=contrast",
    "?theme=dark&motion=reduced",
    "?theme=dark&density=compact",
    "?theme=dark&scale=200",
    "?theme=dark&layout=narrow",
  ]) assert.equal(queries.has(query), true, query);
});

test("T140 right dock implements an accessible roving tab contract", () => {
  for (const value of [
    'role="tablist"', 'role="tab"', "aria-controls=\"right-dock-panel\"",
    "aria-selected={surface === tab}", "tabIndex={surface === tab ? 0 : -1}",
    'role="tabpanel"', "aria-labelledby={`right-dock-tab-${surface}`}",
    'event.key === "ArrowRight"', 'event.key === "ArrowLeft"',
    'event.key === "Home"', 'event.key === "End"',
  ]) assert.equal(rightDock.includes(value), true, value);
});

test("T140 primary Session controls expose keyboard selected state and preserve IME safety", () => {
  assert.match(sessionSurface, /role="group" aria-label="Session surface view"/);
  assert.match(sessionSurface, /aria-pressed=\{surface === "work_stream"\}/);
  assert.match(sessionSurface, /aria-pressed=\{surface === "terminal"\}/);
  assert.match(sessionSurface, /event\.nativeEvent\.isComposing/);
  assert.match(sessionSurface, /shouldSubmitComposerShortcut/);
  assert.match(sessionSurface, /metaKey: event\.metaKey/);
  assert.match(sessionSurface, /ctrlKey: event\.ctrlKey/);
  assert.match(sessionSurface, /event\.key === "ArrowRight"/);
  assert.match(sessionSurface, /event\.key === "ArrowLeft"/);
});

test("T140 screen reader structure preserves Project Session runtime attention and focus truth", () => {
  assert.match(leftDock, /aria-current=\{selected \? "page" : undefined\}/);
  assert.match(leftDock, /\$\{runtime\.label\}/);
  assert.match(leftDock, /\$\{attention\.label\}/);
  assert.match(leftDock, /Project\. \$\{current\.sessionCount\} Sessions/);
  assert.match(dualSession, /aria-current=\{focusedSlot === slot \? "true" : undefined\}/);
  assert.match(commandPalette, /role="combobox"/);
  assert.match(commandPalette, /aria-activedescendant/);
  assert.match(commandPalette, /role="listbox"/);
  assert.match(commandPalette, /role="option"/);
  assert.match(commandPalette, /tabIndex=\{-1\}/);
});

test("T140 keeps keyboard and pointer routes paired without adding runtime authority", () => {
  assert.match(dualSession, /onFocusCapture=\{\(\) => setFocusedSlot\(slot\)\}/);
  assert.match(dualSession, /onPointerDown=\{\(\) => setFocusedSlot\(slot\)\}/);
  assert.match(rightDock, /onClick=\{\(\) => setSurface\(tab\)\}/);
  for (const forbidden of ["codex_launch", "claude_launch", "runtime_start", "runtime_input"]) {
    assert.equal(tauriHost.includes(forbidden), false, forbidden);
  }
  assert.equal(readFileSync(join(repositoryRoot, "desktop/package-lock.json"), "utf8").length > 0, true);
});

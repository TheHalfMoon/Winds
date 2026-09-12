import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import test from "node:test";

const root = new URL("..", import.meta.url).pathname;
const app = readFileSync(join(root, "src/App.tsx"), "utf8");
const main = readFileSync(join(root, "src/main.tsx"), "utf8");
const styles = readFileSync(join(root, "src/styles.css"), "utf8");
const tokens = readFileSync(join(root, "src/tokens.css"), "utf8");
const fixtureModes = readFileSync(join(root, "src/fixture-modes.css"), "utf8");
const runtimeMark = readFileSync(join(root, "src/components/RuntimeMark.tsx"), "utf8");
const fixtureManifest = JSON.parse(
  readFileSync(join(root, "tests/fixtures/visual-fixtures.json"), "utf8"),
);

const requiredTokens = [
  "--canvas",
  "--surface-1",
  "--surface-2",
  "--border",
  "--text",
  "--text-muted",
  "--accent",
  "--success",
  "--warning",
  "--danger",
  "--focus",
  "--left-dock-width",
  "--right-dock-width",
  "--pane-min-width",
];

test("T129 defines dark, light, and high-contrast Quiet Current foundations", () => {
  for (const token of requiredTokens) assert.equal(tokens.includes(token), true, token);
  assert.match(tokens, /:root\[data-theme="light"\]/);
  assert.match(tokens, /:root\[data-theme="contrast"\]/);
  assert.match(tokens, /prefers-contrast: more/);
  assert.match(tokens, /prefers-reduced-motion: reduce/);
});

test("T129 shell exposes left dock, dual independent Sessions, and Files-first right dock", () => {
  const required = [
    "Projects and Sessions",
    "Dual Session view",
    "Context dock",
    "Work Stream",
    "Terminal",
    "Files",
    "Changes",
    "Evidence",
    "Context",
    "Artifacts",
    "composer",
    "Rename",
  ];
  for (const value of required) assert.equal(app.includes(value), true, value);
  assert.equal((app.match(/<SessionPane/g) ?? []).length, 2);
});

test("T129 runtime identity uses Winds-authored accessible marks", () => {
  for (const mark of ['codex: "CX"', 'claude: "CL"', 'shell: "$"']) {
    assert.equal(runtimeMark.includes(mark), true, mark);
  }
  for (const state of ["unknown", "unavailable", "conflicting", "stale"]) {
    assert.equal(runtimeMark.includes(`${state}:`), true, state);
  }
  assert.match(runtimeMark, /aria-label=\{accessible\}/);
});

test("T129 visual grammar rejects clone-prone decorative patterns", () => {
  const combined = `${styles}\n${tokens}\n${fixtureModes}`.toLowerCase();
  const forbidden = [
    "linear-gradient(",
    "radial-gradient(",
    "conic-gradient(",
    "backdrop-filter",
    "text-shadow:",
    "filter: blur(",
  ];
  for (const value of forbidden) assert.equal(combined.includes(value), false, value);
});

test("T129 primary controls have explicit focus, hover, active, disabled, loading, error, and empty states", () => {
  const required = [
    ":focus-visible",
    ":hover",
    ":active",
    ":disabled",
    '[data-state="loading"]',
    '[data-state="error"]',
    ".empty-state",
  ];
  for (const value of required) assert.equal(styles.includes(value), true, value);
});

test("T129 focus and 125% scale fixtures create observable rendered differences", () => {
  assert.match(main, /dataset\.fixture = "keyboard-focus"/);
  assert.match(main, /dataset\.scale = "125"/);
  assert.match(fixtureModes, /data-fixture="keyboard-focus"/);
  assert.match(fixtureModes, /outline: 2px solid var\(--focus\)/);
  assert.match(fixtureModes, /data-scale="125"/);
  assert.match(fixtureModes, /transform: scale\(1\.25\)/);
});

test("T129 fixture matrix contains the required viewport and accessibility surfaces", () => {
  const fixtures = fixtureManifest.fixtures;
  assert.equal(fixtureManifest.schema, "winds-quiet-current-visual-fixtures/1");
  assert.equal(fixtures.length, 7);
  assert.deepEqual(
    fixtures.slice(0, 3).map(({ width, height }) => [width, height]),
    [
      [1280, 800],
      [1440, 900],
      [1920, 1080],
    ],
  );
  const queries = fixtures.map(({ query }) => query);
  assert.equal(queries.includes("?theme=light"), true);
  assert.equal(queries.includes("?theme=contrast"), true);
  assert.equal(queries.some((query) => query.includes("keyboard-focus")), true);
  assert.equal(queries.some((query) => query.includes("scale=125")), true);
});

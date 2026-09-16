import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const app = readFileSync(join(root, "src/App.tsx"), "utf8");
const rail = readFileSync(join(root, "src/activityRail/ActivityRail.tsx"), "utf8");
const mark = readFileSync(join(root, "src/components/CurrentMark.tsx"), "utf8");
const chat = readFileSync(join(root, "src/leftDock/ChatToolWindow.tsx"), "utf8");
const workbench = readFileSync(join(root, "src/dualSession/DualSessionWorkspace.tsx"), "utf8");
const sessionSurface = readFileSync(join(root, "src/sessionSurface/SessionSurface.tsx"), "utf8");
const main = readFileSync(join(root, "src/main.tsx"), "utf8");
const tokens = readFileSync(join(root, "src/tokens.css"), "utf8");
const styles = readFileSync(join(root, "src/styles.css"), "utf8");
const manifest = JSON.parse(readFileSync(join(root, "tests/fixtures/visual-fixtures.json"), "utf8"));

const productSurface = `${app}\n${rail}\n${chat}\n${workbench}\n${sessionSurface}`;

test("T142A makes Chat the default left tool window and Projects an explicit sibling", () => {
  assert.match(app, /dataset\.leftTool === "projects" \? "projects" : "chat"/);
  assert.match(app, /<ActivityRail active=\{leftTool\}/);
  assert.match(app, /leftTool === "chat"/);
  assert.match(app, /<ChatToolWindow selection=\{selection\}/);
  assert.match(app, /<LeftDock onSelectSession=\{selectProjectSession\}/);
  assert.match(rail, /aria-label="Open Chat tool window"/);
  assert.match(rail, /aria-label="Open Projects tool window"/);
  assert.ok(app.indexOf("<ActivityRail") < app.indexOf("<ChatToolWindow"));
});

test("T142A left Chat stays exact-target and truthful about composer authority", () => {
  assert.match(chat, /canonicalWorkspaceId === selection\.workspaceId/);
  assert.match(chat, /canonicalSessionId === selection\.sessionId/);
  assert.match(chat, /composerAvailability\(surface\)/);
  assert.match(chat, /disabled=\{!availability\?\.enabled\}/);
  assert.match(chat, /Exact Session projection/);
  assert.equal(chat.includes("broadcast"), false);
  assert.equal(chat.includes("@tauri-apps/api/core"), false);
  assert.equal(chat.includes("invoke("), false);
});

test("T142A center is a Workbench and no longer duplicates the primary composer", () => {
  assert.match(app, /aria-label="Workbench"/);
  assert.match(workbench, /mode="workbench"/);
  assert.match(workbench, /Dual Session Workbench/);
  assert.match(sessionSurface, /mode === "workbench" \? "terminal" : "work_stream"/);
  assert.match(sessionSurface, /hidden=\{surface !== "work_stream" \|\| mode === "workbench"\}/);
});

test("T142A Current Spectrum is Winds-authored and keeps semantic status separate", () => {
  for (const token of ["--spectrum-violet", "--spectrum-azure", "--spectrum-magenta", "--spectrum-coral", "--activity-rail-width"]) {
    assert.equal(tokens.includes(token), true, token);
  }
  assert.match(mark, /Winds Current Mark/);
  assert.equal((mark.match(/<path/g) ?? []).length, 3);
  const lower = `${mark}\n${styles}`.toLowerCase();
  for (const forbidden of ["linear-gradient(", "radial-gradient(", "conic-gradient(", "backdrop-filter", "text-shadow:"]) {
    assert.equal(lower.includes(forbidden), false, forbidden);
  }
  for (const semantic of ["--success", "--warning", "--danger"]) assert.equal(tokens.includes(semantic), true, semantic);
});

test("T142A deterministic fixtures cover dark light Chat and Projects states", () => {
  assert.match(main, /dataset\.leftTool = leftTool/);
  const queries = new Set(manifest.fixtures.map(({ query }) => query));
  for (const query of [
    "?theme=dark&tool=chat",
    "?theme=dark&tool=projects",
    "?theme=light&tool=chat",
    "?theme=light&tool=projects",
  ]) assert.equal(queries.has(query), true, query);
});

test("T142A preserves exact Session focus and contextual Inspector binding", () => {
  assert.match(app, /setSelection\(\(current\)/);
  assert.match(app, /setDockTarget\(\(current\)/);
  assert.match(app, /onFocusSession=\{focusDock\}/);
  assert.match(app, /onFocusAttention=/);
  assert.match(productSurface, /exact Session/i);
});

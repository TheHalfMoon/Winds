import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = path.resolve(root, "..");

function read(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}

function readRepo(relative) {
  return fs.readFileSync(path.join(repoRoot, relative), "utf8");
}

test("T174 agent dock keeps exact identity and detection-only authority", () => {
  const component = read("src/agentDock/AgentDock.tsx");
  const model = read("src/agentDock/model.ts");

  assert.match(component, /observation\.observationId/);
  assert.match(component, /observation\.multiplexerWorkspaceId/);
  assert.match(component, /observation\.tabId/);
  assert.match(component, /observation\.paneId/);
  assert.match(component, /topologyBridge\(\)/);
  assert.match(component, /bindTarget\(topology\.topologyGeneration, target\)/);
  assert.match(component, /presentation only · no controller\/write authority/);
  assert.match(component, /Display alias only/);
  assert.match(model, /Detection only · execution unproven/);

  assert.doesNotMatch(component, /agent\.start|provider[_ -]?launch|request[_ -]?control|takeover/i);
  assert.doesNotMatch(model, /agent\.start|provider[_ -]?launch|request[_ -]?control|takeover/i);
});

test("T174 reuses the typed left-dock snapshot and adds no agent invoke command", () => {
  const leftDock = read("src/leftDock/LeftDock.tsx");
  const main = read("src-tauri/src/main.rs");
  const rustBridge = readRepo("src/desktop.rs");

  assert.match(leftDock, /<AgentDock snapshot=\{snapshot\.agentDock\}/);
  assert.match(rustBridge, /refresh_agent_observation_projection/);
  assert.match(rustBridge, /OWNER_AUTHORITATIVE_AGENT_OBSERVATIONS/);
  assert.match(rustBridge, /DETECTION_ONLY_UNPROVEN/);
  assert.doesNotMatch(main, /agent_dock_|agent_observation_snapshot/);
});

test("T174 aliasing is presentation-local and exact targeting ignores aliases", () => {
  const component = read("src/agentDock/AgentDock.tsx");
  const model = read("src/agentDock/model.ts");

  assert.match(component, /setAliases/);
  assert.match(component, /immutable identity unchanged/);
  assert.match(model, /multiplexerWorkspaceId: observation\.multiplexerWorkspaceId/);
  assert.match(model, /tabId: observation\.tabId/);
  assert.match(model, /paneId: observation\.paneId/);
  assert.doesNotMatch(model, /displayName.*paneId|alias.*paneId/i);
});

import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const bridge = await readFile(
  new URL("../src/multiplexer/bridge.ts", import.meta.url),
  "utf8",
);
const surface = await readFile(
  new URL("../src/multiplexer/TopologySurface.tsx", import.meta.url),
  "utf8",
);
const host = await readFile(
  new URL("../src-tauri/src/main.rs", import.meta.url),
  "utf8",
);
const rustBridge = await readFile(
  new URL("../../src/desktop_topology.rs", import.meta.url),
  "utf8",
);

test("T169 exposes only closed typed topology commands", () => {
  assert.match(bridge, /multiplexer_topology_capability/);
  assert.match(bridge, /multiplexer_topology_snapshot/);
  assert.match(bridge, /multiplexer_topology_bind_target/);
  assert.doesNotMatch(bridge, /invoke\([^"']/);
  assert.match(host, /multiplexer_topology_capability,/);
  assert.match(host, /multiplexer_topology_snapshot,/);
  assert.match(host, /multiplexer_topology_bind_target,/);
});

test("T169 keeps renderer away from owner transport and cache authority", () => {
  assert.doesNotMatch(surface, /UnixStream|NamedPipe|socket|sqlite|localStorage/i);
  assert.doesNotMatch(bridge, /UnixStream|NamedPipe|socket|sqlite|localStorage/i);
  assert.match(rustBridge, /RustLocalControlClient/);
  assert.match(rustBridge, /refresh_topology_projection/);
  assert.match(rustBridge, /authoritative generation changed/);
});

test("T169 recursive presentation binds immutable ids rather than labels", () => {
  assert.match(surface, /key=\{workspace\.multiplexerWorkspaceId\}/);
  assert.match(surface, /key=\{tab\.tabId\}/);
  assert.match(surface, /data-pane-id=\{node\.paneId\}/);
  assert.match(surface, /bindTopologyTarget/);
  assert.doesNotMatch(surface, /displayLabel.*===.*target/);
});

test("T169 preserves native keyboard, pointer, contrast, scale, and reduced-motion hooks", async () => {
  const css = await readFile(
    new URL("../src/multiplexer/topology.css", import.meta.url),
    "utf8",
  );
  assert.match(surface, /type="button"/);
  assert.match(surface, /onClick=/);
  assert.match(surface, /aria-current/);
  assert.match(surface, /aria-pressed/);
  assert.doesNotMatch(surface, /onKeyDown=/);
  assert.match(css, /data-theme="contrast"/);
  assert.match(css, /data-scale="125"/);
  assert.match(css, /data-scale="200"/);
  assert.match(css, /prefers-reduced-motion/);
  assert.match(css, /data-motion="reduced"/);
});

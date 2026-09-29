import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const repo = join(root, "..");

const bridge = await readFile(join(root, "src/multiplexer/bridge.ts"), "utf8");
const surface = await readFile(join(root, "src/multiplexer/TopologySurface.tsx"), "utf8");
const types = await readFile(join(root, "src/multiplexer/types.ts"), "utf8");
const host = readFileSync(join(root, "src-tauri/src/main.rs"), "utf8");
const rustBridge = readFileSync(join(repo, "src/desktop_topology.rs"), "utf8");

function rendererSources() {
  const sourceRoot = join(root, "src");
  const files = [];
  const collect = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) collect(path);
      else if (/\.(ts|tsx)$/.test(entry.name)) files.push(path);
    }
  };
  collect(sourceRoot);
  return files.map((path) => ({
    path: path.slice(sourceRoot.length + 1).replaceAll("\\", "/"),
    text: readFileSync(path, "utf8"),
  }));
}

const renderer = rendererSources();
const hostCommands = [
  ...(host.match(/tauri::generate_handler!\[([\s\S]*?)\]/) ?? ["", ""])[1]
    .split(",")
    .map((entry) => entry.trim())
    .filter(Boolean),
];

test("T169 exposes only closed typed topology commands", () => {
  assert.match(bridge, /"multiplexer_topology_capability"/);
  assert.match(bridge, /"multiplexer_topology_snapshot"/);
  assert.match(bridge, /"multiplexer_topology_bind_target"/);
  assert.doesNotMatch(bridge, /invoke\([^"']/);
  assert.match(host, /multiplexer_topology_capability,/);
  assert.match(host, /multiplexer_topology_snapshot,/);
  assert.match(host, /multiplexer_topology_bind_target,/);
  assert.equal(new Set(hostCommands).size, hostCommands.length);
});

test("T169 keeps renderer away from owner transport and cache authority", () => {
  assert.doesNotMatch(surface, /UnixStream|NamedPipe|socket|sqlite|localStorage/i);
  assert.doesNotMatch(bridge, /UnixStream|NamedPipe|socket|sqlite|localStorage/i);
  assert.match(rustBridge, /RustLocalControlClient/);
  assert.match(rustBridge, /refresh_topology_projection/);
  assert.match(rustBridge, /authoritative generation changed/);
});

test("T169 fails closed instead of projecting fixture topology outside the trusted host", () => {
  assert.match(bridge, /isTauri\(\) \? canonicalTopologyBridge : fixtureTopologyBridge/);
  assert.match(bridge, /VITE_WINDS_T143_BENCHMARK === "1"/);
  assert.match(bridge, /VITE_WINDS_T143_NATIVE_READY === "1"/);
  for (const fabricated of [
    "topology:",
    "workspaces:",
    "MultiplexerWorkspaceId",
    "tabId:",
    "paneId:",
  ]) {
    const fixtureSection = bridge.slice(bridge.indexOf("const fixtureTopologyBridge"));
    assert.doesNotMatch(fixtureSection, new RegExp(escapeRegExp(fabricated)), fabricated);
  }
});

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

test("T169 never lets the renderer supply or read the expected owner generation", () => {
  for (const source of [bridge, surface, types]) {
    assert.doesNotMatch(source, /ownerGenerationId/i);
    assert.doesNotMatch(source, /dataset\.|documentElement/);
  }
  assert.match(rustBridge, /latest_persistent_runtime_owner_generation/);
  assert.match(rustBridge, /RustLocalControlClient::connect\(None, Some\(expected_owner_generation_id\)\)/);
  assert.doesNotMatch(rustBridge, /expected_owner_generation_id:\s*Option<String>/);
  assert.match(rustBridge, /renderer_supplied_owner_generation: false/);
});

test("T169 recursive presentation binds immutable ids rather than labels", () => {
  assert.match(surface, /key=\{workspace\.multiplexerWorkspaceId\}/);
  assert.match(surface, /key=\{tab\.tabId\}/);
  assert.match(surface, /data-pane-id=\{node\.paneId\}/);
  assert.match(surface, /bindTarget/);
  assert.doesNotMatch(surface, /displayLabel.*===.*target/);
});

test("T169 preserves native keyboard, pointer, contrast, scale, and reduced-motion hooks", async () => {
  const css = await readFile(join(root, "src/multiplexer/topology.css"), "utf8");
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

test("T169 renderer fuzz cannot reach an unknown privileged method", () => {
  const literalCommand = /invoke(?:<[^>]+>)?\("([^"]+)"/g;
  const invoked = new Map();
  for (const source of renderer) {
    for (const match of source.text.matchAll(literalCommand)) {
      if (!invoked.has(match[1])) invoked.set(match[1], []);
      invoked.get(match[1]).push(source.path);
    }
  }

  assert.ok(hostCommands.length > 0, "host must register a closed typed command surface");
  for (const command of invoked.keys()) {
    assert.equal(
      hostCommands.includes(command),
      true,
      `renderer invokes unregistered privileged method: ${command}`,
    );
  }
  assert.deepEqual(
    [...invoked.keys()].sort(),
    [...hostCommands].sort(),
    "every registered privileged method must have an exact renderer literal, with no extras",
  );

  for (const source of renderer) {
    for (const match of source.text.matchAll(/invoke(?:<[^>]+>)?\(/g)) {
      const tail = source.text.slice(match.index + match[0].length, match.index + match[0].length + 120);
      assert.match(
        tail.trimStart(),
        /^"[^"]+"/,
        `${source.path} must dispatch only a literal privileged method name`,
      );
      assert.doesNotMatch(
        tail.slice(0, 60),
        /\$\{|\+\s*[A-Za-z_$]|`/,
        `${source.path} must never build a privileged method name from renderer data`,
      );
    }
  }

  const corpus = new Set();
  const seeds = [...invoked.keys()];
  const alphabet = "abcdefghijklmnopqrstuvwxyz_";
  for (const seed of seeds) {
    corpus.add(seed);
    corpus.add(seed.toUpperCase());
    corpus.add(`${seed}s`);
    corpus.add(`${seed}_`);
    corpus.add(`winds_${seed}`);
    for (let index = 0; index < seed.length; index += 1) {
      corpus.add(seed.slice(0, index) + seed.slice(index + 1));
      corpus.add(
        seed.slice(0, index) + alphabet[index % alphabet.length] + seed.slice(index),
      );
      corpus.add(seed.slice(0, index) + seed.slice(index + 1).toUpperCase());
    }
  }
  for (const candidate of corpus) {
    if (invoked.has(candidate)) continue;
    for (const source of renderer) {
      assert.equal(
        source.text.includes(`invoke<DesktopTopologySnapshot>("${candidate}"`) ||
          source.text.includes(`invoke("${candidate}"`),
        false,
        `fuzzed privileged method name became reachable: ${candidate}`,
      );
    }
    assert.equal(
      hostCommands.includes(candidate),
      false,
      `fuzzed privileged method name is registered by the host: ${candidate}`,
    );
  }
});

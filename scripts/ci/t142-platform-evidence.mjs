import { createHash } from 'node:crypto';
import { execFileSync, execSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import process from 'node:process';

function required(name) {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is required`);
  return value.trim();
}

function git(...args) {
  return execFileSync('git', args, { encoding: 'utf8' }).trim();
}

function command(commandLine) {
  return execSync(commandLine, { encoding: 'utf8' }).trim();
}

const candidate = required('CANDIDATE_SHA');
const head = git('rev-parse', 'HEAD');
const tree = git('rev-parse', 'HEAD^{tree}');
if (head !== candidate) throw new Error(`candidate moved: ${head} != ${candidate}`);
const executable = process.platform === 'win32' ? 'winds-desktop-host.exe' : 'winds-desktop-host';
const binaryPath = join(process.cwd(), 'desktop', 'src-tauri', 'target', 'release', executable);
if (!existsSync(binaryPath)) throw new Error(`release desktop binary missing: ${binaryPath}`);
const binarySha256 = createHash('sha256').update(readFileSync(binaryPath)).digest('hex');

const runnerOs = required('RUNNER_OS');
const runnerArch = required('RUNNER_ARCH');
const webviewVersion = required('WINDS_WEBVIEW_VERSION');
const webviewSource = required('WINDS_WEBVIEW_SOURCE');
const terminalBackend = runnerOs === 'Windows' ? 'ConPTY' : 'PTY';

const evidence = {
  schema: 'WINDS_SPEC_010_T142_PLATFORM_EVIDENCE_V1',
  candidate_commit: head,
  candidate_tree: tree,
  runner: {
    os: runnerOs,
    arch: runnerArch,
    image_os: process.env.ImageOS ?? null,
    image_version: process.env.ImageVersion ?? null,
  },
  webview: { version: webviewVersion, source: webviewSource },
  build: {
    profile: 'release',
    command: 'npm run desktop:build',
    binary_path: binaryPath,
    binary_sha256: binarySha256,
  },
  toolchain: {
    node: process.version,
    npm: command('npm --version'),
    rustc: command('rustc --version'),
  },
  direct_evidence: {
    tauri_release_build: true,
    native_workbench_terminal_input_focus_resize: true,
    native_terminal_backend: terminalBackend,
    renderer_keyboard_pointer_ime_contract: true,
    deterministic_system_appearance_and_high_dpi_contract: true,
  },
  nonclaims: [
    'No OS-level Tauri-window focus automation is claimed by T142.',
    'No OS-level IME injection automation is claimed by T142.',
    'No live native system-theme transition automation is claimed by T142.',
    'T142 does not substitute platform builds for the human visual acceptance required by T144.',
  ],
};
const destination = required('T142_EVIDENCE_PATH');
writeFileSync(destination, `${JSON.stringify(evidence, null, 2)}\n`, 'utf8');
console.log(`T142_PLATFORM_EVIDENCE=${destination}`);
console.log(`T142_BINARY_SHA256=${binarySha256}`);
console.log(`T142_WEBVIEW_VERSION=${webviewVersion}`);

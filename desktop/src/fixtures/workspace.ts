export type RuntimeFamily =
  | "codex"
  | "claude"
  | "shell"
  | "unknown"
  | "unavailable"
  | "conflicting"
  | "stale";

export type StreamKind = "agent" | "command" | "result" | "attention";

export interface StreamFixture {
  readonly id: string;
  readonly kind: StreamKind;
  readonly label: string;
  readonly body: string;
  readonly meta?: string;
}

export interface SessionFixture {
  readonly id: string;
  readonly title: string;
  readonly runtime: RuntimeFamily;
  readonly runtimeLabel: string;
  readonly status: string;
  readonly branch: string;
  readonly active?: boolean;
  readonly attention?: boolean;
  readonly stream: readonly StreamFixture[];
}

export interface ProjectFixture {
  readonly id: string;
  readonly name: string;
  readonly meta: string;
  readonly sessions: readonly SessionFixture[];
}

export interface FileFixture {
  readonly name: string;
  readonly depth: number;
  readonly kind: "folder" | "file";
  readonly open?: boolean;
  readonly selected?: boolean;
}

export const projectFixtures: readonly ProjectFixture[] = [
  {
    id: "winds",
    name: "Winds",
    meta: "TheHalfMoon/Winds",
    sessions: [
      {
        id: "quiet-current",
        title: "Quiet Current",
        runtime: "codex",
        runtimeLabel: "Codex",
        status: "Working",
        branch: "feat/010-t129-quiet-current",
        active: true,
        stream: [
          {
            id: "s1-a",
            kind: "agent",
            label: "Codex",
            body: "Mapped the desktop shell into Projects, independent Sessions, and a contextual right dock. No host authority is exposed in this static slice.",
            meta: "2m ago",
          },
          {
            id: "s1-b",
            kind: "command",
            label: "Plan",
            body: "Design tokens → static anatomy → visual contract tests → exact-head review",
          },
          {
            id: "s1-c",
            kind: "result",
            label: "Visual contract",
            body: "Dark, light, high-contrast, reduced-motion, keyboard focus, and responsive dual-pane behavior are represented by one component tree.",
            meta: "fixture",
          },
        ],
      },
      {
        id: "runtime-bridge",
        title: "Runtime bridge review",
        runtime: "claude",
        runtimeLabel: "Claude",
        status: "Needs review",
        branch: "main",
        attention: true,
        stream: [],
      },
      {
        id: "release-shell",
        title: "Release shell",
        runtime: "shell",
        runtimeLabel: "Shell",
        status: "Idle",
        branch: "main",
        stream: [],
      },
    ],
  },
  {
    id: "research",
    name: "Research",
    meta: "3 sessions",
    sessions: [
      {
        id: "parity-notes",
        title: "Agent workspace notes",
        runtime: "stale",
        runtimeLabel: "Stale runtime",
        status: "Stale",
        branch: "notes",
        stream: [],
      },
    ],
  },
];

export const secondarySession: SessionFixture = {
  id: "design-review",
  title: "Design review",
  runtime: "claude",
  runtimeLabel: "Claude",
  status: "Reviewing",
  branch: "feat/010-t129-quiet-current",
  stream: [
    {
      id: "s2-a",
      kind: "agent",
      label: "Claude",
      body: "The center workspace should read as a work surface, not two chat columns. Keep event rows compact and preserve independent focus per Session.",
      meta: "now",
    },
    {
      id: "s2-b",
      kind: "attention",
      label: "Needs you",
      body: "Confirm the final visual QA only after the exact candidate is captured at the required fixture sizes.",
      meta: "fixture state",
    },
    {
      id: "s2-c",
      kind: "result",
      label: "Constraint",
      body: "No proprietary vendor logo files. Runtime marks are Winds-authored presentation glyphs with accessible text.",
    },
  ],
};

export const fileFixtures: readonly FileFixture[] = [
  { name: "desktop", depth: 0, kind: "folder", open: true },
  { name: "src", depth: 1, kind: "folder", open: true },
  { name: "App.tsx", depth: 2, kind: "file", selected: true },
  { name: "styles.css", depth: 2, kind: "file" },
  { name: "components", depth: 2, kind: "folder", open: true },
  { name: "RuntimeMark.tsx", depth: 3, kind: "file" },
  { name: "fixtures", depth: 2, kind: "folder", open: false },
  { name: "DESIGN.md", depth: 0, kind: "file" },
  { name: "PRODUCT.md", depth: 0, kind: "file" },
];

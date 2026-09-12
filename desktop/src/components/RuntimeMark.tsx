import type { RuntimeFamily } from "../fixtures/workspace";

const runtimeGlyph: Record<RuntimeFamily, string> = {
  codex: "CX",
  claude: "CL",
  shell: "$",
  unknown: "?",
  unavailable: "—",
  conflicting: "!!",
  stale: "·",
};

const runtimeName: Record<RuntimeFamily, string> = {
  codex: "Codex",
  claude: "Claude",
  shell: "Shell",
  unknown: "Unknown runtime",
  unavailable: "Runtime unavailable",
  conflicting: "Runtime identity conflicting",
  stale: "Runtime identity stale",
};

export interface RuntimeMarkProps {
  readonly runtime: RuntimeFamily;
  readonly label?: string;
  readonly compact?: boolean;
}

export function RuntimeMark({ runtime, label, compact = false }: RuntimeMarkProps) {
  const accessible = label ?? runtimeName[runtime];

  return (
    <span className="runtime-identity" data-runtime={runtime} aria-label={accessible}>
      <span className="runtime-mark" aria-hidden="true">
        {runtimeGlyph[runtime]}
      </span>
      {!compact && <span className="runtime-name">{accessible}</span>}
    </span>
  );
}

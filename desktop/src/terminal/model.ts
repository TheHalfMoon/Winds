import type { TerminalStatus } from "./types";

export interface LiteralMatch {
  readonly line: number;
  readonly column: number;
  readonly length: number;
}

export function findLiteralMatch(
  lines: readonly string[],
  query: string,
  startLine = 0,
): LiteralMatch | null {
  if (query.length === 0 || lines.length === 0) return null;
  const start = Math.max(0, Math.min(startLine, lines.length - 1));
  for (let offset = 0; offset < lines.length; offset += 1) {
    const line = (start + offset) % lines.length;
    const column = lines[line]?.indexOf(query) ?? -1;
    if (column >= 0) return { line, column, length: query.length };
  }
  return null;
}


export function reconcileTerminalStatus(
  current: TerminalStatus | null,
  incoming: TerminalStatus,
): TerminalStatus {
  if (!current || current.terminalId !== incoming.terminalId) return incoming;
  if (current.lifecycle !== "live" && incoming.lifecycle === "live") return current;
  return incoming;
}

export function lifecycleLabel(status: TerminalStatus | null): string {
  if (!status) return "No live terminal";
  switch (status.lifecycle) {
    case "live": return `Live · ${status.profileDisplayName}`;
    case "exited": return `Exited${status.exitCode === null ? "" : ` · code ${status.exitCode}`}`;
    case "interrupted": return "Interrupted by explicit Winds action";
    case "ownership_lost": return "Ownership lost · process state unknown";
  }
}

export function validatedHttpLink(value: string): URL | null {
  try {
    const url = new URL(value);
    return url.protocol === "http:" || url.protocol === "https:" ? url : null;
  } catch {
    return null;
  }
}

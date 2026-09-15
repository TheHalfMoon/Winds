import type { BridgeSnapshot } from "../leftDock/types";
import type { RightDockSurface } from "../rightDock/types";

export type CommandPaletteKind = "surface" | "project" | "session" | "action";

export interface CommandPaletteItem {
  readonly id: string;
  readonly kind: CommandPaletteKind;
  readonly title: string;
  readonly subtitle: string;
  readonly workspaceId?: string;
  readonly sessionId?: string;
  readonly surface?: RightDockSurface;
}

const surfaces: readonly { surface: RightDockSurface; title: string }[] = [
  { surface: "files", title: "Open Files" },
  { surface: "changes", title: "Open Changes" },
  { surface: "evidence", title: "Open Evidence" },
  { surface: "context", title: "Open Context" },
  { surface: "artifacts", title: "Open Artifacts" },
  { surface: "needs_you", title: "Open Needs You" },
];

function normalized(value: string): string {
  return value.trim().toLowerCase();
}

function matches(item: CommandPaletteItem, query: string): boolean {
  const needle = normalized(query);
  if (!needle) return true;
  return normalized(`${item.title}\n${item.subtitle}\n${item.id}`).includes(needle);
}

export function commandPaletteItems(
  snapshot: BridgeSnapshot,
  query: string,
  projectScope: string | null,
): readonly CommandPaletteItem[] {
  if (projectScope) {
    const project = snapshot.projects.find((entry) => entry.project.canonicalWorkspaceId === projectScope);
    if (!project) return [];
    return project.sessions
      .filter((session) => !session.archived)
      .map((session) => ({
        id: `session:${session.canonicalSessionId}`,
        kind: "session" as const,
        title: session.displayName,
        subtitle: `${project.project.displayName} · ${session.canonicalSessionId}`,
        workspaceId: projectScope,
        sessionId: session.canonicalSessionId,
      }))
      .filter((item) => matches(item, query));
  }

  const items: CommandPaletteItem[] = [
    ...surfaces.map(({ surface, title }) => ({
      id: `surface:${surface}`,
      kind: "surface" as const,
      title,
      subtitle: "Safe dock navigation · no execution authority",
      surface,
    })),
    {
      id: "action:refresh",
      kind: "action",
      title: "Refresh navigation snapshot",
      subtitle: "Read canonical Project/Session presentation state",
    },
  ];
  for (const project of snapshot.projects) {
    items.push({
      id: `project:${project.project.canonicalWorkspaceId}`,
      kind: "project",
      title: project.project.displayName,
      subtitle: `Project · ${project.project.canonicalWorkspaceId} · choose an exact Session next`,
      workspaceId: project.project.canonicalWorkspaceId,
    });
    for (const session of project.sessions) {
      if (session.archived) continue;
      items.push({
        id: `session:${session.canonicalSessionId}`,
        kind: "session",
        title: session.displayName,
        subtitle: `${project.project.displayName} · ${session.canonicalSessionId}`,
        workspaceId: project.project.canonicalWorkspaceId,
        sessionId: session.canonicalSessionId,
      });
    }
  }
  return items.filter((item) => matches(item, query));
}

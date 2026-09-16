import { CurrentMark } from "../components/CurrentMark";

export type LeftToolSurface = "chat" | "projects";

function ChatGlyph() {
  return <span className="rail-glyph rail-glyph-chat" aria-hidden="true"><i /><i /></span>;
}

function ProjectsGlyph() {
  return <span className="rail-glyph rail-glyph-projects" aria-hidden="true"><i /><i /></span>;
}

export function ActivityRail({
  active,
  onSelect,
  onOpenCommandMenu,
}: {
  readonly active: LeftToolSurface;
  readonly onSelect: (surface: LeftToolSurface) => void;
  readonly onOpenCommandMenu: () => void;
}) {
  return (
    <nav className="activity-rail" aria-label="Winds tool windows">
      <div className="activity-rail-mark"><CurrentMark compact /></div>
      <div className="activity-rail-tools" role="group" aria-label="Left tool window">
        <button type="button" className="rail-action" data-active={active === "chat" ? "true" : "false"} aria-pressed={active === "chat"} aria-label="Open Chat tool window" onClick={() => onSelect("chat")}><ChatGlyph /></button>
        <button type="button" className="rail-action" data-active={active === "projects" ? "true" : "false"} aria-pressed={active === "projects"} aria-label="Open Projects tool window" onClick={() => onSelect("projects")}><ProjectsGlyph /></button>
      </div>
      <div className="activity-rail-bottom">
        <button type="button" className="rail-action rail-command" aria-label="Open command menu" onClick={onOpenCommandMenu}><span aria-hidden="true">⌘</span></button>
      </div>
    </nav>
  );
}

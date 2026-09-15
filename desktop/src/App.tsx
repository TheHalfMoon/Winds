import { useEffect, useState } from "react";
import { CommandPalette } from "./commandPalette/CommandPalette";
import { DualSessionWorkspace, type DualSessionDockIntent, type DualSessionSelection } from "./dualSession/DualSessionWorkspace";
import { LeftDock } from "./leftDock/LeftDock";
import { RightDock } from "./rightDock/RightDock";
import type { RightDockSurface } from "./rightDock/types";

export function App() {
  const [selection, setSelection] = useState<DualSessionSelection | null>(null);
  const [dockTarget, setDockTarget] = useState<DualSessionSelection | null>(null);
  const [dockSurface, setDockSurface] = useState<RightDockSurface>("files");
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setCommandPaletteOpen((value) => !value);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  const focusDock = (target: DualSessionSelection) => {
    setDockTarget((current) => current?.workspaceId === target.workspaceId && current.sessionId === target.sessionId ? current : target);
  };

  const openDockIntent = (intent: DualSessionDockIntent) => {
    focusDock(intent);
    setDockSurface(intent.surface);
  };

  const selectSession = (workspaceId: string, sessionId: string) => {
    const target = { workspaceId, sessionId };
    setSelection(target);
    focusDock(target);
  };

  return (
    <div className="winds-app" aria-label="Winds Desktop workspace">
      <header className="top-chrome">
        <div className="brand-lockup" aria-label="Winds"><span className="winds-mark" aria-hidden="true">W</span><span className="winds-wordmark">Winds</span></div>
        <div className="workspace-crumbs" aria-label="Current presentation context">
          <span>TheHalfMoon</span><span aria-hidden="true">/</span><strong>Winds</strong><span className="chrome-separator" aria-hidden="true" /><span>Dual Session</span>
        </div>
        <div className="chrome-actions">
          <span className="static-mode">T138 · trusted attention + command navigation</span>
          <button type="button" className="icon-button" aria-label="Open command menu" onClick={() => setCommandPaletteOpen(true)}>⌘</button>
          <button type="button" className="avatar-button" aria-label="Profile">AS</button>
        </div>
      </header>

      <div className="workspace-grid">
        <LeftDock onSelectSession={selectSession} requestedSessionId={selection?.sessionId ?? null} />
        <main className="center-workspace" aria-label="Session workspace">
          <DualSessionWorkspace selection={selection} onFocusSession={focusDock} onDockIntent={openDockIntent} />
        </main>
        <RightDock target={dockTarget} requestedSurface={dockSurface} onFocusAttention={(workspaceId, sessionId) => { selectSession(workspaceId, sessionId); setDockSurface("context"); }} />
      </div>

      <CommandPalette
        open={commandPaletteOpen}
        onClose={() => setCommandPaletteOpen(false)}
        onSelectSession={selectSession}
        onOpenSurface={setDockSurface}
      />

      <footer className="status-rail" aria-label="Desktop status">
        <div><span className="status-dot" data-tone="ok" /> renderer boundary: immutable truth surfaces · trusted Needs You · safe command navigation</div>
        <div>dual Session · exact target · no broadcast</div>
        <div>theme · quiet current</div>
      </footer>
    </div>
  );
}

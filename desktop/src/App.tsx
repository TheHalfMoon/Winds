import { lazy, Suspense, useEffect, useState } from "react";
import { ActivityRail, type LeftToolSurface } from "./activityRail/ActivityRail";
import { CurrentMark } from "./components/CurrentMark";
import { DualSessionWorkspace, type DualSessionDockIntent, type DualSessionSelection } from "./dualSession/DualSessionWorkspace";
import { ChatToolWindow } from "./leftDock/ChatToolWindow";
import { RightDock } from "./rightDock/RightDock";
import type { RightDockSurface } from "./rightDock/types";

const CommandPalette = lazy(async () => {
  const module = await import("./commandPalette/CommandPalette");
  return { default: module.CommandPalette };
});

const LeftDock = lazy(async () => {
  const module = await import("./leftDock/LeftDock");
  return { default: module.LeftDock };
});

function initialLeftTool(): LeftToolSurface {
  return document.documentElement.dataset.leftTool === "projects" ? "projects" : "chat";
}

export function App() {
  const [selection, setSelection] = useState<DualSessionSelection | null>(null);
  const [dockTarget, setDockTarget] = useState<DualSessionSelection | null>(null);
  const [dockSurface, setDockSurface] = useState<RightDockSurface>("files");
  const [leftTool, setLeftTool] = useState<LeftToolSurface>(initialLeftTool);
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
    setSelection((current) => current?.workspaceId === target.workspaceId && current.sessionId === target.sessionId ? current : target);
    setDockTarget((current) => current?.workspaceId === target.workspaceId && current.sessionId === target.sessionId ? current : target);
  };

  const openDockIntent = (intent: DualSessionDockIntent) => {
    focusDock(intent);
    setDockSurface(intent.surface);
  };

  const selectSession = (workspaceId: string, sessionId: string) => {
    focusDock({ workspaceId, sessionId });
  };

  const selectProjectSession = (workspaceId: string, sessionId: string) => {
    selectSession(workspaceId, sessionId);
    setLeftTool("chat");
  };

  return (
    <div className="winds-app" aria-label="Winds Desktop workspace">
      <header className="top-chrome">
        <div className="brand-lockup" aria-label="Winds Current Spectrum"><CurrentMark /><span className="winds-wordmark">Winds</span></div>
        <div className="workspace-crumbs" aria-label="Current presentation context">
          <span>TheHalfMoon</span><span aria-hidden="true">/</span><strong>Winds</strong><span className="chrome-separator" aria-hidden="true" /><span>Workbench</span>
        </div>
        <div className="chrome-actions">
          <span className="static-mode">T142A · Current Spectrum · exact Session identity</span>
          <button type="button" className="icon-button" aria-label="Open command menu" onClick={() => setCommandPaletteOpen(true)}>⌘</button>
          <button type="button" className="avatar-button" aria-label="Profile">AS</button>
        </div>
      </header>

      <div className="workspace-grid" data-left-tool={leftTool}>
        <ActivityRail active={leftTool} onSelect={setLeftTool} onOpenCommandMenu={() => setCommandPaletteOpen(true)} />
        {leftTool === "chat" ? (
          <ChatToolWindow selection={selection} onSelectSession={selectSession} />
        ) : (
          <Suspense fallback={<aside className="left-dock" aria-label="Projects and Sessions">Loading Projects…</aside>}>
            <LeftDock onSelectSession={selectProjectSession} requestedSessionId={selection?.sessionId ?? null} />
          </Suspense>
        )}
        <main className="center-workspace" aria-label="Workbench">
          <DualSessionWorkspace selection={selection} onFocusSession={focusDock} onDockIntent={openDockIntent} />
        </main>
        <RightDock target={dockTarget} requestedSurface={dockSurface} onFocusAttention={(workspaceId, sessionId) => { focusDock({ workspaceId, sessionId }); setDockSurface("context"); }} />
      </div>

      {commandPaletteOpen && (
        <Suspense fallback={null}>
          <CommandPalette
            open
            onClose={() => setCommandPaletteOpen(false)}
            onSelectSession={(workspaceId, sessionId) => { selectSession(workspaceId, sessionId); setLeftTool("chat"); }}
            onOpenSurface={setDockSurface}
          />
        </Suspense>
      )}

      <footer className="status-rail" aria-label="Desktop status">
        <div><span className="status-dot" data-tone="ok" /> renderer boundary · immutable truth surfaces · trusted Needs You</div>
        <div>exact Session target · no broadcast</div>
        <div>identity · Current Spectrum</div>
      </footer>
    </div>
  );
}

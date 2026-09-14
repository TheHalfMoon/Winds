import { useState } from "react";
import { DualSessionWorkspace } from "./dualSession/DualSessionWorkspace";
import { fileFixtures } from "./fixtures/workspace";
import { LeftDock } from "./leftDock/LeftDock";

function RightDock() {
  return (
    <aside className="right-dock" aria-label="Context dock">
      <div className="right-tabs" role="tablist" aria-label="Context surfaces">
        {["Files", "Changes", "Evidence", "Context", "Artifacts"].map((tab, index) => (
          <button
            type="button"
            role="tab"
            aria-selected={index === 0}
            className="right-tab"
            data-active={index === 0 ? "true" : "false"}
            key={tab}
          >
            {tab}
          </button>
        ))}
      </div>

      <div className="right-dock-body" role="tabpanel" aria-label="Files">
        <div className="right-dock-heading">
          <div><p className="section-kicker">Session scope</p><h2>Files</h2></div>
          <button type="button" className="icon-button" aria-label="Collapse right dock">›</button>
        </div>
        <div className="file-filter"><span aria-hidden="true">⌕</span><input type="search" aria-label="Filter files" placeholder="Filter files" /></div>
        <div className="file-tree" role="tree" aria-label="Session files">
          {fileFixtures.map((file) => (
            <button
              type="button"
              role="treeitem"
              aria-selected={file.selected ?? false}
              className="file-row"
              data-selected={file.selected ? "true" : "false"}
              style={{ paddingInlineStart: `${10 + file.depth * 14}px` }}
              key={`${file.depth}-${file.name}`}
            >
              <span className="file-icon" aria-hidden="true">{file.kind === "folder" ? (file.open ? "⌄" : "›") : "·"}</span>
              <span>{file.name}</span>
            </button>
          ))}
        </div>
        <section className="dock-inspector" aria-label="Selected file fixture">
          <p className="section-kicker">Selected</p>
          <strong>desktop/src/App.tsx</strong>
          <dl>
            <div><dt>Binding</dt><dd>Fixture only</dd></div>
            <div><dt>Authority</dt><dd>None</dd></div>
          </dl>
        </section>
      </div>
    </aside>
  );
}

export function App() {
  const [selection, setSelection] = useState<{ workspaceId: string; sessionId: string } | null>(null);

  return (
    <div className="winds-app" aria-label="Winds Desktop workspace">
      <header className="top-chrome">
        <div className="brand-lockup" aria-label="Winds"><span className="winds-mark" aria-hidden="true">W</span><span className="winds-wordmark">Winds</span></div>
        <div className="workspace-crumbs" aria-label="Current presentation context">
          <span>TheHalfMoon</span><span aria-hidden="true">/</span><strong>Winds</strong><span className="chrome-separator" aria-hidden="true" /><span>Dual Session</span>
        </div>
        <div className="chrome-actions">
          <span className="static-mode">T134 · exact-target dual Session presentation</span>
          <button type="button" className="icon-button" aria-label="Open command menu">⌘</button>
          <button type="button" className="avatar-button" aria-label="Profile">AS</button>
        </div>
      </header>

      <div className="workspace-grid">
        <LeftDock onSelectSession={(workspaceId, sessionId) => setSelection({ workspaceId, sessionId })} />
        <main className="center-workspace" aria-label="Session workspace">
          <DualSessionWorkspace selection={selection} />
        </main>
        <RightDock />
      </div>

      <footer className="status-rail" aria-label="Desktop status">
        <div><span className="status-dot" data-tone="ok" /> renderer boundary: canonical layout identities + fixture-only Session surfaces</div>
        <div>dual Session · exact target · no broadcast</div>
        <div>theme · quiet current</div>
      </footer>
    </div>
  );
}

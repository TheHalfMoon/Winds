import { RuntimeMark } from "./components/RuntimeMark";
import { LeftDock } from "./leftDock/LeftDock";
import { fileFixtures, secondarySession, type SessionFixture } from "./fixtures/workspace";
import { SessionSurface } from "./sessionSurface/SessionSurface";
import { sessionSurfaceFixtures } from "./sessionSurface/fixtures";

function StreamEvent({ item }: { readonly item: SessionFixture["stream"][number] }) {
  return (
    <article className="stream-event" data-kind={item.kind}>
      <div className="stream-rail" aria-hidden="true" />
      <div className="stream-event-body">
        <header className="stream-event-header">
          <span className="stream-label">{item.label}</span>
          {item.meta && <span className="stream-meta">{item.meta}</span>}
        </header>
        {item.kind === "command" ? (
          <pre className="command-block"><code>{item.body}</code></pre>
        ) : (
          <p>{item.body}</p>
        )}
      </div>
    </article>
  );
}

function SessionPane({ session }: { readonly session: SessionFixture }) {
  return (
    <section className="session-pane" data-primary="false" aria-label={`${session.title} static Session fixture`}>
      <header className="session-header">
        <div className="session-identity">
          <RuntimeMark runtime={session.runtime} label={session.runtimeLabel} />
          <div>
            <h2>{session.title}</h2>
            <p>{session.branch}</p>
          </div>
        </div>
        <div className="session-header-actions">
          <span className="status-chip" data-status={session.status.toLowerCase().replaceAll(" ", "-")}>
            {session.status}
          </span>
        </div>
      </header>

      <div className="stream-toolbar" aria-label="Static Session stream controls">
        <div className="segmented-control" aria-label="Work Stream view">
          <button type="button" data-active="true">Work Stream</button>
          <button type="button" disabled>Terminal</button>
        </div>
        <span className="fixture-label">Static T129 fixture · no dispatch</span>
      </div>

      <div className="work-stream" tabIndex={0} aria-label={`${session.title} Work Stream`}>
        <div className="stream-day"><span>Today</span></div>
        {session.stream.length > 0 ? (
          session.stream.map((item) => <StreamEvent item={item} key={item.id} />)
        ) : (
          <div className="empty-state">
            <strong>No work events yet</strong>
            <span>Static fixture only; no runtime input is authorized.</span>
          </div>
        )}
      </div>

      <form className="composer" aria-label={`${session.title} static composer`} onSubmit={(event) => event.preventDefault()}>
        <div className="composer-context">
          <span className="context-pill">Target · static fixture</span>
          <span className="context-pill">Branch · {session.branch}</span>
        </div>
        <textarea aria-label={`${session.runtimeLabel} fixture input unavailable`} placeholder="T133 live input is unavailable" rows={3} disabled />
        <div className="composer-actions">
          <div className="composer-tools"><span className="composer-hint">Static anatomy only</span></div>
          <div className="composer-submit"><button type="submit" className="primary-action" disabled>Unavailable</button></div>
        </div>
      </form>
    </section>
  );
}

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
  return (
    <div className="winds-app" aria-label="Winds Desktop workspace">
      <header className="top-chrome">
        <div className="brand-lockup" aria-label="Winds"><span className="winds-mark" aria-hidden="true">W</span><span className="winds-wordmark">Winds</span></div>
        <div className="workspace-crumbs" aria-label="Current presentation context">
          <span>TheHalfMoon</span><span aria-hidden="true">/</span><strong>Winds</strong><span className="chrome-separator" aria-hidden="true" /><span>Session Surface</span>
        </div>
        <div className="chrome-actions">
          <span className="static-mode">T133 · fixture-only agent surface</span>
          <button type="button" className="icon-button" aria-label="Open command menu">⌘</button>
          <button type="button" className="avatar-button" aria-label="Profile">AS</button>
        </div>
      </header>

      <div className="workspace-grid">
        <LeftDock />
        <main className="center-workspace" aria-label="Session workspace">
          <div className="dual-session" aria-label="Dual Session view">
            <SessionSurface session={sessionSurfaceFixtures[0]} />
            <SessionPane session={secondarySession} />
          </div>
        </main>
        <RightDock />
      </div>

      <footer className="status-rail" aria-label="Desktop status">
        <div><span className="status-dot" data-tone="ok" /> renderer boundary: canonical left dock + fixture-only session surface</div>
        <div>composer · no live dispatch before T139</div>
        <div>theme · quiet current</div>
      </footer>
    </div>
  );
}

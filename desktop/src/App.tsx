import { RuntimeMark } from "./components/RuntimeMark";
import {
  fileFixtures,
  projectFixtures,
  secondarySession,
  type SessionFixture,
} from "./fixtures/workspace";

function SessionRow({ session }: { readonly session: SessionFixture }) {
  return (
    <button
      type="button"
      className="session-row"
      data-active={session.active ? "true" : "false"}
      data-attention={session.attention ? "true" : "false"}
      aria-current={session.active ? "page" : undefined}
    >
      <RuntimeMark runtime={session.runtime} label={session.runtimeLabel} compact />
      <span className="session-row-copy">
        <span className="session-row-title">{session.title}</span>
        <span className="session-row-meta">{session.status}</span>
      </span>
      {session.attention && (
        <span className="attention-pip" aria-label="Needs attention">
          1
        </span>
      )}
    </button>
  );
}

function LeftDock() {
  return (
    <aside className="left-dock" aria-label="Projects and Sessions">
      <div className="dock-heading-row">
        <div>
          <p className="section-kicker">Workspace</p>
          <h2>Projects</h2>
        </div>
        <button type="button" className="icon-button" aria-label="Create Project">
          +
        </button>
      </div>

      <nav className="project-list" aria-label="Project navigation">
        {projectFixtures.map((project, index) => (
          <section className="project-group" key={project.id} data-open={index === 0 ? "true" : "false"}>
            <button type="button" className="project-row" aria-expanded={index === 0}>
              <span className="disclosure" aria-hidden="true">
                {index === 0 ? "⌄" : "›"}
              </span>
              <span className="project-copy">
                <span className="project-name">{project.name}</span>
                <span className="project-meta">{project.meta}</span>
              </span>
              <span className="project-count">{project.sessions.length}</span>
            </button>
            {index === 0 && (
              <div className="session-list" aria-label={`${project.name} Sessions`}>
                {project.sessions.map((session) => (
                  <SessionRow session={session} key={session.id} />
                ))}
              </div>
            )}
          </section>
        ))}
      </nav>

      <div className="dock-foot">
        <button type="button" className="quiet-action">
          <span aria-hidden="true">⌘</span>
          Command
          <kbd>⌘K</kbd>
        </button>
      </div>
    </aside>
  );
}

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
          <pre className="command-block">
            <code>{item.body}</code>
          </pre>
        ) : (
          <p>{item.body}</p>
        )}
      </div>
    </article>
  );
}

function SessionPane({ session, primary = false }: { readonly session: SessionFixture; readonly primary?: boolean }) {
  return (
    <section className="session-pane" data-primary={primary ? "true" : "false"} aria-label={`${session.title} Session`}>
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
          <button type="button" className="icon-button" aria-label={`Rename ${session.title} Session`}>
            ✎
          </button>
          <button type="button" className="icon-button" aria-label={`More actions for ${session.title}`}>
            ···
          </button>
        </div>
      </header>

      <div className="stream-toolbar" aria-label="Session stream controls">
        <div className="segmented-control" aria-label="Work Stream view">
          <button type="button" data-active="true">Work Stream</button>
          <button type="button">Terminal</button>
        </div>
        <span className="fixture-label">Static fixture</span>
      </div>

      <div className="work-stream" tabIndex={0} aria-label={`${session.title} Work Stream`}>
        <div className="stream-day"><span>Today</span></div>
        {session.stream.length > 0 ? (
          session.stream.map((item) => <StreamEvent item={item} key={item.id} />)
        ) : (
          <div className="empty-state">
            <strong>No work events yet</strong>
            <span>Start the Session when live projections are authorized.</span>
          </div>
        )}
      </div>

      <form className="composer" aria-label={`${session.title} composer`}>
        <div className="composer-context">
          <span className="context-pill">Project · Winds</span>
          <span className="context-pill">Branch · {session.branch}</span>
        </div>
        <textarea
          aria-label={`Message ${session.runtimeLabel}`}
          placeholder={`Ask ${session.runtimeLabel} to work in this Session…`}
          rows={3}
        />
        <div className="composer-actions">
          <div className="composer-tools">
            <button type="button" className="icon-button" aria-label="Attach context">＋</button>
            <button type="button" className="text-button">Plan</button>
          </div>
          <div className="composer-submit">
            <span className="composer-hint">⌘↵ send</span>
            <button type="submit" className="primary-action">Send</button>
          </div>
        </div>
      </form>
    </section>
  );
}

function RightDock() {
  return (
    <aside className="right-dock" aria-label="Context dock">
      <div className="right-tabs" role="tablist" aria-label="Context surfaces">
        {['Files', 'Changes', 'Evidence', 'Context', 'Artifacts'].map((tab, index) => (
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
          <div>
            <p className="section-kicker">Session scope</p>
            <h2>Files</h2>
          </div>
          <button type="button" className="icon-button" aria-label="Collapse right dock">›</button>
        </div>

        <div className="file-filter">
          <span aria-hidden="true">⌕</span>
          <input type="search" aria-label="Filter files" placeholder="Filter files" />
        </div>

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
              <span className="file-icon" aria-hidden="true">
                {file.kind === "folder" ? (file.open ? "⌄" : "›") : "·"}
              </span>
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
  const primarySession = projectFixtures[0].sessions[0];

  return (
    <div className="winds-app" aria-label="Winds Desktop static anatomy">
      <header className="top-chrome">
        <div className="brand-lockup" aria-label="Winds">
          <span className="winds-mark" aria-hidden="true">W</span>
          <span className="winds-wordmark">Winds</span>
        </div>
        <div className="workspace-crumbs" aria-label="Current presentation context">
          <span>TheHalfMoon</span>
          <span aria-hidden="true">/</span>
          <strong>Winds</strong>
          <span className="chrome-separator" aria-hidden="true" />
          <span>Quiet Current</span>
        </div>
        <div className="chrome-actions">
          <span className="static-mode">T129 · static anatomy</span>
          <button type="button" className="icon-button" aria-label="Open command menu">⌘</button>
          <button type="button" className="avatar-button" aria-label="Profile">AS</button>
        </div>
      </header>

      <div className="workspace-grid">
        <LeftDock />
        <main className="center-workspace" aria-label="Session workspace">
          <div className="dual-session" aria-label="Dual Session view">
            <SessionPane session={primarySession} primary />
            <SessionPane session={secondarySession} />
          </div>
        </main>
        <RightDock />
      </div>

      <footer className="status-rail" aria-label="Static fixture status">
        <div><span className="status-dot" data-tone="ok" /> renderer authority: none</div>
        <div>dual session · independent input</div>
        <div>theme · quiet current</div>
      </footer>
    </div>
  );
}

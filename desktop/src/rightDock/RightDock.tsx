import { useEffect, useMemo, useRef, useState } from "react";
import { rightDockBridge } from "./bridge";
import { displayPath, sameRightDockBinding, shortIdentity, statusLabel } from "./model";
import type {
  FilePreviewResponse, RightDockBinding, RightDockChangesResponse, RightDockFilesResponse,
  RightDockSurface, RightDockTarget,
} from "./types";

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function RightDock({
  target,
  requestedSurface = "files",
}: {
  readonly target: RightDockTarget | null;
  readonly requestedSurface?: RightDockSurface;
}) {
  const bridge = useMemo(() => rightDockBridge(), []);
  const [surface, setSurface] = useState<RightDockSurface>(requestedSurface);
  const [binding, setBinding] = useState<RightDockBinding | null>(null);
  const [files, setFiles] = useState<RightDockFilesResponse | null>(null);
  const [changes, setChanges] = useState<RightDockChangesResponse | null>(null);
  const [preview, setPreview] = useState<FilePreviewResponse | null>(null);
  const [filter, setFilter] = useState("");
  const [status, setStatus] = useState("Select a Session to bind Files and Changes");
  const [busy, setBusy] = useState(false);
  const requestGeneration = useRef(0);

  useEffect(() => setSurface(requestedSurface), [requestedSurface]);

  useEffect(() => {
    const generation = ++requestGeneration.current;
    setBinding(null);
    setFiles(null);
    setChanges(null);
    setPreview(null);
    if (!target) {
      setBusy(false);
      setStatus("Select a Session to bind Files and Changes");
      return;
    }
    setBusy(true);
    setStatus(`Binding ${surface === "files" ? "Files" : "Changes"} to exact Session…`);
    void (async () => {
      try {
        const bound = await bridge.bind(target);
        if (generation !== requestGeneration.current) return;
        setBinding(bound);
        if (surface === "files") {
          const response = await bridge.files(bound);
          if (generation !== requestGeneration.current) return;
          if (!sameRightDockBinding(bound, response.binding)) {
            setStatus("Discarded stale Files result · original binding did not match current Session");
            return;
          }
          setFiles(response);
          setStatus(`${response.entries.length} bounded file entries${response.truncated ? " · truncated" : ""}`);
        } else {
          const response = await bridge.changes(bound);
          if (generation !== requestGeneration.current) return;
          if (!sameRightDockBinding(bound, response.binding)) {
            setStatus("Discarded stale Changes result · original binding did not match current Session");
            return;
          }
          setChanges(response);
          setStatus(`${response.entries.length} Git change entries · diff is not verification evidence`);
        }
      } catch (error) {
        if (generation === requestGeneration.current) setStatus(`Right dock unavailable · ${errorMessage(error)}`);
      } finally {
        if (generation === requestGeneration.current) setBusy(false);
      }
    })();
    return () => {
      if (requestGeneration.current === generation) requestGeneration.current += 1;
    };
  }, [bridge, target?.workspaceId, target?.sessionId, surface]);

  const visibleFiles = useMemo(() => {
    const query = filter.trim().toLocaleLowerCase();
    if (!query) return files?.entries ?? [];
    return (files?.entries ?? []).filter((entry) => entry.path.toLocaleLowerCase().includes(query));
  }, [files, filter]);

  const openPreview = async (path: string, kind: "file" | "symlink") => {
    if (!binding || kind === "symlink") {
      setPreview(null);
      setStatus(kind === "symlink" ? "Symlink preview is blocked by the host boundary" : "No active file binding");
      return;
    }
    const generation = ++requestGeneration.current;
    setBusy(true);
    setStatus(`Reading bounded preview · ${displayPath(path)}`);
    try {
      const response = await bridge.preview(binding, path);
      if (generation !== requestGeneration.current) return;
      if (!sameRightDockBinding(binding, response.binding)) {
        setStatus("Discarded stale file preview · binding changed before render");
        return;
      }
      setPreview(response);
      setStatus(response.state === "text" ? `Preview · ${displayPath(path)}` : `${response.state.replace("_", " ")} · ${displayPath(path)}`);
    } catch (error) {
      if (generation === requestGeneration.current) setStatus(`File preview unavailable · ${errorMessage(error)}`);
    } finally {
      if (generation === requestGeneration.current) setBusy(false);
    }
  };

  return (
    <aside className="right-dock" aria-label="Context dock" data-source={bridge.source}>
      <div className="right-tabs" role="tablist" aria-label="Context surfaces">
        <button type="button" role="tab" aria-selected={surface === "files"} className="right-tab" data-active={surface === "files" ? "true" : "false"} onClick={() => setSurface("files")}>Files</button>
        <button type="button" role="tab" aria-selected={surface === "changes"} className="right-tab" data-active={surface === "changes" ? "true" : "false"} onClick={() => setSurface("changes")}>Changes</button>
        {["Evidence", "Context", "Artifacts"].map((tab) => <button type="button" role="tab" aria-selected={false} className="right-tab" disabled key={tab}>{tab}</button>)}
      </div>

      <div className="right-dock-body" role="tabpanel" aria-label={surface === "files" ? "Files" : "Changes"}>
        <div className="right-dock-heading">
          <div><p className="section-kicker">Exact Session scope</p><h2>{surface === "files" ? "Files" : "Changes"}</h2></div>
          <span className="dock-source-badge">{bridge.source === "canonical" ? "Rust-owned" : "Fixture"}</span>
        </div>

        {binding ? (
          <dl className="dock-binding" aria-label="Immutable right dock binding">
            <div><dt>Session</dt><dd>{binding.sessionId}</dd></div>
            <div><dt>HEAD</dt><dd>{shortIdentity(binding.headOid)}</dd></div>
            <div><dt>Tree</dt><dd>{shortIdentity(binding.treeOid)}</dd></div>
            <div><dt>Candidate</dt><dd>{binding.candidateOid ? shortIdentity(binding.candidateOid) : "none bound"}</dd></div>
            <div><dt>Stage</dt><dd>{binding.stageRunId ?? "none bound"}</dd></div>
            <div><dt>Binding</dt><dd>{shortIdentity(binding.bindingDigest, 12)}</dd></div>
          </dl>
        ) : <div className="dock-empty-binding">No canonical binding</div>}

        {surface === "files" ? (
          <div className="right-dock-surface">
            <div className="file-filter"><span aria-hidden="true">⌕</span><input type="search" aria-label="Filter files" placeholder="Filter files" value={filter} onChange={(event) => setFilter(event.currentTarget.value)} /></div>
            <div className="file-tree" role="tree" aria-label="Session files" aria-busy={busy}>
              {visibleFiles.map((file) => (
                <button type="button" role="treeitem" aria-selected={preview?.path === file.path} className="file-row" data-selected={preview?.path === file.path ? "true" : "false"} onClick={() => void openPreview(file.path, file.kind)} key={file.path}>
                  <span className="file-icon" aria-hidden="true">{file.kind === "symlink" ? "↗" : "·"}</span><span>{displayPath(file.path)}</span>
                </button>
              ))}
              {!busy && files && visibleFiles.length === 0 && <div className="dock-empty-state">No files match this binding/filter.</div>}
            </div>
            {preview && (
              <section className="dock-inspector" aria-label="Selected file preview">
                <p className="section-kicker">Bound preview · {preview.state.replace("_", " ")}</p>
                <strong>{displayPath(preview.path)}</strong>
                {preview.state === "text" && preview.content !== null ? <pre className="file-preview"><code>{preview.content}</code></pre> : <p>{preview.state === "binary" ? "Binary preview intentionally unavailable." : "File exceeds the bounded preview size."}</p>}
              </section>
            )}
          </div>
        ) : (
          <div className="right-dock-surface changes-surface" aria-busy={busy}>
            <div className="change-list" aria-label="Git change status">
              {changes?.entries.map((entry) => <div className="change-row" key={`${entry.status}-${entry.path}`}><code>{entry.status}</code><span>{displayPath(entry.path)}</span><small>{statusLabel(entry.status)}</small></div>)}
              {!busy && changes && changes.entries.length === 0 && <div className="dock-empty-state">Working tree has no reported changes.</div>}
            </div>
            {changes && <section className="dock-inspector" aria-label="Repository-native diff"><p className="section-kicker">Git diff · untrusted presentation · not verification</p>{changes.diff ? <pre className="changes-diff"><code>{changes.diff}</code></pre> : <p>No staged or unstaged diff content.</p>}{changes.diffLossy && <p className="dock-warning">Diff contained non-UTF-8 bytes and is rendered lossily.</p>}</section>}
          </div>
        )}
        <p className="dock-status" role="status" aria-live="polite">{status}</p>
      </div>
    </aside>
  );
}

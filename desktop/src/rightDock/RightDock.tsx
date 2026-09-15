import { useEffect, useMemo, useRef, useState } from "react";
import { leftDockBridge } from "../leftDock/bridge";
import type { BridgeAttentionSnapshot } from "../leftDock/types";
import { rightDockBridge } from "./bridge";
import { displayPath, sameRightDockBinding, shortIdentity, statusLabel } from "./model";
import type {
  FilePreviewResponse, RightDockArtifactEntry, RightDockArtifactsResponse, RightDockBinding,
  RightDockChangesResponse, RightDockContextResponse, RightDockEvidenceResponse,
  RightDockFilesResponse, RightDockSurface, RightDockTarget,
} from "./types";

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

const TRUTH_REVALIDATION_MS = 2_000;
const RIGHT_DOCK_SURFACES = ["files", "changes", "evidence", "context", "artifacts", "needs_you"] as const satisfies readonly RightDockSurface[];

function surfaceTitle(surface: RightDockSurface): string {
  if (surface === "needs_you") return "Needs You";
  return surface.charAt(0).toUpperCase() + surface.slice(1);
}

function isTruthSensitiveSurface(surface: RightDockSurface): boolean {
  return surface === "evidence" || surface === "context" || surface === "artifacts";
}

export function RightDock({
  target,
  requestedSurface = "files",
  onFocusAttention,
}: {
  readonly target: RightDockTarget | null;
  readonly requestedSurface?: RightDockSurface;
  readonly onFocusAttention?: (workspaceId: string, sessionId: string) => void;
}) {
  const bridge = useMemo(() => rightDockBridge(), []);
  const attentionBridge = useMemo(() => leftDockBridge(), []);
  const [surface, setSurface] = useState<RightDockSurface>(requestedSurface);
  const [binding, setBinding] = useState<RightDockBinding | null>(null);
  const [files, setFiles] = useState<RightDockFilesResponse | null>(null);
  const [changes, setChanges] = useState<RightDockChangesResponse | null>(null);
  const [evidence, setEvidence] = useState<RightDockEvidenceResponse | null>(null);
  const [context, setContext] = useState<RightDockContextResponse | null>(null);
  const [artifacts, setArtifacts] = useState<RightDockArtifactsResponse | null>(null);
  const [attention, setAttention] = useState<BridgeAttentionSnapshot | null>(null);
  const [preview, setPreview] = useState<FilePreviewResponse | null>(null);
  const [selectedArtifact, setSelectedArtifact] = useState<RightDockArtifactEntry | null>(null);
  const [filter, setFilter] = useState("");
  const [status, setStatus] = useState("Select a Session to bind the right dock");
  const [busy, setBusy] = useState(false);
  const [truthRefresh, setTruthRefresh] = useState(0);
  const requestGeneration = useRef(0);
  const tabRefs = useRef<(HTMLButtonElement | null)[]>([]);

  useEffect(() => setSurface(requestedSurface), [requestedSurface]);

  useEffect(() => {
    const generation = ++requestGeneration.current;
    setBinding(null);
    setFiles(null);
    setChanges(null);
    setEvidence(null);
    setContext(null);
    setArtifacts(null);
    setAttention(null);
    setPreview(null);
    setSelectedArtifact(null);
    if (surface === "needs_you") {
      setBusy(true);
      setStatus("Loading trusted canonical attention…");
      void attentionBridge.attentionSnapshot()
        .then((response) => {
          if (generation !== requestGeneration.current) return;
          setAttention(response);
          setStatus(`${response.items.length} trusted attention item${response.items.length === 1 ? "" : "s"} · ranking grants no authority`);
        })
        .catch((error) => {
          if (generation === requestGeneration.current) setStatus(`Needs You unavailable · ${errorMessage(error)}`);
        })
        .finally(() => {
          if (generation === requestGeneration.current) setBusy(false);
        });
      return () => {
        if (requestGeneration.current === generation) requestGeneration.current += 1;
      };
    }
    if (!target) {
      setBusy(false);
      setStatus("Select a Session to bind the right dock");
      return;
    }
    setBusy(true);
    setStatus(`Binding ${surfaceTitle(surface)} to exact Session…`);
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
        } else if (surface === "changes") {
          const response = await bridge.changes(bound);
          if (generation !== requestGeneration.current) return;
          if (!sameRightDockBinding(bound, response.binding)) {
            setStatus("Discarded stale Changes result · original binding did not match current Session");
            return;
          }
          setChanges(response);
          setStatus(`${response.entries.length} Git change entries · diff is not verification evidence`);
        } else if (surface === "evidence") {
          const response = await bridge.evidence(bound);
          if (generation !== requestGeneration.current) return;
          if (!sameRightDockBinding(bound, response.binding)) {
            setStatus("Discarded stale Evidence result · original binding did not match current Session");
            return;
          }
          setEvidence(response);
          const trusted = response.entries.filter((entry) => entry.trusted).length;
          setStatus(`${trusted} exact-candidate evidence record${trusted === 1 ? "" : "s"} · ${response.entries.length - trusted} stale`);
        } else if (surface === "context") {
          const response = await bridge.context(bound);
          if (generation !== requestGeneration.current) return;
          if (!sameRightDockBinding(bound, response.binding)) {
            setStatus("Discarded stale Context result · original binding did not match current Session");
            return;
          }
          setContext(response);
          setStatus(`${response.facts.length} source-labelled canonical context facts`);
        } else {
          const response = await bridge.artifacts(bound);
          if (generation !== requestGeneration.current) return;
          if (!sameRightDockBinding(bound, response.binding)) {
            setStatus("Discarded stale Artifacts result · original binding did not match current Session");
            return;
          }
          setArtifacts(response);
          setStatus(`${response.entries.length} canonical artifact reference${response.entries.length === 1 ? "" : "s"} · browsing grants no verification authority`);
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
  }, [attentionBridge, bridge, target?.workspaceId, target?.sessionId, surface, truthRefresh]);

  useEffect(() => {
    if (!target || !binding || !isTruthSensitiveSurface(surface)) return;
    let cancelled = false;
    let timer = 0;
    const revalidate = async () => {
      try {
        const current = await bridge.bind(target);
        if (cancelled) return;
        if (!sameRightDockBinding(binding, current)) {
          cancelled = true;
          requestGeneration.current += 1;
          setBinding(null);
          setEvidence(null);
          setContext(null);
          setArtifacts(null);
          setSelectedArtifact(null);
          setStatus("Bound candidate/tree moved · trusted dock treatment removed before refresh");
          setTruthRefresh((value) => value + 1);
          return;
        }
      } catch (error) {
        if (cancelled) return;
        cancelled = true;
        requestGeneration.current += 1;
        setBinding(null);
        setEvidence(null);
        setContext(null);
        setArtifacts(null);
        setSelectedArtifact(null);
        setBusy(false);
        setStatus(`Truth revalidation failed closed · ${errorMessage(error)}`);
        return;
      }
      timer = window.setTimeout(() => void revalidate(), TRUTH_REVALIDATION_MS);
    };
    timer = window.setTimeout(() => void revalidate(), TRUTH_REVALIDATION_MS);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [bridge, target?.workspaceId, target?.sessionId, surface, binding?.bindingDigest]);

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
        {RIGHT_DOCK_SURFACES.map((tab, index) => (
          <button
            type="button"
            role="tab"
            id={`right-dock-tab-${tab}`}
            aria-controls="right-dock-panel"
            aria-selected={surface === tab}
            tabIndex={surface === tab ? 0 : -1}
            className="right-tab"
            data-active={surface === tab ? "true" : "false"}
            ref={(node) => { tabRefs.current[index] = node; }}
            onClick={() => setSurface(tab)}
            onKeyDown={(event) => {
              let nextIndex: number | null = null;
              if (event.key === "ArrowRight") nextIndex = (index + 1) % RIGHT_DOCK_SURFACES.length;
              if (event.key === "ArrowLeft") nextIndex = (index - 1 + RIGHT_DOCK_SURFACES.length) % RIGHT_DOCK_SURFACES.length;
              if (event.key === "Home") nextIndex = 0;
              if (event.key === "End") nextIndex = RIGHT_DOCK_SURFACES.length - 1;
              if (nextIndex === null) return;
              event.preventDefault();
              const nextSurface = RIGHT_DOCK_SURFACES[nextIndex];
              setSurface(nextSurface);
              tabRefs.current[nextIndex]?.focus();
            }}
            key={tab}
          >
            {surfaceTitle(tab)}
          </button>
        ))}
      </div>

      <div id="right-dock-panel" className="right-dock-body" role="tabpanel" aria-labelledby={`right-dock-tab-${surface}`}>
        <div className="right-dock-heading">
          <div><p className="section-kicker">Exact Session scope</p><h2>{surfaceTitle(surface)}</h2></div>
          <span className="dock-source-badge">{bridge.source === "canonical" ? "Rust-owned" : "Fixture"}</span>
        </div>

        {surface !== "needs_you" && (binding ? (
          <dl className="dock-binding" aria-label="Immutable right dock binding">
            <div><dt>Session</dt><dd>{binding.sessionId}</dd></div>
            <div><dt>HEAD</dt><dd>{shortIdentity(binding.headOid)}</dd></div>
            <div><dt>Tree</dt><dd>{shortIdentity(binding.treeOid)}</dd></div>
            <div><dt>Candidate</dt><dd>{binding.candidateOid ? shortIdentity(binding.candidateOid) : "none bound"}</dd></div>
            <div><dt>Stage</dt><dd>{binding.stageRunId ?? "none bound"}</dd></div>
            <div><dt>Binding</dt><dd>{shortIdentity(binding.bindingDigest, 12)}</dd></div>
          </dl>
        ) : <div className="dock-empty-binding">No canonical binding</div>)}

        {surface === "files" && (
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
        )}

        {surface === "changes" && (
          <div className="right-dock-surface changes-surface" aria-busy={busy}>
            <div className="change-list" aria-label="Git change status">
              {changes?.entries.map((entry) => <div className="change-row" key={`${entry.status}-${entry.path}`}><code>{entry.status}</code><span>{displayPath(entry.path)}</span><small>{statusLabel(entry.status)}</small></div>)}
              {!busy && changes && changes.entries.length === 0 && <div className="dock-empty-state">Working tree has no reported changes.</div>}
            </div>
            {changes && <section className="dock-inspector" aria-label="Repository-native diff"><p className="section-kicker">Git diff · untrusted presentation · not verification</p>{changes.diff ? <pre className="changes-diff"><code>{changes.diff}</code></pre> : <p>No staged or unstaged diff content.</p>}{changes.diffLossy && <p className="dock-warning">Diff contained non-UTF-8 bytes and is rendered lossily.</p>}</section>}
          </div>
        )}

        {surface === "evidence" && (
          <div className="right-dock-surface truth-surface" aria-busy={busy}>
            <div className="truth-list" aria-label="Winds verification evidence">
              {evidence?.entries.map((entry) => (
                <article className="truth-row" data-state={entry.freshness} key={entry.runId}>
                  <div className="truth-row-heading"><strong>{entry.runId}</strong><span className="truth-badge">{entry.trusted ? "Exact candidate" : "Stale"}</span></div>
                  <p>{entry.authority} · {entry.eligibility}</p>
                  <code>{shortIdentity(entry.candidateOid, 12)} / {shortIdentity(entry.candidateTree, 12)}</code>
                </article>
              ))}
              {!busy && evidence && evidence.entries.length === 0 && <div className="dock-empty-state">No persisted eligible Winds verification evidence is bound to this stage.</div>}
            </div>
            <p className="dock-trust-note">Only persisted ELIGIBLE Winds evidence for the exact current candidate receives trusted treatment. Agent/model prose never does.</p>
          </div>
        )}

        {surface === "context" && (
          <div className="right-dock-surface truth-surface" aria-busy={busy}>
            <dl className="context-facts" aria-label="Canonical Session and workflow context">
              {context?.facts.map((fact) => (
                <div className="context-fact" key={`${fact.key}-${fact.value}`}><dt>{fact.key.replaceAll("_", " ")}</dt><dd><strong>{displayPath(fact.value)}</strong><small>{fact.source} · {fact.authority}</small></dd></div>
              ))}
              {!busy && context && context.facts.length === 0 && <div className="dock-empty-state">No canonical context facts are available.</div>}
            </dl>
            <p className="dock-trust-note">Context is a source-labelled read-only projection. It does not create runtime, provider, model, evidence, or decision authority.</p>
          </div>
        )}

        {surface === "artifacts" && (
          <div className="right-dock-surface truth-surface" aria-busy={busy}>
            <div className="truth-list" aria-label="Canonical artifact references">
              {artifacts?.entries.map((entry) => (
                <button type="button" className="truth-row artifact-row" data-state={entry.candidateState} aria-pressed={selectedArtifact?.baselineId === entry.baselineId} onClick={() => setSelectedArtifact(entry)} key={entry.baselineId}>
                  <div className="truth-row-heading"><strong>{displayPath(entry.baselineId)}</strong><span className="truth-badge">{entry.candidateState.replaceAll("_", " ")}</span></div>
                  <p>{entry.kind}</p><small>{entry.provenance}</small>
                </button>
              ))}
              {!busy && artifacts && artifacts.entries.length === 0 && <div className="dock-empty-state">No canonical artifact references are bound to this stage.</div>}
            </div>
            {selectedArtifact && <section className="dock-inspector" aria-label="Artifact identity details"><p className="section-kicker">Safe in-dock reveal · no host open</p><strong>{displayPath(selectedArtifact.baselineId)}</strong><dl><div><dt>Reference</dt><dd>{displayPath(selectedArtifact.stableReference)}</dd></div><div><dt>Provenance</dt><dd>{selectedArtifact.provenance}</dd></div><div><dt>Candidate</dt><dd>{selectedArtifact.candidateOid ? shortIdentity(selectedArtifact.candidateOid, 12) : "not candidate-bound"}</dd></div><div><dt>Authority</dt><dd>{selectedArtifact.grantsVerificationAuthority ? "verification authority" : "none from artifact presence"}</dd></div></dl></section>}
            <p className="dock-trust-note">Artifact presence never grants verification authority. Reveal stays inside this bounded dock; no arbitrary host-open capability is exposed.</p>
          </div>
        )}


        {surface === "needs_you" && (
          <div className="right-dock-surface truth-surface" aria-busy={busy}>
            <div className="needs-you-toolbar"><button type="button" className="quiet-action" disabled={busy} onClick={() => setTruthRefresh((value) => value + 1)}>Refresh canonical attention</button></div>
            <div className="truth-list" aria-label="Trusted human attention items">
              {attention?.items.map((item) => (
                <article className="truth-row needs-you-row" data-state={item.state} key={`${item.stageRunId}-${item.sessionId}`}>
                  <div className="truth-row-heading"><strong>{displayPath(item.reason)}</strong><span className="truth-badge">{item.state.replaceAll("_", " ")}</span></div>
                  <dl className="needs-you-context">
                    <div><dt>Project</dt><dd>{displayPath(item.workspaceId)}</dd></div>
                    <div><dt>Session</dt><dd>{displayPath(item.sessionId)}</dd></div>
                    <div><dt>Workflow</dt><dd>{displayPath(item.workflowRunId)}</dd></div>
                    <div><dt>Stage</dt><dd>{displayPath(item.stageKey)} · {displayPath(item.stageRunId)}</dd></div>
                    <div><dt>Candidate</dt><dd>{item.candidateOid ? `${shortIdentity(item.candidateOid, 12)} / ${shortIdentity(item.candidateTree, 12)}` : "none bound"}</dd></div>
                    <div><dt>Truth</dt><dd>{item.source} · {item.authority}</dd></div>
                  </dl>
                  <div className="needs-you-actions">
                    <button type="button" className="quiet-action" onClick={() => onFocusAttention?.(item.workspaceId, item.sessionId)}>Focus exact Session</button>
                    <span>{item.approvalActionAvailable ? "Canonical approval action available" : "Approve/deny unavailable · no authorized desktop seam"}</span>
                  </div>
                </article>
              ))}
              {!busy && attention && attention.items.length === 0 && <div className="dock-empty-state">No trusted canonical state currently requires human attention.</div>}
            </div>
            <p className="dock-trust-note">Needs You is derived only from accepted canonical stage facts. Agent text, ranking, and visual prominence grant no approval or decision authority.</p>
          </div>
        )}

        <p className="dock-status" role="status" aria-live="polite">{status}</p>
      </div>
    </aside>
  );
}

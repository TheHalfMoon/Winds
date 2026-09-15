import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { leftDockBridge } from "../leftDock/bridge";
import type { BridgeSnapshot } from "../leftDock/types";
import { displayPath } from "../rightDock/model";
import type { RightDockSurface } from "../rightDock/types";
import { commandPaletteItems, type CommandPaletteItem } from "./model";

const emptySnapshot: BridgeSnapshot = { projects: [] };

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function CommandPalette({
  open,
  onClose,
  onSelectSession,
  onOpenSurface,
}: {
  readonly open: boolean;
  readonly onClose: () => void;
  readonly onSelectSession: (workspaceId: string, sessionId: string) => void;
  readonly onOpenSurface: (surface: RightDockSurface) => void;
}) {
  const bridge = useMemo(() => leftDockBridge(), []);
  const [snapshot, setSnapshot] = useState<BridgeSnapshot>(emptySnapshot);
  const [query, setQuery] = useState("");
  const [projectScope, setProjectScope] = useState<string | null>(null);
  const [activeIndex, setActiveIndex] = useState(0);
  const [status, setStatus] = useState("Canonical navigation only · no execution authority");
  const inputRef = useRef<HTMLInputElement>(null);
  const dialogRef = useRef<HTMLElement>(null);
  const priorFocusRef = useRef<HTMLElement | null>(null);
  const requestGeneration = useRef(0);

  const refresh = useCallback(async (generation = ++requestGeneration.current) => {
    const next = await bridge.snapshot();
    if (generation !== requestGeneration.current) return;
    setSnapshot(next);
    setStatus(bridge.source === "canonical" ? "Canonical navigation snapshot" : "Fixture navigation snapshot");
  }, [bridge]);

  useEffect(() => {
    if (!open) return;
    const generation = ++requestGeneration.current;
    priorFocusRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    setQuery("");
    setProjectScope(null);
    setActiveIndex(0);
    void refresh(generation).catch((error) => {
      if (generation === requestGeneration.current) setStatus(`Navigation unavailable · ${errorMessage(error)}`);
    });
    const focusTimer = window.setTimeout(() => inputRef.current?.focus(), 0);
    return () => {
      window.clearTimeout(focusTimer);
      requestGeneration.current += 1;
      priorFocusRef.current?.focus();
      priorFocusRef.current = null;
    };
  }, [open, refresh]);

  const items = useMemo(
    () => commandPaletteItems(snapshot, query, projectScope),
    [projectScope, query, snapshot],
  );

  useEffect(() => setActiveIndex(0), [projectScope, query]);

  if (!open) return null;

  const choose = (item: CommandPaletteItem) => {
    if (item.kind === "project" && item.workspaceId) {
      setProjectScope(item.workspaceId);
      setQuery("");
      setStatus("Project selected · choose an exact Session explicitly");
      inputRef.current?.focus();
      return;
    }
    if (item.kind === "session" && item.workspaceId && item.sessionId) {
      onSelectSession(item.workspaceId, item.sessionId);
      onClose();
      return;
    }
    if (item.kind === "surface" && item.surface) {
      onOpenSurface(item.surface);
      onClose();
      return;
    }
    if (item.kind === "action" && item.id === "action:refresh") {
      const generation = ++requestGeneration.current;
      void refresh(generation).catch((error) => {
        if (generation === requestGeneration.current) setStatus(`Refresh failed · ${errorMessage(error)}`);
      });
    }
  };

  return (
    <div className="command-palette-backdrop" role="presentation" onMouseDown={(event) => {
      if (event.currentTarget === event.target) onClose();
    }}>
      <section
        ref={dialogRef}
        className="command-palette"
        role="dialog"
        aria-modal="true"
        aria-label="Winds command palette"
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.preventDefault();
            projectScope ? setProjectScope(null) : onClose();
            return;
          }
          if (event.key !== "Tab") return;
          const focusable = Array.from(dialogRef.current?.querySelectorAll<HTMLElement>(
            'button:not([disabled]), input:not([disabled]), [tabindex]:not([tabindex="-1"])',
          ) ?? []);
          if (focusable.length === 0) return;
          const first = focusable[0];
          const last = focusable[focusable.length - 1];
          if (event.shiftKey && document.activeElement === first) {
            event.preventDefault();
            last.focus();
          } else if (!event.shiftKey && document.activeElement === last) {
            event.preventDefault();
            first.focus();
          }
        }}
      >
        <div className="command-palette-heading">
          <div><p className="section-kicker">Cmd/Ctrl+K</p><h2>{projectScope ? "Choose exact Session" : "Command palette"}</h2></div>
          <button type="button" className="mini-action" onClick={projectScope ? () => setProjectScope(null) : onClose} aria-label={projectScope ? "Back to all commands" : "Close command palette"}>{projectScope ? "←" : "×"}</button>
        </div>
        <input
          ref={inputRef}
          className="command-palette-search"
          type="search"
          value={query}
          onChange={(event) => setQuery(event.currentTarget.value)}
          role="combobox"
          aria-label="Search commands Projects and Sessions"
          aria-expanded="true"
          aria-controls="command-palette-options"
          aria-activedescendant={items[activeIndex] ? `command-option-${activeIndex}` : undefined}
          aria-autocomplete="list"
          placeholder={projectScope ? "Filter Sessions in this Project" : "Search Projects, Sessions, docks, safe actions"}
          onKeyDown={(event) => {
            if (event.key === "ArrowDown" && items.length > 0) { event.preventDefault(); setActiveIndex((value) => (value + 1) % items.length); return; }
            if (event.key === "ArrowUp" && items.length > 0) { event.preventDefault(); setActiveIndex((value) => (value - 1 + items.length) % items.length); return; }
            if (event.key === "Enter" && items[activeIndex]) { event.preventDefault(); choose(items[activeIndex]); }
          }}
        />
        <div id="command-palette-options" className="command-palette-list" role="listbox" aria-label="Available commands">
          {items.map((item, index) => (
            <button
              type="button"
              role="option"
              id={`command-option-${index}`}
              aria-selected={index === activeIndex}
              className="command-palette-item"
              data-active={index === activeIndex ? "true" : "false"}
              onMouseEnter={() => setActiveIndex(index)}
              onClick={() => choose(item)}
              key={item.id}
            >
              <span><strong>{displayPath(item.title)}</strong><small>{displayPath(item.subtitle)}</small></span><code>{item.kind}</code>
            </button>
          ))}
          {items.length === 0 && <p className="dock-empty-state">No unambiguous command target matches this search.</p>}
        </div>
        <p className="command-palette-status" role="status" aria-live="polite">{status}</p>
      </section>
    </div>
  );
}

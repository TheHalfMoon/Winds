import { useEffect, useMemo, useRef, useState } from "react";
import type { FitAddon as XTermFitAddon } from "@xterm/addon-fit";
import type { IDisposable, Terminal as XTermTerminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import { terminalBridge } from "./bridge";
import {
  drainTerminalOutput,
  findLiteralMatch,
  lifecycleLabel,
  reconcileTerminalStatus,
  terminalOutputFlushDelay,
  validatedHttpLink,
} from "./model";
import type { TerminalOutputBatch } from "./model";
import type { TerminalStatus, TerminalTargetRequest } from "./types";
import "./terminal.css";

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function TerminalRuntimeSurface({
  canonicalSessionId,
  visible,
  autoStart = false,
}: {
  readonly canonicalSessionId: string;
  readonly visible: boolean;
  readonly autoStart?: boolean;
}) {
  const bridge = useMemo(() => terminalBridge(), []);
  const hostRef = useRef<HTMLDivElement | null>(null);
  const terminalRef = useRef<XTermTerminal | null>(null);
  const fitRef = useRef<XTermFitAddon | null>(null);
  const lifecycleRef = useRef<TerminalStatus | null>(null);
  const attachedRef = useRef(false);
  const visibleRef = useRef(visible);
  const generationRef = useRef(0);
  const streamGenerationRef = useRef(0);
  const initializingRef = useRef(false);
  const disposablesRef = useRef<IDisposable[]>([]);
  const observerRef = useRef<ResizeObserver | null>(null);
  const pendingOutputRef = useRef<TerminalOutputBatch[]>([]);
  const flushTimerRef = useRef<number | null>(null);
  const [terminalStatus, setTerminalStatus] = useState<TerminalStatus | null>(null);
  const [message, setMessage] = useState(
    bridge.source === "canonical"
      ? "Terminal is idle · Rust owns launch and lifecycle"
      : "Browser fixture · canonical terminal host unavailable",
  );
  const [busy, setBusy] = useState(false);
  const [ready, setReady] = useState(false);
  const [search, setSearch] = useState("");
  const [searchLine, setSearchLine] = useState(0);

  const discardPendingOutput = () => {
    pendingOutputRef.current = [];
    if (flushTimerRef.current !== null) {
      window.clearTimeout(flushTimerRef.current);
      flushTimerRef.current = null;
    }
  };

  const invalidateOutputStream = () => {
    streamGenerationRef.current += 1;
    discardPendingOutput();
  };

  const updateStatus = (status: TerminalStatus, detachOnFinal = false): boolean => {
    const current = lifecycleRef.current;
    const reconciled = reconcileTerminalStatus(current, status);
    if (current && reconciled === current && status !== current) return false;
    const acceptedFinalTransition = reconciled.lifecycle !== "live" && (
      !current
      || current.lifecycle === "live"
      || current.generation !== reconciled.generation
      || current.terminalId !== reconciled.terminalId
    );
    if (acceptedFinalTransition) invalidateOutputStream();
    if (detachOnFinal && reconciled.lifecycle !== "live") attachedRef.current = false;
    lifecycleRef.current = reconciled;
    setTerminalStatus(reconciled);
    const terminal = terminalRef.current;
    if (terminal) terminal.options.disableStdin = reconciled.lifecycle !== "live" || !attachedRef.current;
    setMessage(lifecycleLabel(reconciled));
    return true;
  };

  const flushOutput = (scheduledGeneration: number) => {
    const drained = drainTerminalOutput(
      pendingOutputRef.current,
      scheduledGeneration,
      streamGenerationRef.current,
    );
    pendingOutputRef.current = [...drained.remaining];
    if (drained.chunks.length === 0) return;
    const total = drained.chunks.reduce((sum, chunk) => sum + chunk.byteLength, 0);
    const combined = new Uint8Array(total);
    let offset = 0;
    for (const chunk of drained.chunks) {
      combined.set(chunk, offset);
      offset += chunk.byteLength;
    }
    terminalRef.current?.write(combined);
  };

  const scheduleOutputFlush = (streamGeneration: number, delay: number) => {
    if (flushTimerRef.current !== null) return;
    const timer = window.setTimeout(() => {
      if (flushTimerRef.current === timer) flushTimerRef.current = null;
      flushOutput(streamGeneration);
    }, delay);
    flushTimerRef.current = timer;
  };

  const enqueueOutput = (bytes: Uint8Array, streamGeneration: number) => {
    if (streamGenerationRef.current !== streamGeneration) return;
    pendingOutputRef.current.push({ streamGeneration, bytes });
    scheduleOutputFlush(streamGeneration, terminalOutputFlushDelay(visibleRef.current));
  };

  useEffect(() => {
    visibleRef.current = visible;
    if (visible) {
      window.setTimeout(() => fitRef.current?.fit(), 0);
      if (flushTimerRef.current === null && pendingOutputRef.current.length > 0) {
        scheduleOutputFlush(streamGenerationRef.current, 0);
      }
    }
  }, [visible]);

  useEffect(() => {
    const generation = generationRef.current;
    return () => {
      generationRef.current = generation + 1;
      invalidateOutputStream();
      initializingRef.current = false;
      observerRef.current?.disconnect();
      observerRef.current = null;
      for (const disposable of disposablesRef.current.splice(0)) disposable.dispose();
      terminalRef.current?.dispose();
      terminalRef.current = null;
      fitRef.current = null;
      attachedRef.current = false;
      setReady(false);
    };
  }, [canonicalSessionId]);

  const initializeTerminal = async (): Promise<XTermTerminal> => {
    if (terminalRef.current) return terminalRef.current;
    if (initializingRef.current) throw new Error("Terminal renderer initialization already in progress");
    const host = hostRef.current;
    if (!host) throw new Error("Terminal renderer host unavailable");
    const generation = generationRef.current;
    initializingRef.current = true;

    try {
      const [{ Terminal }, { FitAddon }] = await Promise.all([
        import("@xterm/xterm"),
        import("@xterm/addon-fit"),
      ]);
      if (generationRef.current !== generation) {
        throw new Error("Terminal renderer target changed during initialization");
      }
      if (terminalRef.current) return terminalRef.current;

      const terminal = new Terminal({
        allowProposedApi: false,
        convertEol: false,
        cursorBlink: true,
        disableStdin: true,
        fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
        fontSize: 12,
        scrollback: 5000,
        logLevel: "off",
        linkHandler: {
          allowNonHttpProtocols: false,
          activate: (_event, text) => {
            const link = validatedHttpLink(text);
            setMessage(link
              ? `External link not opened · validated ${link.protocol} link requires a separately authorized host action`
              : "Unsafe terminal link blocked");
          },
        },
      });
      const fit = new FitAddon();
      terminal.loadAddon(fit);
      disposablesRef.current.push(terminal.parser.registerOscHandler(52, () => {
        setMessage("OSC 52 clipboard request blocked · terminal output is untrusted");
        return true;
      }));
      terminal.open(host);
      terminalRef.current = terminal;
      fitRef.current = fit;
      fit.fit();
      setReady(true);

      disposablesRef.current.push(terminal.onData((data) => {
        const current = lifecycleRef.current;
        if (!current || current.lifecycle !== "live" || !attachedRef.current) return;
        const bytes = Array.from(new TextEncoder().encode(data));
        void bridge.input({
          canonicalSessionId,
          terminalId: current.terminalId,
          bytes,
        }).then(updateStatus).catch((error) => {
          if (lifecycleRef.current?.generation === current.generation) {
            setMessage(`Terminal input failed · ${errorMessage(error)}`);
          }
        });
      }));
      disposablesRef.current.push(terminal.onResize(({ rows, cols }) => {
        const current = lifecycleRef.current;
        if (!current || current.lifecycle !== "live" || !attachedRef.current) return;
        void bridge.resize({
          canonicalSessionId,
          terminalId: current.terminalId,
          rows,
          cols,
        }).then(updateStatus).catch((error) => {
          if (lifecycleRef.current?.generation === current.generation) {
            setMessage(`Terminal resize failed · ${errorMessage(error)}`);
          }
        });
      }));
      const observer = new ResizeObserver(() => {
        if (visibleRef.current) fit.fit();
      });
      observer.observe(host);
      observerRef.current = observer;
      return terminal;
    } finally {
      if (generationRef.current === generation) initializingRef.current = false;
    }
  };

  useEffect(() => {
    if (!visible || autoStart) return;
    const generation = generationRef.current;
    void bridge.status(canonicalSessionId)
      .then((status) => {
        if (generationRef.current !== generation || !status) return;
        lifecycleRef.current = status;
        setTerminalStatus(status);
        setMessage(
          status.lifecycle === "live"
            ? "Live Rust terminal already exists · prior output stream cannot be reattached in T135"
            : lifecycleLabel(status),
        );
      })
      .catch((error) => {
        if (generationRef.current === generation) {
          setMessage(`Terminal status failed · ${errorMessage(error)}`);
        }
      });
  }, [bridge, canonicalSessionId, visible]);

  const target = (): TerminalTargetRequest | null => {
    const current = lifecycleRef.current;
    if (!current || current.lifecycle !== "live") return null;
    return { canonicalSessionId, terminalId: current.terminalId };
  };

  const start = async () => {
    if (bridge.source !== "canonical" || busy) return;
    setBusy(true);
    setMessage("Preparing terminal renderer…");
    invalidateOutputStream();
    const streamGeneration = streamGenerationRef.current;
    try {
      const terminal = await initializeTerminal();
      setMessage("Starting Rust-owned terminal…");
      const status = await bridge.start(
        { canonicalSessionId, rows: terminal.rows, cols: terminal.cols },
        {
          onOutput: (bytes) => {
            enqueueOutput(bytes, streamGeneration);
          },
          onLifecycle: (next) => {
            if (streamGenerationRef.current !== streamGeneration) return;
            updateStatus(next, true);
          },
        },
      );
      const accepted = updateStatus(status);
      if (accepted && lifecycleRef.current?.lifecycle === "live") {
        attachedRef.current = true;
        terminal.options.disableStdin = false;
        if (visibleRef.current) terminal.focus();
      }
    } catch (error) {
      setMessage(`Terminal start failed · ${errorMessage(error)}`);
      const existing = await bridge.status(canonicalSessionId).catch(() => null);
      if (existing) updateStatus(existing);
    } finally {
      setBusy(false);
    }
  };

  const autoStartConsumedRef = useRef(false);
  useEffect(() => {
    if (!autoStart || autoStartConsumedRef.current || !visible) return;
    autoStartConsumedRef.current = true;
    void start();
  }, [autoStart, visible]);

  const control = async (kind: "interrupt" | "terminate" | "close") => {
    const current = target();
    if (!current || busy) return;
    setBusy(true);
    try {
      const next = await bridge[kind](current);
      if (kind !== "interrupt") attachedRef.current = false;
      updateStatus(next);
    } catch (error) {
      setMessage(`Terminal ${kind} failed · ${errorMessage(error)}`);
      const next = await bridge.status(canonicalSessionId).catch(() => null);
      if (next) updateStatus(next);
    } finally {
      setBusy(false);
    }
  };

  const searchLiteral = () => {
    const terminal = terminalRef.current;
    if (!terminal || search.length === 0) return;
    const buffer = terminal.buffer.active;
    const lines = Array.from({ length: buffer.length }, (_, line) => buffer.getLine(line)?.translateToString(true) ?? "");
    const match = findLiteralMatch(lines, search, searchLine);
    if (!match) {
      setMessage(`Literal terminal search: no match for “${search}”`);
      return;
    }
    terminal.select(match.column, match.line, match.length);
    terminal.scrollToLine(match.line);
    setSearchLine((match.line + 1) % Math.max(1, lines.length));
    setMessage(`Literal terminal search match · line ${match.line + 1}`);
  };

  const live = terminalStatus?.lifecycle === "live";
  const canStart = bridge.source === "canonical" && !live && !busy;
  return (
    <section className="terminal-surface" hidden={!visible} aria-label={`Terminal for canonical Session ${canonicalSessionId}`}>
      <header className="terminal-trust-bar">
        <div>
          <strong>Terminal output · untrusted</strong>
          <span>Not verification evidence · OSC 52 blocked · no automatic links or file actions</span>
        </div>
        <span className="terminal-lifecycle" data-lifecycle={terminalStatus?.lifecycle ?? "idle"}>
          {lifecycleLabel(terminalStatus)}
        </span>
      </header>
      <div className="terminal-tools">
        <button type="button" className="text-button" disabled={!canStart} onClick={() => void start()}>Start terminal</button>
        <button type="button" className="text-button" disabled={!live || busy} onClick={() => void control("interrupt")}>Interrupt</button>
        <button type="button" className="text-button" disabled={!live || busy} onClick={() => void control("terminate")}>Terminate</button>
        <button type="button" className="text-button" disabled={!live || busy} onClick={() => void control("close")}>Close terminal</button>
        <label className="terminal-search">
          <span>Literal search</span>
          <input
            value={search}
            disabled={!ready}
            onChange={(event) => { setSearch(event.currentTarget.value); setSearchLine(0); }}
            onKeyDown={(event) => { if (event.key === "Enter") { event.preventDefault(); searchLiteral(); } }}
            placeholder={ready ? "Exact text" : "Start terminal to load renderer"}
          />
        </label>
        <button type="button" className="text-button" disabled={!ready || !search} onClick={searchLiteral}>Find next</button>
      </div>
      <div className="terminal-host" ref={hostRef} aria-label="Untrusted interactive terminal output" />
      <p className="terminal-status" role="status" aria-live="polite">{message}</p>
    </section>
  );
}

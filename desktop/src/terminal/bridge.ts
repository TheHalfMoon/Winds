import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import type {
  TerminalBridge,
  TerminalInputRequest,
  TerminalResizeRequest,
  TerminalStartRequest,
  TerminalStatus,
  TerminalStreamHandlers,
  TerminalTargetRequest,
} from "./types";

function unavailable(): Promise<never> {
  return Promise.reject(new Error("Canonical Tauri terminal host is unavailable in browser fixture mode"));
}

function fixtureBridge(): TerminalBridge {
  return {
    source: "fixture",
    status: async () => null,
    start: unavailable,
    input: unavailable,
    resize: unavailable,
    interrupt: unavailable,
    terminate: unavailable,
    close: unavailable,
  };
}

function canonicalBridge(): TerminalBridge {
  return {
    source: "canonical",
    status: (canonicalSessionId: string) => invoke("terminal_status", { canonicalSessionId }),
    start: (request: TerminalStartRequest, handlers: TerminalStreamHandlers) => {
      const output = new Channel<ArrayBuffer>((payload) => handlers.onOutput(new Uint8Array(payload)));
      const lifecycle = new Channel<TerminalStatus>(handlers.onLifecycle);
      return invoke("terminal_start", { request, output, lifecycle });
    },
    input: (request: TerminalInputRequest) => invoke("terminal_input", { request }),
    resize: (request: TerminalResizeRequest) => invoke("terminal_resize", { request }),
    interrupt: (request: TerminalTargetRequest) => invoke("terminal_interrupt", { request }),
    terminate: (request: TerminalTargetRequest) => invoke("terminal_terminate", { request }),
    close: (request: TerminalTargetRequest) => invoke("terminal_close", { request }),
  };
}

export function terminalBridge(): TerminalBridge {
  return isTauri() ? canonicalBridge() : fixtureBridge();
}

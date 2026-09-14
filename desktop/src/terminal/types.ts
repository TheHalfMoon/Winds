export type TerminalLifecycle = "live" | "exited" | "interrupted" | "ownership_lost";

export interface TerminalStatus {
  readonly terminalId: string;
  readonly generation: number;
  readonly canonicalSessionId: string;
  readonly canonicalWorkspaceId: string;
  readonly profileId: string;
  readonly profileDisplayName: string;
  readonly lifecycle: TerminalLifecycle;
  readonly rows: number;
  readonly cols: number;
  readonly exitCode: number | null;
  readonly signal: string | null;
  readonly closeReason: string | null;
}

export interface TerminalStartRequest {
  readonly canonicalSessionId: string;
  readonly rows: number;
  readonly cols: number;
}

export interface TerminalTargetRequest {
  readonly canonicalSessionId: string;
  readonly terminalId: string;
}

export interface TerminalInputRequest extends TerminalTargetRequest {
  readonly bytes: readonly number[];
}

export interface TerminalResizeRequest extends TerminalTargetRequest {
  readonly rows: number;
  readonly cols: number;
}

export interface TerminalStreamHandlers {
  readonly onOutput: (bytes: Uint8Array) => void;
  readonly onLifecycle: (status: TerminalStatus) => void;
}

export interface TerminalBridge {
  readonly source: "canonical" | "fixture";
  readonly status: (canonicalSessionId: string) => Promise<TerminalStatus | null>;
  readonly start: (request: TerminalStartRequest, handlers: TerminalStreamHandlers) => Promise<TerminalStatus>;
  readonly input: (request: TerminalInputRequest) => Promise<TerminalStatus>;
  readonly resize: (request: TerminalResizeRequest) => Promise<TerminalStatus>;
  readonly interrupt: (request: TerminalTargetRequest) => Promise<TerminalStatus>;
  readonly terminate: (request: TerminalTargetRequest) => Promise<TerminalStatus>;
  readonly close: (request: TerminalTargetRequest) => Promise<TerminalStatus>;
}

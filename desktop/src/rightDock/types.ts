export type RightDockSurface = "files" | "changes";

export interface RightDockTarget {
  readonly workspaceId: string;
  readonly sessionId: string;
}

export interface RightDockBinding {
  readonly workspaceId: string;
  readonly sessionId: string;
  readonly worktreeRoot: string;
  readonly gitCommonDir: string;
  readonly workflowRunId: string | null;
  readonly stageRunId: string | null;
  readonly candidateOid: string | null;
  readonly candidateTree: string | null;
  readonly headOid: string | null;
  readonly treeOid: string | null;
  readonly worktreeStateSha256: string;
  readonly bindingDigest: string;
}

export type RightDockFileKind = "file" | "symlink";

export interface RightDockFileEntry {
  readonly path: string;
  readonly kind: RightDockFileKind;
}

export interface RightDockFilesResponse {
  readonly binding: RightDockBinding;
  readonly entries: readonly RightDockFileEntry[];
  readonly truncated: boolean;
}

export type FilePreviewState = "text" | "binary" | "too_large";

export interface FilePreviewResponse {
  readonly binding: RightDockBinding;
  readonly path: string;
  readonly state: FilePreviewState;
  readonly content: string | null;
  readonly byteLen: number;
}

export interface RightDockChangeEntry {
  readonly path: string;
  readonly status: string;
}

export interface RightDockChangesResponse {
  readonly binding: RightDockBinding;
  readonly entries: readonly RightDockChangeEntry[];
  readonly diff: string;
  readonly diffLossy: boolean;
}

export interface RightDockBridge {
  readonly source: "canonical" | "fixture";
  bind(target: RightDockTarget): Promise<RightDockBinding>;
  files(binding: RightDockBinding): Promise<RightDockFilesResponse>;
  preview(binding: RightDockBinding, path: string): Promise<FilePreviewResponse>;
  changes(binding: RightDockBinding): Promise<RightDockChangesResponse>;
}

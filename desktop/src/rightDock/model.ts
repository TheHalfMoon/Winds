import type { RightDockBinding } from "./types";

export function sameRightDockBinding(left: RightDockBinding | null, right: RightDockBinding | null): boolean {
  if (!left || !right) return left === right;
  return left.bindingDigest === right.bindingDigest
    && left.workspaceId === right.workspaceId
    && left.sessionId === right.sessionId
    && left.worktreeRoot === right.worktreeRoot
    && left.gitCommonDir === right.gitCommonDir
    && left.workflowRunId === right.workflowRunId
    && left.stageRunId === right.stageRunId
    && left.candidateOid === right.candidateOid
    && left.candidateTree === right.candidateTree
    && left.headOid === right.headOid
    && left.treeOid === right.treeOid
    && left.worktreeStateSha256 === right.worktreeStateSha256;
}

export function shortIdentity(value: string | null, width = 10): string {
  if (!value) return "unborn";
  return value.length <= width ? value : value.slice(0, width);
}

export function statusLabel(status: string): string {
  if (status === "??") return "untracked";
  const index = status[0] ?? " ";
  const worktree = status[1] ?? " ";
  const labels: string[] = [];
  if (index !== " ") labels.push(`index ${index}`);
  if (worktree !== " ") labels.push(`worktree ${worktree}`);
  return labels.join(" · ") || "unchanged";
}


const UNSAFE_FORMAT_CODE_POINTS = new Set([
  0x061c, 0x200e, 0x200f,
  0x202a, 0x202b, 0x202c, 0x202d, 0x202e,
  0x2066, 0x2067, 0x2068, 0x2069,
]);

export function displayPath(path: string): string {
  let displayed = "";
  for (const character of path) {
    const codePoint = character.codePointAt(0) ?? 0;
    const unsafeControl = codePoint < 0x20 || (codePoint >= 0x7f && codePoint <= 0x9f);
    if (unsafeControl || UNSAFE_FORMAT_CODE_POINTS.has(codePoint)) {
      displayed += `\\u{${codePoint.toString(16).toUpperCase()}}`;
    } else {
      displayed += character;
    }
  }
  return displayed;
}

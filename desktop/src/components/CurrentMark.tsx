export function CurrentMark({ compact = false }: { readonly compact?: boolean }) {
  const size = compact ? 22 : 26;
  return (
    <svg
      className="current-mark"
      width={size}
      height={size}
      viewBox="0 0 28 28"
      role="img"
      aria-label="Winds Current Mark"
    >
      <path className="current-mark-line current-mark-line-a" d="M4 9.2c4.2-3.1 8.4-3.1 12.6 0 2.4 1.8 4.8 1.8 7.2 0" />
      <path className="current-mark-line current-mark-line-b" d="M4 14c3.2-2.2 6.4-2.2 9.6 0 3.4 2.3 6.8 2.3 10.2 0" />
      <path className="current-mark-line current-mark-line-c" d="M4 18.8c2.4-1.5 4.8-1.5 7.2 0 4.2 2.7 8.4 2.7 12.6 0" />
    </svg>
  );
}

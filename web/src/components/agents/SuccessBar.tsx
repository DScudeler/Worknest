interface Props {
  /** 0..1 */
  value: number;
}

export function SuccessBar({ value }: Props) {
  const pct = Math.round(Math.max(0, Math.min(1, value)) * 100);
  const color = pct >= 90 ? "#10b981" : pct >= 75 ? "#f59e0b" : "#ef4444";
  return (
    <span className="success-bar">
      <span className="track">
        <div style={{ width: `${pct}%`, background: color }} />
      </span>
      <span style={{ fontSize: 12, color: "var(--text-2)", fontWeight: 600 }}>{pct}%</span>
    </span>
  );
}

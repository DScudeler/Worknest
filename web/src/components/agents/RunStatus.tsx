import type { CSSProperties } from "react";
import { agentStatusLabel, type AgentStatus, toDisplayAgentStatus } from "../../lib/types";

const COLOR: Record<string, string> = {
  running: "#10b981",
  paused: "#f59e0b",
  idle: "#94a3b8",
  stopped: "#64748b",
  error: "#ef4444",
  activating: "#94a3b8",
};

interface Props {
  status: AgentStatus;
  withPulse?: boolean;
}

export function RunStatus({ status, withPulse = true }: Props) {
  const display = toDisplayAgentStatus(status);
  const dotColor = COLOR[display] ?? "#94a3b8";
  const dotStyle: CSSProperties = { background: dotColor };
  const showPulse = withPulse && display === "running";
  return (
    <span className={`run-status rs-${display}`}>
      <span className="dot" style={dotStyle}>
        {showPulse ? <span className="pulse" style={{ background: dotColor }} /> : null}
      </span>
      <span>{agentStatusLabel(status)}</span>
    </span>
  );
}

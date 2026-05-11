import { AlertCircle, X } from "lucide-react";
import { useEffect } from "react";

import { AgentAvatar } from "./AgentAvatar";
import { RunControls } from "./RunControls";
import { RunStatus } from "./RunStatus";
import { SuccessBar } from "./SuccessBar";
import {
  type DeploymentControl,
  useAgentDeployment,
  useAgentDeploymentEvents,
  useAgentDeploymentTicks,
  useUpdateDeploymentStatus,
} from "../../lib/agentsHooks";
import { CAPABILITIES, type AgentDeploymentId, type Capability } from "../../lib/types";

interface Props {
  deploymentId: AgentDeploymentId;
  onClose: () => void;
}

function formatTime(iso: string): string {
  const d = new Date(iso);
  const now = new Date();
  const sameDay = d.toDateString() === now.toDateString();
  if (sameDay) {
    return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }
  return `${d.toLocaleDateString([], { day: "2-digit", month: "short" })} ${d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}`;
}

function formatRelative(iso: string | null): string {
  if (!iso) return "—";
  const t = new Date(iso).getTime();
  const diff = Date.now() - t;
  const sec = Math.floor(diff / 1000);
  if (sec < 60) return `${sec}s ago`;
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min}m ago`;
  const hr = Math.floor(min / 60);
  if (hr < 24) return `${hr}h ago`;
  const day = Math.floor(hr / 24);
  return `${day}d ago`;
}

function capabilityLabel(c: Capability): string {
  return CAPABILITIES.find((x) => x.id === c)?.label ?? c;
}

export function RunDetailDrawer({ deploymentId, onClose }: Props) {
  const detailQ = useAgentDeployment(deploymentId);
  const ticksQ = useAgentDeploymentTicks(deploymentId);
  const eventsQ = useAgentDeploymentEvents(deploymentId);
  const update = useUpdateDeploymentStatus();

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const deployment = detailQ.data;
  if (!deployment) {
    return (
      <>
        <div className="scrim" onClick={onClose} />
        <div className="sheet">
          <div className="sheet-head">
            <button className="theme-toggle" onClick={onClose} aria-label="Close">
              <X size={16} />
            </button>
          </div>
          <div className="center-page" style={{ flex: 1 }}>
            <span className="spinner" />
          </div>
        </div>
      </>
    );
  }

  const onAction = (action: DeploymentControl) =>
    update.mutate({ deployment, action });

  return (
    <>
      <div className="scrim" onClick={onClose} />
      <div className="sheet" role="dialog" aria-label="Agent run details">
        <div className="sheet-head" style={{ display: "flex", gap: 12, alignItems: "center" }}>
          <AgentAvatar emoji={deployment.persona.emoji} color={deployment.persona.color} size="md" />
          <div style={{ flex: 1 }}>
            <div style={{ fontWeight: 700, fontSize: 16 }}>{deployment.persona.name}</div>
            <div style={{ fontSize: 12, color: "var(--text-3)" }}>
              {deployment.persona.role} · project {deployment.project_id.slice(0, 8)}
            </div>
          </div>
          <button className="theme-toggle" onClick={onClose} aria-label="Close">
            <X size={16} />
          </button>
        </div>
        <div className="sheet-body" style={{ display: "block", overflowY: "auto" }}>
          <div className="run-hero">
            <RunStatus status={deployment.status} />
            <RunControls
              deployment={deployment}
              onAction={onAction}
              busy={update.isPending}
            />
          </div>

          {deployment.status === "Error" && deployment.error_message ? (
            <div className="error-card" style={{ marginBottom: 18 }}>
              <div className="ec-head">
                <AlertCircle />
                {deployment.last_error_step
                  ? `Activation failed at ${deployment.last_error_step}`
                  : "Tick failures crossed the threshold"}
              </div>
              <div className="ec-body">{deployment.error_message}</div>
              <div className="ec-actions">
                <button
                  className="btn secondary"
                  onClick={() => onAction("retry")}
                  disabled={update.isPending}
                >
                  Retry
                </button>
              </div>
            </div>
          ) : null}

          <div className="run-kpis">
            <div className="rk">
              <div className="rk-l">Runs today</div>
              <div className="rk-v">{deployment.runs_today}</div>
            </div>
            <div className="rk">
              <div className="rk-l">Tickets touched</div>
              <div className="rk-v">{deployment.touched_this_week}</div>
            </div>
            <div className="rk">
              <div className="rk-l">Success rate</div>
              <div className="rk-v"><SuccessBar value={deployment.success_rate} /></div>
            </div>
            <div className="rk">
              <div className="rk-l">Last activity</div>
              <div className="rk-v" style={{ fontSize: 14 }}>
                {formatRelative(deployment.last_activity_at)}
              </div>
            </div>
          </div>

          <div style={{ marginTop: 18 }}>
            <div className="section-h">Permissions</div>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 6, marginTop: 6 }}>
              {(deployment.snapshot_capabilities.length > 0
                ? deployment.snapshot_capabilities
                : deployment.persona.capabilities
              ).map((c) => (
                <span key={c} className="cap-chip">
                  {capabilityLabel(c)}
                </span>
              ))}
            </div>
          </div>

          <div style={{ marginTop: 18 }}>
            <div className="section-h">Recent ticks</div>
            <div className="event-log">
              {ticksQ.data?.length ? (
                ticksQ.data.map((t) => (
                  <div key={t.id} className="ev">
                    <span className="ev-time">{formatTime(t.started_at)}</span>
                    <span className="ev-dot" />
                    <span className="ev-body">
                      {t.outcome === "Success"
                        ? t.action_summary || "tick succeeded"
                        : t.outcome === "Failure"
                          ? `failed: ${t.error_message ?? "unknown"}`
                          : "tick in flight"}
                    </span>
                  </div>
                ))
              ) : (
                <div style={{ color: "var(--text-3)", fontSize: 13, padding: "8px 0" }}>
                  No ticks yet.
                </div>
              )}
            </div>
          </div>

          <div style={{ marginTop: 18 }}>
            <div className="section-h">Lifecycle events</div>
            <div className="event-log">
              {eventsQ.data?.map((evt) => (
                <div key={evt.id} className="ev">
                  <span className="ev-time">{formatTime(evt.at)}</span>
                  <span className="ev-dot" />
                  <span className="ev-body">{evt.message}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </>
  );
}

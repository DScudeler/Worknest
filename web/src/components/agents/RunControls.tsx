import type { AgentDeploymentResponse } from "../../lib/types";
import type { DeploymentControl } from "../../lib/agentsHooks";

interface Props {
  deployment: AgentDeploymentResponse;
  onAction: (action: DeploymentControl) => void;
  busy?: boolean;
}

interface Btn {
  action: DeploymentControl;
  label: string;
  variant: "ok" | "warn" | "bad";
}

function buttonsForStatus(deployment: AgentDeploymentResponse): Btn[] {
  switch (deployment.status) {
    case "Running":
      return [
        { action: "suspend", label: "Suspend", variant: "warn" },
        { action: "stop", label: "Stop", variant: "bad" },
      ];
    case "Paused":
      return [
        { action: "resume", label: "Resume", variant: "ok" },
        { action: "stop", label: "Stop", variant: "bad" },
      ];
    case "Stopped":
      return [{ action: "resume", label: "Resume", variant: "ok" }];
    case "Error":
      return [
        { action: "retry", label: "Retry", variant: "ok" },
        { action: "stop", label: "Stop", variant: "bad" },
      ];
    case "Idle":
      return [{ action: "stop", label: "Stop", variant: "bad" }];
    // While the activation pipeline runs, don't surface controls.
    default:
      return [];
  }
}

export function RunControls({ deployment, onAction, busy = false }: Props) {
  const buttons = buttonsForStatus(deployment);
  if (buttons.length === 0) return null;
  return (
    <span className="run-controls">
      {buttons.map((b) => (
        <button
          key={b.action}
          type="button"
          className={`ctl ctl-${b.variant}`}
          disabled={busy}
          onClick={(e) => {
            e.stopPropagation();
            onAction(b.action);
          }}
        >
          {b.label}
        </button>
      ))}
    </span>
  );
}

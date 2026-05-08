import { useState } from "react";

import { Modal } from "../Modal";
import { useDeployPersona } from "../../lib/agentsHooks";
import type { Persona, Project } from "../../lib/types";
import { AgentAvatar } from "./AgentAvatar";

interface Props {
  open: boolean;
  onClose: () => void;
  persona: Persona | null;
  projects: Project[];
}

export function DeployModal({ open, onClose, persona, projects }: Props) {
  const [projectId, setProjectId] = useState<string>("");
  const [cron, setCron] = useState<string>("");
  const deploy = useDeployPersona();

  if (!persona) return null;

  const submit = async () => {
    if (!projectId) return;
    await deploy.mutateAsync({
      projectId,
      data: { persona_id: persona.id, cron_expression: cron.trim() || undefined },
    });
    onClose();
  };

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={`Deploy ${persona.name}`}
      foot={
        <>
          <button className="btn ghost" onClick={onClose}>
            Cancel
          </button>
          <button
            className="btn primary"
            onClick={submit}
            disabled={!projectId || deploy.isPending}
          >
            {deploy.isPending ? "Deploying…" : "Deploy"}
          </button>
        </>
      }
    >
      <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
        <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
          <AgentAvatar emoji={persona.emoji} color={persona.color} size="md" />
          <div>
            <div style={{ fontWeight: 600 }}>{persona.name}</div>
            <div style={{ color: "var(--text-3)", fontSize: 13 }}>{persona.description}</div>
          </div>
        </div>
        <div>
          <div className="field-label">Project</div>
          <select
            className="input"
            value={projectId}
            onChange={(e) => setProjectId(e.target.value)}
          >
            <option value="" disabled>
              Pick a project…
            </option>
            {projects
              .filter((p) => !p.archived)
              .map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
          </select>
        </div>
        <div>
          <div className="field-label">Cron (optional, 5-field UTC)</div>
          <input
            className="input"
            placeholder={persona.default_cron}
            value={cron}
            onChange={(e) => setCron(e.target.value)}
            style={{ fontFamily: "ui-monospace, Menlo, monospace" }}
          />
        </div>
      </div>
    </Modal>
  );
}

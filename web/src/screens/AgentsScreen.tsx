import { Activity, Bot, Pencil, Plus, Rocket, Trash2 } from "lucide-react";
import { useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";

import { AgentAvatar } from "../components/agents/AgentAvatar";
import { DeployModal } from "../components/agents/DeployModal";
import { PersonaEditorModal } from "../components/agents/PersonaEditorModal";
import { RunControls } from "../components/agents/RunControls";
import { RunDetailDrawer } from "../components/agents/RunDetailDrawer";
import { RunStatus } from "../components/agents/RunStatus";
import { SuccessBar } from "../components/agents/SuccessBar";
import { Spinner } from "../components/Spinner";
import { Topbar } from "../components/Topbar";
import {
  type DeploymentControl,
  useAllAgentDeployments,
  useDeletePersona,
  usePersonas,
  useUpdateDeploymentStatus,
} from "../lib/agentsHooks";
import { CAPABILITIES, type Persona } from "../lib/types";

type Tab = "library" | "deployments";

export function AgentsScreen() {
  const [params, setParams] = useSearchParams();
  const tab: Tab = params.get("tab") === "deployments" ? "deployments" : "library";
  const editId = params.get("edit");
  const isNew = params.get("new") === "1";
  const runId = params.get("run");

  const personasQ = usePersonas();
  const personas = personasQ.data ?? [];
  const editing = editId ? personas.find((p) => p.id === editId) ?? null : null;

  const setTab = (next: Tab) => {
    const p = new URLSearchParams(params);
    p.set("tab", next);
    setParams(p, { replace: true });
  };
  const closeEditor = () => {
    const p = new URLSearchParams(params);
    p.delete("edit");
    p.delete("new");
    setParams(p, { replace: true });
  };
  const openCreate = () => {
    const p = new URLSearchParams(params);
    p.delete("edit");
    p.set("new", "1");
    setParams(p, { replace: true });
  };
  const openEdit = (id: string) => {
    const p = new URLSearchParams(params);
    p.delete("new");
    p.set("edit", id);
    setParams(p, { replace: true });
  };
  const openRun = (id: string) => {
    const p = new URLSearchParams(params);
    p.set("run", id);
    setParams(p, { replace: true });
  };
  const closeRun = () => {
    const p = new URLSearchParams(params);
    p.delete("run");
    setParams(p, { replace: true });
  };

  const { deployments, isLoading: deploysLoading } = useAllAgentDeployments();

  const deploymentsCount = deployments.length;

  return (
    <div className="content">
      <Topbar crumbs={[{ label: "Workspace" }, { label: "Agents" }]} />

      <div
        className="page-head"
        style={{ display: "flex", alignItems: "center", gap: 12 }}
      >
        <div>
          <h1 style={{ margin: 0 }}>Agents</h1>
          <div className="muted">
            Workspace-shared persona library and per-project deployments.
          </div>
        </div>
        <span style={{ flex: 1 }} />
        <button type="button" className="btn primary" onClick={openCreate}>
          <Plus size={14} /> New agent
        </button>
      </div>

      <div className="tabs" style={{ margin: "8px 0 18px" }}>
        <button
          type="button"
          className={tab === "library" ? "active" : ""}
          onClick={() => setTab("library")}
        >
          <Bot size={14} />
          Available agents
          <span className="count">{personas.length}</span>
        </button>
        <button
          type="button"
          className={tab === "deployments" ? "active" : ""}
          onClick={() => setTab("deployments")}
        >
          <Activity size={14} />
          In projects
          <span className="count">{deploymentsCount}</span>
        </button>
      </div>

      {tab === "library" ? (
        <Library
          personas={personas}
          loading={personasQ.isLoading}
          onCreate={openCreate}
          onEdit={(p) => openEdit(p.id)}
        />
      ) : (
        <DeploymentsTab
          loading={deploysLoading}
          onOpenRun={(id) => openRun(id)}
        />
      )}

      <PersonaEditorModal
        open={isNew || !!editing}
        onClose={closeEditor}
        initial={editing}
        templates={personas}
      />

      {runId ? <RunDetailDrawer deploymentId={runId} onClose={closeRun} /> : null}
    </div>
  );
}

interface LibraryProps {
  personas: Persona[];
  loading: boolean;
  onCreate: () => void;
  onEdit: (p: Persona) => void;
}

function Library({ personas, loading, onCreate, onEdit }: LibraryProps) {
  const del = useDeletePersona();
  const [deployingPersonaId, setDeployingPersonaId] = useState<string | null>(null);
  const onDelete = (p: Persona) => {
    if (!window.confirm(`Delete agent "${p.name}"? This cannot be undone.`)) return;
    del.mutate(p.id);
  };
  const onDeploy = (p: Persona) => setDeployingPersonaId(p.id);

  if (loading) {
    return (
      <div style={{ padding: 40 }} className="center-page">
        <Spinner />
      </div>
    );
  }

  return (
    <>
      <div
        className="agent-grid"
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fill, minmax(360px, 1fr))",
          gap: 14,
        }}
      >
        {personas.map((p) => (
          <PersonaCard
            key={p.id}
            persona={p}
            onEdit={() => onEdit(p)}
            onDelete={() => onDelete(p)}
            onDeploy={() => onDeploy(p)}
          />
        ))}
        <button
          type="button"
          onClick={onCreate}
          className="agent-card create"
          style={{
            border: "1px dashed var(--border-strong)",
            borderRadius: 14,
            padding: 18,
            background: "transparent",
            color: "var(--text-3)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            gap: 8,
            cursor: "pointer",
            minHeight: 220,
          }}
        >
          <Plus size={16} /> New agent
        </button>
      </div>
      <DeployModalRouter
        deployingPersonaId={deployingPersonaId}
        personas={personas}
        onClose={() => setDeployingPersonaId(null)}
      />
    </>
  );
}

interface PersonaCardProps {
  persona: Persona;
  onEdit: () => void;
  onDelete: () => void;
  onDeploy: () => void;
}

function PersonaCard({ persona, onEdit, onDelete, onDeploy }: PersonaCardProps) {
  return (
    <div
      className="agent-card"
      style={{
        background: "var(--surface)",
        border: "1px solid var(--border)",
        borderRadius: 14,
        padding: 18,
        display: "flex",
        flexDirection: "column",
        gap: 10,
      }}
    >
      <div style={{ display: "flex", gap: 12, alignItems: "flex-start" }}>
        <AgentAvatar emoji={persona.emoji} color={persona.color} size="lg" />
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <h3 style={{ margin: 0, fontSize: 15 }}>{persona.name}</h3>
            <span style={{ fontSize: 11, color: "var(--text-3)" }}>· {persona.role}</span>
          </div>
          <div style={{ color: "var(--text-2)", fontSize: 13, marginTop: 4 }}>
            {persona.description}
          </div>
          <div style={{ fontSize: 12.5, fontStyle: "italic", color: "var(--text-2)", marginTop: 6 }}>
            {persona.tone}
          </div>
        </div>
        <div style={{ display: "flex", gap: 4 }}>
          <button
            className="theme-toggle"
            title="Edit"
            onClick={onEdit}
          >
            <Pencil size={14} />
          </button>
          <button className="theme-toggle" title="Delete" onClick={onDelete}>
            <Trash2 size={14} />
          </button>
        </div>
      </div>

      {persona.expertise.length > 0 ? (
        <div style={{ display: "flex", flexWrap: "wrap", gap: 4 }}>
          {persona.expertise.map((e) => (
            <span key={e} className="exp-chip">
              {e}
            </span>
          ))}
        </div>
      ) : null}

      <div style={{ display: "flex", flexWrap: "wrap", gap: 4 }}>
        {persona.capabilities.map((c) => (
          <span key={c} className="cap-chip">
            {CAPABILITIES.find((x) => x.id === c)?.label ?? c}
          </span>
        ))}
      </div>

      <div
        style={{
          marginTop: "auto",
          display: "flex",
          alignItems: "center",
          gap: 8,
          paddingTop: 8,
          borderTop: "1px solid var(--border)",
          fontSize: 12,
          color: "var(--text-3)",
        }}
      >
        <span>⚡ {persona.model}</span>
        <span style={{ flex: 1 }} />
        <button className="btn secondary" type="button" onClick={onDeploy}>
          <Rocket size={13} /> Deploy
        </button>
      </div>
    </div>
  );
}

interface DeploymentsTabProps {
  loading: boolean;
  onOpenRun: (id: string) => void;
}

function DeploymentsTab({ loading, onOpenRun }: DeploymentsTabProps) {
  const { deployments, projects } = useAllAgentDeployments();
  const update = useUpdateDeploymentStatus();
  const onAction = (action: DeploymentControl) => (deployment: typeof deployments[number]) =>
    update.mutate({ deployment, action });
  const projectsById = useMemo(
    () => new Map(projects.map((p) => [p.id, p])),
    [projects],
  );

  const totals = useMemo(() => {
    const t = { running: 0, paused: 0, error: 0, today: 0 };
    for (const d of deployments) {
      if (d.status === "Running") t.running++;
      if (d.status === "Paused") t.paused++;
      if (d.status === "Error") t.error++;
      t.today += d.runs_today;
    }
    return t;
  }, [deployments]);

  if (loading && deployments.length === 0) {
    return (
      <div style={{ padding: 40 }} className="center-page">
        <Spinner />
      </div>
    );
  }

  if (deployments.length === 0) {
    return (
      <div className="card" style={{ padding: 32, textAlign: "center", color: "var(--text-3)" }}>
        No deployments yet. Click <strong>Deploy</strong> on any persona in the Library tab.
      </div>
    );
  }

  // Group by project.
  const grouped = new Map<string, typeof deployments>();
  for (const d of deployments) {
    const arr = grouped.get(d.project_id) ?? [];
    arr.push(d);
    grouped.set(d.project_id, arr);
  }

  return (
    <>
      <div className="run-stats">
        <div className="run-stat">
          <span className="rs-label">
            <span className="rs-dot" style={{ background: "#10b981" }} /> Running
          </span>
          <span className="rs-val">{totals.running}</span>
        </div>
        <div className="run-stat">
          <span className="rs-label">
            <span className="rs-dot" style={{ background: "#f59e0b" }} /> Suspended
          </span>
          <span className="rs-val">{totals.paused}</span>
        </div>
        <div className="run-stat">
          <span className="rs-label">
            <span className="rs-dot" style={{ background: "#ef4444" }} /> Error
          </span>
          <span className="rs-val">{totals.error}</span>
        </div>
        <div className="run-stat">
          <span className="rs-label">
            <span className="rs-dot" style={{ background: "var(--accent-500)" }} /> Runs today
          </span>
          <span className="rs-val">{totals.today}</span>
        </div>
      </div>

      <div className="run-groups" style={{ marginTop: 16 }}>
        {Array.from(grouped.entries()).map(([pid, rows]) => {
          const project = projectsById.get(pid);
          return (
            <div key={pid} className="run-group">
              <div className="rg-head" style={{ display: "flex", alignItems: "center", gap: 10, padding: "12px 16px", borderBottom: "1px solid var(--border)" }}>
                <strong>{project?.name ?? pid.slice(0, 8)}</strong>
                <span style={{ color: "var(--text-3)", fontSize: 12 }}>
                  · {rows.length} agent{rows.length === 1 ? "" : "s"}
                </span>
              </div>
              <div className="run-table">
                <div
                  className="rt-head"
                  style={{
                    display: "grid",
                    gridTemplateColumns: "1.6fr 1.1fr 1fr 1.1fr 1.1fr 1.1fr 1.6fr",
                    fontSize: 11,
                    color: "var(--text-3)",
                    textTransform: "uppercase",
                    letterSpacing: "0.06em",
                    padding: "10px 16px",
                    borderBottom: "1px solid var(--border)",
                  }}
                >
                  <span>Agent</span>
                  <span>Status</span>
                  <span>Current</span>
                  <span>Today · week</span>
                  <span>Success</span>
                  <span>Last activity</span>
                  <span style={{ textAlign: "right" }}>Controls</span>
                </div>
                {rows.map((d) => (
                  <div
                    key={d.id}
                    className="rt-row"
                    onClick={() => onOpenRun(d.id)}
                    style={{
                      display: "grid",
                      gridTemplateColumns: "1.6fr 1.1fr 1fr 1.1fr 1.1fr 1.1fr 1.6fr",
                      padding: "12px 16px",
                      borderBottom: "1px solid var(--border)",
                      alignItems: "center",
                      cursor: "pointer",
                    }}
                  >
                    <span style={{ display: "flex", gap: 10, alignItems: "center", minWidth: 0 }}>
                      <AgentAvatar emoji={d.persona.emoji} color={d.persona.color} size="sm" />
                      <span style={{ overflow: "hidden", textOverflow: "ellipsis" }}>
                        <div style={{ fontWeight: 600, fontSize: 13.5 }}>{d.persona.name}</div>
                        <div style={{ fontSize: 11.5, color: "var(--text-3)" }}>{d.persona.role}</div>
                      </span>
                    </span>
                    <span>
                      <RunStatus status={d.status} />
                    </span>
                    <span style={{ fontSize: 13, color: "var(--text-2)" }}>
                      {d.current_ticket_id ? d.current_ticket_id.slice(0, 8) : "—"}
                    </span>
                    <span style={{ fontSize: 13, color: "var(--text-2)" }}>
                      {d.runs_today} · {d.touched_this_week}
                    </span>
                    <span><SuccessBar value={d.success_rate} /></span>
                    <span style={{ fontSize: 12.5, color: "var(--text-2)" }}>
                      {d.last_activity_at ? new Date(d.last_activity_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }) : "—"}
                    </span>
                    <span style={{ textAlign: "right" }}>
                      <RunControls deployment={d} onAction={(a) => onAction(a)(d)} busy={update.isPending} />
                    </span>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>
    </>
  );
}

interface RouterProps {
  deployingPersonaId: string | null;
  personas: Persona[];
  onClose: () => void;
}

function DeployModalRouter({ deployingPersonaId, personas, onClose }: RouterProps) {
  const persona = deployingPersonaId
    ? personas.find((p) => p.id === deployingPersonaId) ?? null
    : null;
  const { projects } = useAllAgentDeployments();
  return (
    <DeployModal
      open={!!persona}
      onClose={onClose}
      persona={persona}
      projects={projects}
    />
  );
}

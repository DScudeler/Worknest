import { useMemo, useState } from "react";
import { useNavigate, useOutletContext, useParams, useSearchParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { ArrowLeft, List as ListIcon, Plus, Search, SquareStack } from "lucide-react";
import {
  projectsApi,
  ticketsApi,
  usersApi,
} from "../lib/api";
import { Topbar } from "../components/Topbar";
import { TicketRow } from "../components/TicketRow";
import { KanbanBoard } from "../components/KanbanBoard";
import { CreateTicketModal } from "../components/CreateTicketModal";
import { TicketSheet } from "../components/TicketSheet";
import { CenterSpinner } from "../components/Spinner";
import { FilterChip, ToggleChip } from "../components/FilterChip";
import { Avatar, AvatarStack } from "../components/Avatar";
import { projectCover, projectIcon } from "../lib/colors";
import type { Priority, Ticket, TicketStatus } from "../lib/types";
import { priorityLabel, statusLabel } from "../lib/types";
import { useAuth } from "../state/auth";

const STATUS_OPTIONS: { value: TicketStatus; label: string }[] = [
  { value: "Open", label: statusLabel("Open") },
  { value: "InProgress", label: statusLabel("InProgress") },
  { value: "Review", label: statusLabel("Review") },
  { value: "Done", label: statusLabel("Done") },
  { value: "Closed", label: statusLabel("Closed") },
];

const PRIORITY_OPTIONS: { value: Priority; label: string }[] = [
  { value: "Low", label: priorityLabel("Low") },
  { value: "Medium", label: priorityLabel("Medium") },
  { value: "High", label: priorityLabel("High") },
  { value: "Critical", label: priorityLabel("Critical") },
];

export function ProjectScreen() {
  const { projectId } = useParams<{ projectId: string }>();
  const [searchParams, setSearchParams] = useSearchParams();
  const navigate = useNavigate();
  const { user } = useAuth();
  const ctx = useOutletContext<{ openCreateProject: () => void }>();
  void ctx; // currently unused here, kept for parity

  const view = (searchParams.get("view") === "board" ? "board" : "list") as "list" | "board";
  const setView = (v: "list" | "board") => {
    const next = new URLSearchParams(searchParams);
    if (v === "board") next.set("view", "board");
    else next.delete("view");
    setSearchParams(next, { replace: true });
  };

  const [search, setSearch] = useState("");
  const [statusFilter, setStatusFilter] = useState<TicketStatus | null>(null);
  const [priorityFilter, setPriorityFilter] = useState<Priority | null>(null);
  const [assigneeFilter, setAssigneeFilter] = useState<string | null>(null);
  const [mineOnly, setMineOnly] = useState(false);
  const [createOpen, setCreateOpen] = useState(false);
  const [openTicketId, setOpenTicketId] = useState<string | null>(null);

  const projectQ = useQuery({
    queryKey: ["project", projectId],
    queryFn: () => projectsApi.get(projectId!),
    enabled: !!projectId,
  });
  const ticketsQ = useQuery({
    queryKey: ["tickets", { project_id: projectId }],
    queryFn: () => ticketsApi.list({ project_id: projectId }),
    enabled: !!projectId,
  });
  const membersQ = useQuery({
    queryKey: ["project", projectId, "members"],
    queryFn: () => projectsApi.members(projectId!),
    enabled: !!projectId,
  });
  const usersQ = useQuery({
    queryKey: ["users"],
    queryFn: () => usersApi.list(),
  });

  const userMap = useMemo(
    () => new Map((usersQ.data ?? []).map((u) => [u.id, u])),
    [usersQ.data],
  );
  const memberUsers = useMemo(
    () =>
      (membersQ.data ?? [])
        .map((m) => userMap.get(m.user_id))
        .filter((u): u is NonNullable<typeof u> => Boolean(u)),
    [membersQ.data, userMap],
  );

  const filtered = useMemo(() => {
    const all = ticketsQ.data ?? [];
    const q = search.trim().toLowerCase();
    return all.filter((t) => {
      if (statusFilter && t.status !== statusFilter) return false;
      if (priorityFilter && t.priority !== priorityFilter) return false;
      if (assigneeFilter && t.assignee_id !== assigneeFilter) return false;
      if (mineOnly && t.assignee_id !== user?.id) return false;
      if (q) {
        const hay = `${t.title} ${t.id}`.toLowerCase();
        if (!hay.includes(q)) return false;
      }
      return true;
    });
  }, [ticketsQ.data, search, statusFilter, priorityFilter, assigneeFilter, mineOnly, user]);

  const total = ticketsQ.data?.length ?? 0;

  const shortId = (id: string) => id.slice(0, 8);
  const project = projectQ.data;

  if (!projectId) return <div className="content">Project not found.</div>;
  if (projectQ.isLoading) return <CenterSpinner />;
  if (projectQ.isError || !project) {
    return (
      <div className="content">
        <p className="muted">Project not found or you do not have access.</p>
        <button className="btn secondary" onClick={() => navigate("/")}>
          ← Back to dashboard
        </button>
      </div>
    );
  }

  return (
    <>
      <Topbar
        crumbs={[
          { label: "Workspace" },
          { label: "Projects", href: "/" },
          { label: project.name },
        ]}
      />
      <div className="content">
        <button
          className="btn ghost"
          onClick={() => navigate("/")}
          style={{ marginBottom: 16 }}
        >
          <ArrowLeft size={14} /> Projects
        </button>

        <div className="page-head">
          <div style={{ display: "flex", gap: 14, alignItems: "center" }}>
            <div
              style={{
                width: 48,
                height: 48,
                borderRadius: 12,
                background: projectCover(project.id, project.color),
                display: "grid",
                placeItems: "center",
                fontSize: 22,
              }}
            >
              {projectIcon(project.id)}
            </div>
            <div>
              <h1>{project.name}</h1>
              <p className="sub">
                {project.description ?? "No description yet."}
              </p>
            </div>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
            {memberUsers.length > 0 && (
              <AvatarStack
                users={memberUsers.map((u) => ({ id: u.id, name: u.username }))}
                max={4}
              />
            )}
            <button className="btn secondary">Invite</button>
            <button className="btn primary" onClick={() => setCreateOpen(true)}>
              <Plus size={14} /> New ticket
            </button>
          </div>
        </div>

        <div className="filter-bar">
          <div className="input-icon search">
            <Search size={14} />
            <input
              className="input"
              placeholder="Search tickets…"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
          <FilterChip
            label="Status"
            value={statusFilter}
            options={STATUS_OPTIONS}
            onChange={setStatusFilter}
          />
          <FilterChip
            label="Priority"
            value={priorityFilter}
            options={PRIORITY_OPTIONS}
            onChange={setPriorityFilter}
          />
          <FilterChip
            label="Assignee"
            value={assigneeFilter}
            options={memberUsers.map((u) => ({ value: u.id, label: u.username }))}
            onChange={setAssigneeFilter}
          />
          <ToggleChip
            label="My tickets"
            active={mineOnly}
            onChange={setMineOnly}
            variant="tint"
          />
          {(statusFilter || priorityFilter || assigneeFilter || mineOnly) && (
            <button
              className="btn ghost"
              onClick={() => {
                setStatusFilter(null);
                setPriorityFilter(null);
                setAssigneeFilter(null);
                setMineOnly(false);
              }}
            >
              Clear
            </button>
          )}
          <span className="spacer" />
          <span className="muted" style={{ fontSize: 12.5 }}>
            {filtered.length} of {total}
          </span>
          <div className="view-toggle">
            <button
              className={view === "list" ? "active" : ""}
              onClick={() => setView("list")}
            >
              <ListIcon size={13} /> List
            </button>
            <button
              className={view === "board" ? "active" : ""}
              onClick={() => setView("board")}
            >
              <SquareStack size={13} /> Board
            </button>
          </div>
        </div>

        {ticketsQ.isLoading ? (
          <CenterSpinner />
        ) : view === "list" ? (
          <TicketTable
            tickets={filtered}
            shortId={shortId}
            userMap={userMap}
            onClick={setOpenTicketId}
          />
        ) : (
          <KanbanBoard
            tickets={filtered}
            shortId={shortId}
            users={usersQ.data ?? []}
            onTicketClick={setOpenTicketId}
            onAddTicket={() => setCreateOpen(true)}
          />
        )}
      </div>

      {createOpen && (
        <CreateTicketModal
          open={createOpen}
          onClose={() => setCreateOpen(false)}
          projectId={projectId}
          projectName={project.name}
        />
      )}

      {openTicketId && (
        <TicketSheet
          ticketId={openTicketId}
          onClose={() => setOpenTicketId(null)}
        />
      )}
    </>
  );
}

function TicketTable({
  tickets,
  shortId,
  userMap,
  onClick,
}: {
  tickets: Ticket[];
  shortId: (id: string) => string;
  userMap: Map<string, { id: string; username: string }>;
  onClick: (id: string) => void;
}) {
  if (tickets.length === 0) {
    return (
      <div className="card center-page" style={{ padding: 40 }}>
        <p className="muted">No tickets match these filters.</p>
      </div>
    );
  }
  return (
    <div className="ticket-list">
      <div className="tl-head">
        <span>ID</span>
        <span>Title</span>
        <span>Status</span>
        <span>Priority</span>
        <span>Assignee</span>
        <span>Due</span>
        <span>Activity</span>
      </div>
      {tickets.map((t) => (
        <TicketRow
          key={t.id}
          ticket={t}
          shortId={shortId(t.id)}
          assigneeName={t.assignee_id ? userMap.get(t.assignee_id)?.username : undefined}
          onClick={() => onClick(t.id)}
        />
      ))}
    </div>
  );
}

// Re-export Avatar so we don't get an unused-import warning if user state changes.
void Avatar;

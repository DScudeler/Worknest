import { useMemo } from "react";
import { Plus } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { projectsApi, ticketsApi } from "../lib/api";
import { useAuth } from "../state/auth";
import { ProjectCard } from "../components/ProjectCard";

interface Props {
  onCreateProject: () => void;
}

function greeting(): string {
  const h = new Date().getHours();
  if (h < 5) return "You're up late,";
  if (h < 12) return "Good morning,";
  if (h < 18) return "Good afternoon,";
  return "Good evening,";
}

function startOfWeek(): Date {
  const d = new Date();
  const day = d.getDay();
  const diff = (day + 6) % 7; // Monday-start
  d.setHours(0, 0, 0, 0);
  d.setDate(d.getDate() - diff);
  return d;
}

export function DashboardScreen({ onCreateProject }: Props) {
  const { user } = useAuth();
  const navigate = useNavigate();
  const { data: projects = [] } = useQuery({
    queryKey: ["projects"],
    queryFn: () => projectsApi.list(),
  });
  // Pull tickets assigned to me to derive stats. Phase 7 swaps this for /api/stats.
  const { data: myTickets = [] } = useQuery({
    queryKey: ["tickets", { assignee_id: "me" }],
    queryFn: () => ticketsApi.list({ assignee_id: "me" }),
  });
  const { data: openTickets = [] } = useQuery({
    queryKey: ["tickets", { status: "open" }],
    queryFn: () => ticketsApi.list({ status: "open" }),
  });

  const activeProjects = projects.filter((p) => !p.archived);

  const stats = useMemo(() => {
    const weekStart = startOfWeek();
    const weekEnd = new Date(weekStart);
    weekEnd.setDate(weekEnd.getDate() + 7);
    const dueThisWeek = myTickets.filter((t) => {
      if (!t.due_date) return false;
      const d = new Date(t.due_date);
      return d >= weekStart && d < weekEnd;
    }).length;
    return {
      open: openTickets.length,
      mine: myTickets.length,
      dueThisWeek,
      activeProjects: activeProjects.length,
    };
  }, [myTickets, openTickets, activeProjects]);

  return (
    <div>
      <div className="page-head">
        <div>
          <h1>
            {greeting()} <span className="greet-accent">{user?.username ?? "there"}</span>
          </h1>
          <p className="sub">
            {new Date().toLocaleDateString(undefined, {
              weekday: "long",
              month: "long",
              day: "numeric",
            })}
            {" · "}
            {stats.mine} ticket{stats.mine === 1 ? "" : "s"} assigned to you.
          </p>
        </div>
        <button type="button" className="btn primary" onClick={onCreateProject}>
          <Plus size={14} /> New project
        </button>
      </div>

      <div className="stat-row">
        <StatCard label="Open tickets" value={stats.open} />
        <StatCard label="Assigned to me" value={stats.mine} />
        <StatCard label="Due this week" value={stats.dueThisWeek} />
        <StatCard label="Active projects" value={stats.activeProjects} />
      </div>

      <div className="section-row">
        <h2>Your projects</h2>
        <span className="meta">{activeProjects.length} project{activeProjects.length === 1 ? "" : "s"}</span>
      </div>
      <div className="proj-grid">
        {activeProjects.map((p) => (
          <ProjectCard
            key={p.id}
            project={p}
            onClick={() => navigate(`/projects/${p.id}`)}
          />
        ))}
        <button
          type="button"
          className="proj-card create"
          onClick={onCreateProject}
        >
          <span className="plus">+</span>
          New project
        </button>
      </div>
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: number }) {
  return (
    <div className="stat-card">
      <div className="label">{label}</div>
      <div className="value">{value}</div>
    </div>
  );
}

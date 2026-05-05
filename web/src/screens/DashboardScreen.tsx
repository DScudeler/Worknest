import { Plus } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { projectsApi, statsApi } from "../lib/api";
import { displayName } from "../lib/types";
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

export function DashboardScreen({ onCreateProject }: Props) {
  const { user } = useAuth();
  const navigate = useNavigate();
  const { data: projects = [] } = useQuery({
    queryKey: ["projects"],
    queryFn: () => projectsApi.list(),
  });
  const { data: stats } = useQuery({
    queryKey: ["stats"],
    queryFn: () => statsApi.get(),
  });

  const activeProjects = projects.filter((p) => !p.archived);

  return (
    <div>
      <div className="page-head">
        <div>
          <h1>
            {greeting()}{" "}
            <span className="greet-accent">
              {user ? displayName(user) : "there"}
            </span>
          </h1>
          <p className="sub">
            {new Date().toLocaleDateString(undefined, {
              weekday: "long",
              month: "long",
              day: "numeric",
            })}
            {stats
              ? ` · ${stats.assigned_to_me} ticket${stats.assigned_to_me === 1 ? "" : "s"} assigned to you.`
              : ""}
          </p>
        </div>
        <button type="button" className="btn primary" onClick={onCreateProject}>
          <Plus size={14} /> New project
        </button>
      </div>

      <div className="stat-row">
        <StatCard label="Open tickets" value={stats?.open_tickets ?? 0} />
        <StatCard label="Assigned to me" value={stats?.assigned_to_me ?? 0} />
        <StatCard label="Due this week" value={stats?.due_this_week ?? 0} />
        <StatCard label="Active projects" value={stats?.active_projects ?? 0} />
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

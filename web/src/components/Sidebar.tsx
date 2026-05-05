import { Home, Inbox, LogOut, Settings as SettingsIcon, Star, Plus } from "lucide-react";
import { NavLink, useNavigate, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { projectsApi } from "../lib/api";
import type { Project } from "../lib/types";
import { projectCover } from "../lib/colors";
import { useAuth } from "../state/auth";
import { ThemeToggle } from "./ThemeToggle";
import { Avatar } from "./Avatar";

interface Props {
  onCreateProject: () => void;
}

export function Sidebar({ onCreateProject }: Props) {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const { projectId } = useParams<{ projectId: string }>();
  const { data: projects } = useQuery({
    queryKey: ["projects"],
    queryFn: () => projectsApi.list(),
  });

  const top = (projects ?? [])
    .filter((p: Project) => !p.archived)
    .slice(0, 5);

  return (
    <aside className="sidebar">
      <div className="logo">
        <span className="logo-mark">W</span>
        <span>Worknest</span>
      </div>

      <NavLink
        to="/"
        end
        className={({ isActive }) => `nav-item${isActive ? " active" : ""}`}
      >
        <Home size={16} />
        <span>Dashboard</span>
      </NavLink>
      <NavLink to="/inbox" className={({ isActive }) => `nav-item${isActive ? " active" : ""}`}>
        <Inbox size={16} />
        <span>Inbox</span>
      </NavLink>
      <NavLink to="/my-tickets" className={({ isActive }) => `nav-item${isActive ? " active" : ""}`}>
        <Star size={16} />
        <span>My tickets</span>
      </NavLink>

      <div className="nav-section">Projects</div>
      {top.map((p) => (
        <button
          key={p.id}
          type="button"
          className={`nav-item${projectId === p.id ? " active" : ""}`}
          onClick={() => navigate(`/projects/${p.id}`)}
        >
          <span
            className="proj-dot"
            style={{ background: projectCover(p.id, p.color) }}
          />
          <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {p.name}
          </span>
        </button>
      ))}
      <button type="button" className="nav-item" onClick={onCreateProject}>
        <Plus size={16} />
        <span>New project</span>
      </button>

      <div className="sidebar-foot">
        <ThemeToggle />
        <button type="button" className="theme-toggle" title="Settings" onClick={() => navigate("/settings")}>
          <SettingsIcon size={16} />
        </button>
        <span className="spacer" />
        {user ? (
          <>
            <Avatar
              id={user.id}
              name={user.full_name?.trim() || user.username}
              avatarUrl={user.avatar_url}
              size="sm"
            />
            <button
              type="button"
              className="theme-toggle"
              title="Sign out"
              onClick={() => logout()}
            >
              <LogOut size={16} />
            </button>
          </>
        ) : null}
      </div>
    </aside>
  );
}

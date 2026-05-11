import { Outlet } from "react-router-dom";
import { useState } from "react";
import { Sidebar } from "./Sidebar";
import { CreateProjectModal } from "./CreateProjectModal";
import type { Project } from "../lib/types";

export interface AppOutletContext {
  openCreateProject: () => void;
  openEditProject: (project: Project) => void;
}

export function AppLayout() {
  const [createOpen, setCreateOpen] = useState(false);
  const [editProject, setEditProject] = useState<Project | null>(null);
  const ctx: AppOutletContext = {
    openCreateProject: () => {
      setEditProject(null);
      setCreateOpen(true);
    },
    openEditProject: (p) => {
      setEditProject(p);
      setCreateOpen(true);
    },
  };
  const handleClose = () => {
    setCreateOpen(false);
    // Defer clearing editProject so the modal's closing animation doesn't
    // reflow back to create-mode mid-fade.
    setTimeout(() => setEditProject(null), 200);
  };
  return (
    <div className="app">
      <Sidebar onCreateProject={ctx.openCreateProject} />
      <div className="main">
        <Outlet context={ctx} />
      </div>
      <CreateProjectModal
        open={createOpen}
        onClose={handleClose}
        project={editProject}
      />
    </div>
  );
}

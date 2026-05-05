import { Outlet } from "react-router-dom";
import { useState } from "react";
import { Sidebar } from "./Sidebar";
import { CreateProjectModal } from "./CreateProjectModal";

export function AppLayout() {
  const [createProjectOpen, setCreateProjectOpen] = useState(false);
  return (
    <div className="app">
      <Sidebar onCreateProject={() => setCreateProjectOpen(true)} />
      <div className="main">
        <Outlet context={{ openCreateProject: () => setCreateProjectOpen(true) }} />
      </div>
      <CreateProjectModal
        open={createProjectOpen}
        onClose={() => setCreateProjectOpen(false)}
      />
    </div>
  );
}

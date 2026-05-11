import { useOutletContext } from "react-router-dom";
import { Topbar } from "../components/Topbar";
import { DashboardScreen } from "./DashboardScreen";

export function DashboardWrapper() {
  const { openCreateProject } = useOutletContext<{ openCreateProject: () => void }>();
  return (
    <>
      <Topbar crumbs={[{ label: "Workspace" }, { label: "Dashboard" }]} />
      <div className="content">
        <DashboardScreen onCreateProject={openCreateProject} />
      </div>
    </>
  );
}

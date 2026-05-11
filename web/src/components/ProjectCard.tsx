import type { Project } from "../lib/types";
import { projectCover, projectIcon } from "../lib/colors";

interface Props {
  project: Project;
  onClick?: () => void;
  openCount?: number;
  totalCount?: number;
  progress?: number;
}

export function ProjectCard({ project, onClick, openCount, totalCount, progress }: Props) {
  const cover = projectCover(project.id, project.color);
  return (
    <div
      className="proj-card"
      onClick={onClick}
      role={onClick ? "button" : undefined}
      tabIndex={onClick ? 0 : undefined}
      onKeyDown={(e) => {
        if (onClick && (e.key === "Enter" || e.key === " ")) {
          e.preventDefault();
          onClick();
        }
      }}
    >
      <div className="pc-banner" style={{ background: cover }}>
        <span className="pc-icon">{projectIcon(project.id)}</span>
      </div>
      <div className="pc-body">
        <div className="pc-name">{project.name}</div>
        <div className="pc-desc">
          {project.description ?? "No description yet."}
        </div>
        <div className="pc-foot">
          <div className="pc-stats">
            {openCount !== undefined && totalCount !== undefined ? (
              <span>
                {openCount} open / {totalCount}
              </span>
            ) : (
              <span>—</span>
            )}
          </div>
        </div>
        {progress !== undefined ? (
          <div className="progress">
            <div style={{ width: `${Math.round(progress * 100)}%` }} />
          </div>
        ) : null}
      </div>
    </div>
  );
}

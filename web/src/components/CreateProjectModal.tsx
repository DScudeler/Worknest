import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import toast from "react-hot-toast";
import { Modal } from "./Modal";
import { ProjectCard } from "./ProjectCard";
import { ApiError, projectsApi, usersApi } from "../lib/api";
import type { Project } from "../lib/types";
import { useAuth } from "../state/auth";
import { Avatar } from "./Avatar";

const COVERS = ["#fde68a", "#c4b5fd", "#a7f3d0", "#fbcfe8", "#bae6fd", "#fed7aa"];
const ICONS = [
  "🌐", "📱", "⚙️", "🎨", "📈", "💬", "🚀", "🔬",
  "🧪", "📦", "🛠️", "🧭", "📊", "🗂️", "🎯", "💡",
];

interface Props {
  open: boolean;
  onClose: () => void;
  /// When set, the modal opens in edit mode pre-populated from this project.
  /// Title becomes `Edit <name>`, the primary CTA on step 2 becomes
  /// `Save changes`, and a destructive `Archive project` button appears on
  /// step 1.
  project?: Project | null;
}

export function CreateProjectModal({ open, onClose, project }: Props) {
  const { user } = useAuth();
  const qc = useQueryClient();
  const isEdit = !!project;
  const [step, setStep] = useState<1 | 2>(1);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [repoPath, setRepoPath] = useState("");
  const [color, setColor] = useState<string>(COVERS[0]);
  // icon is currently a UI-only choice — backend doesn't store it yet.
  const [icon, setIcon] = useState<string>(ICONS[0]);
  const [invitees, setInvitees] = useState<Set<string>>(new Set());
  const [error, setError] = useState<string | null>(null);

  const { data: people = [] } = useQuery({
    queryKey: ["users"],
    queryFn: () => usersApi.list(),
    enabled: open,
  });
  const { data: existingMembers } = useQuery({
    queryKey: ["project", project?.id, "members"],
    queryFn: () => projectsApi.members(project!.id),
    enabled: open && isEdit,
  });

  // Reset / re-seed when the modal opens or the editing target changes.
  // Keyed on project?.id (not the project object) so unrelated parent re-renders
  // don't blow away in-progress edits.
  useEffect(() => {
    if (!open) {
      setStep(1);
      setError(null);
      return;
    }
    if (isEdit && project) {
      setName(project.name);
      setDescription(project.description ?? "");
      setRepoPath(project.repo_path ?? "");
      setColor(project.color ?? COVERS[0]);
      setIcon(ICONS[0]);
    } else {
      setName("");
      setDescription("");
      setRepoPath("");
      setColor(COVERS[0]);
      setIcon(ICONS[0]);
      setInvitees(new Set());
    }
    setError(null);
    // Intentionally key on `project?.id` only, not the full `project` object.
    // The parent's `project` reference can change on every render even when
    // the user is mid-edit; including it would clobber unsaved input.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, isEdit, project?.id]);

  // Sync invitees once member data actually arrives in edit mode.
  // Kept separate so the invitees set isn't re-derived from a fresh `[]`
  // default on every render — that was clobbering form input.
  useEffect(() => {
    if (!open || !isEdit || !existingMembers) return;
    setInvitees(new Set(existingMembers.map((m) => m.user_id)));
  }, [open, isEdit, existingMembers]);

  const createMut = useMutation({
    mutationFn: async () => {
      const created = await projectsApi.create({
        name,
        description: description || undefined,
        color,
        repo_path: repoPath.trim() || undefined,
      });
      // V4 migration: owner is auto-added; invite the others.
      await Promise.all(
        Array.from(invitees)
          .filter((id) => id !== user?.id)
          .map((uid) =>
            projectsApi.addMember(created.id, { user_id: uid, role: "member" }).catch(() => {
              /* swallow individual invite failures so the project still creates */
            }),
          ),
      );
      return created;
    },
    onSuccess: (created: Project) => {
      qc.invalidateQueries({ queryKey: ["projects"] });
      toast.success(`Project '${created.name}' created`);
      onClose();
    },
    onError: (err: unknown) => {
      setError(err instanceof ApiError ? err.message : "Could not create project");
    },
  });

  const updateMut = useMutation({
    mutationFn: async () => {
      if (!project) throw new Error("No project to update");
      const updated = await projectsApi.update(project.id, {
        name,
        description: description || null,
        color: color || null,
        repo_path: repoPath.trim() || null,
      });
      // Sync members: add newly-checked, remove newly-unchecked. Owner stays
      // (the API does not let the owner be removed via this endpoint anyway).
      const before = new Set((existingMembers ?? []).map((m) => m.user_id));
      const after = invitees;
      await Promise.all([
        ...Array.from(after)
          .filter((id) => id !== user?.id && !before.has(id))
          .map((uid) =>
            projectsApi
              .addMember(project.id, { user_id: uid, role: "member" })
              .catch(() => {}),
          ),
        ...Array.from(before)
          .filter((id) => id !== project.created_by && !after.has(id))
          .map((uid) => projectsApi.removeMember(project.id, uid).catch(() => {})),
      ]);
      return updated;
    },
    onSuccess: (updated: Project) => {
      qc.invalidateQueries({ queryKey: ["projects"] });
      qc.invalidateQueries({ queryKey: ["project", updated.id] });
      qc.invalidateQueries({ queryKey: ["project", updated.id, "members"] });
      toast.success(`Project '${updated.name}' updated`);
      onClose();
    },
    onError: (err: unknown) => {
      setError(err instanceof ApiError ? err.message : "Could not update project");
    },
  });

  const archiveMut = useMutation({
    mutationFn: () => projectsApi.archive(project!.id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["projects"] });
      qc.invalidateQueries({ queryKey: ["project", project!.id] });
      toast.success(`Project '${project!.name}' archived`);
      onClose();
    },
    onError: (err: unknown) => {
      setError(err instanceof ApiError ? err.message : "Could not archive project");
    },
  });

  const previewProject: Project = useMemo(
    () => ({
      id: project?.id ?? "preview",
      name: name || "New project",
      description: description || "Describe what this project is for.",
      color,
      archived: project?.archived ?? false,
      created_by: project?.created_by ?? user?.id ?? "preview",
      repo_path: repoPath.trim() || null,
      created_at: project?.created_at ?? new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }),
    [name, description, repoPath, color, user, project],
  );

  const peopleMinusMe = people.filter((p) => p.id !== user?.id);
  const submitPending = createMut.isPending || updateMut.isPending || archiveMut.isPending;

  const handlePrimary = () => {
    if (isEdit) updateMut.mutate();
    else createMut.mutate();
  };
  const handleArchive = () => {
    if (!project) return;
    if (window.confirm(`Archive project '${project.name}'? This hides it from the dashboard. You can unarchive later.`)) {
      archiveMut.mutate();
    }
  };

  const titleText = isEdit ? `Edit ${project?.name ?? "project"}` : "New project";
  const subtitleText =
    step === 1
      ? `Step 1 of 2 · Details`
      : `Step 2 of 2 · Members`;

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={titleText}
      subtitle={subtitleText}
      foot={
        step === 1 ? (
          <>
            {isEdit ? (
              <button
                className="btn ghost danger"
                onClick={handleArchive}
                disabled={submitPending}
                style={{ marginRight: "auto" }}
              >
                Archive project
              </button>
            ) : null}
            <button className="btn ghost" onClick={onClose}>
              Cancel
            </button>
            <button
              className="btn primary"
              onClick={() => setStep(2)}
              disabled={!name.trim()}
            >
              Continue →
            </button>
          </>
        ) : (
          <>
            <button className="btn secondary" onClick={() => setStep(1)}>
              Back
            </button>
            <button
              className="btn primary"
              onClick={handlePrimary}
              disabled={submitPending}
            >
              {submitPending
                ? isEdit
                  ? "Saving…"
                  : "Creating…"
                : isEdit
                  ? "✓ Save changes"
                  : "✓ Create project"}
            </button>
          </>
        )
      }
    >
      {error ? (
        <div className="err" style={{ marginBottom: 12, color: "var(--pri-urgent)" }}>
          {error}
        </div>
      ) : null}

      {step === 1 ? (
        <div className="flex-col" style={{ gap: 16 }}>
          <div>
            <label className="field-label" htmlFor="cp-name">Project name</label>
            <input
              id="cp-name"
              className="input"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. Marketing Website"
              autoFocus
            />
          </div>
          <div>
            <label className="field-label" htmlFor="cp-desc">Description</label>
            <textarea
              id="cp-desc"
              className="textarea"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="What's this project about?"
            />
          </div>
          <div>
            <label className="field-label" htmlFor="cp-repo">
              Source repo (optional)
            </label>
            <input
              id="cp-repo"
              className="input"
              value={repoPath}
              onChange={(e) => setRepoPath(e.target.value)}
              placeholder="/abs/path/to/repo  or  https://github.com/you/repo.git"
              style={{ fontFamily: "ui-monospace, Menlo, monospace" }}
            />
            <div className="muted" style={{ fontSize: 12, marginTop: 4 }}>
              When set, agent deployments to this project bootstrap a per-agent
              git worktree on branch <code>swarm/&lt;persona&gt;</code>. Leave
              blank for projects where agents only comment / triage.
            </div>
          </div>
          <div>
            <label className="field-label">Cover color</label>
            <div className="color-grid">
              {COVERS.map((c) => (
                <button
                  key={c}
                  type="button"
                  className={`color-swatch${color === c ? " active" : ""}`}
                  style={{ background: c }}
                  onClick={() => setColor(c)}
                  aria-label={`Cover color ${c}`}
                />
              ))}
            </div>
          </div>
          <div>
            <label className="field-label">Icon</label>
            <div className="icon-grid">
              {ICONS.map((e) => (
                <button
                  key={e}
                  type="button"
                  className={`icon-cell${icon === e ? " active" : ""}`}
                  onClick={() => setIcon(e)}
                >
                  {e}
                </button>
              ))}
            </div>
          </div>
          <div>
            <label className="field-label">Preview</label>
            <ProjectCard project={previewProject} />
          </div>
        </div>
      ) : (
        <div className="flex-col" style={{ gap: 6 }}>
          <p className="muted" style={{ marginTop: 0 }}>
            {isEdit
              ? "Manage who can collaborate on this project."
              : "Invite teammates to collaborate. You can change this later."}
          </p>
          {peopleMinusMe.length === 0 ? (
            <p className="muted">No other users in your workspace yet.</p>
          ) : (
            peopleMinusMe.map((p) => {
              const selected = invitees.has(p.id);
              return (
                <label
                  key={p.id}
                  className={`checkbox-row${selected ? " selected" : ""}`}
                >
                  <input
                    type="checkbox"
                    checked={selected}
                    onChange={(e) => {
                      const next = new Set(invitees);
                      if (e.target.checked) next.add(p.id);
                      else next.delete(p.id);
                      setInvitees(next);
                    }}
                  />
                  <Avatar id={p.id} name={p.username} size="sm" />
                  <span>{p.username}</span>
                </label>
              );
            })
          )}
        </div>
      )}
    </Modal>
  );
}

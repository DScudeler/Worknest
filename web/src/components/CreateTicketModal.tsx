import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import toast from "react-hot-toast";
import { Modal } from "./Modal";
import { ApiError, projectsApi, tagsApi, ticketsApi, usersApi } from "../lib/api";
import type { Priority, ProjectId, TagId, TicketType } from "../lib/types";
import { TagChip } from "./Tag";

interface Props {
  open: boolean;
  onClose: () => void;
  projectId: ProjectId;
  projectName: string;
}

const PRIORITIES: { value: Priority; label: string }[] = [
  { value: "Low", label: "Low" },
  { value: "Medium", label: "Medium" },
  { value: "High", label: "High" },
  { value: "Critical", label: "Urgent" },
];

const TYPES: { value: TicketType; label: string }[] = [
  { value: "Task", label: "Task" },
  { value: "Bug", label: "Bug" },
  { value: "Feature", label: "Feature" },
  { value: "Epic", label: "Epic" },
];

export function CreateTicketModal({ open, onClose, projectId, projectName }: Props) {
  const qc = useQueryClient();
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [type, setType] = useState<TicketType>("Task");
  const [priority, setPriority] = useState<Priority>("Medium");
  const [assigneeId, setAssigneeId] = useState<string>("");
  const [selectedTags, setSelectedTags] = useState<Set<TagId>>(new Set());
  const [error, setError] = useState<string | null>(null);

  const { data: members = [] } = useQuery({
    queryKey: ["projects", projectId, "members"],
    queryFn: () => projectsApi.members(projectId),
    enabled: open,
  });
  const { data: people = [] } = useQuery({
    queryKey: ["users"],
    queryFn: () => usersApi.list(),
    enabled: open,
  });
  const { data: tags = [] } = useQuery({
    queryKey: ["tags"],
    queryFn: () => tagsApi.list(),
    enabled: open,
  });

  const visibleAssignees = members
    .map((m) => people.find((p) => p.id === m.user_id))
    .filter((p): p is NonNullable<typeof p> => Boolean(p));

  useEffect(() => {
    if (!open) {
      setTitle("");
      setDescription("");
      setType("Task");
      setPriority("Medium");
      setAssigneeId("");
      setSelectedTags(new Set());
      setError(null);
    }
  }, [open]);

  const createMut = useMutation({
    mutationFn: async () => {
      const ticket = await ticketsApi.create({
        project_id: projectId,
        title,
        description: description || undefined,
        ticket_type: type,
        priority,
        tag_ids: Array.from(selectedTags),
      });
      if (assigneeId) {
        await ticketsApi.update(ticket.id, { assignee_id: assigneeId });
      }
      return ticket;
    },
    onSuccess: (t) => {
      qc.invalidateQueries({ queryKey: ["tickets"] });
      toast.success(`Ticket '${t.title}' created`);
      onClose();
    },
    onError: (err: unknown) => {
      setError(err instanceof ApiError ? err.message : "Could not create ticket");
    },
  });

  return (
    <Modal
      open={open}
      onClose={onClose}
      size="large"
      title="New ticket"
      subtitle={`in ${projectName}`}
      foot={
        <>
          <button className="btn ghost" onClick={onClose}>
            Cancel
          </button>
          <button
            className="btn primary"
            onClick={() => createMut.mutate()}
            disabled={!title.trim() || createMut.isPending}
          >
            {createMut.isPending ? "Creating…" : "✓ Create ticket"}
          </button>
        </>
      }
    >
      {error ? <div className="err" style={{ marginBottom: 12 }}>{error}</div> : null}
      <div className="flex-col" style={{ gap: 14 }}>
        <input
          className="input"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="Ticket title"
          style={{ fontSize: 16, padding: "12px 14px" }}
          autoFocus
        />
        <textarea
          className="textarea"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Describe the work, acceptance criteria, links…"
        />
        <div className="row-3">
          <div>
            <label className="field-label">Type</label>
            <select className="select" value={type} onChange={(e) => setType(e.target.value as TicketType)}>
              {TYPES.map((t) => (
                <option key={t.value} value={t.value}>{t.label}</option>
              ))}
            </select>
          </div>
          <div>
            <label className="field-label">Priority</label>
            <select className="select" value={priority} onChange={(e) => setPriority(e.target.value as Priority)}>
              {PRIORITIES.map((p) => (
                <option key={p.value} value={p.value}>{p.label}</option>
              ))}
            </select>
          </div>
          <div>
            <label className="field-label">Assignee</label>
            <select
              className="select"
              value={assigneeId}
              onChange={(e) => setAssigneeId(e.target.value)}
            >
              <option value="">Unassigned</option>
              {visibleAssignees.map((u) => (
                <option key={u.id} value={u.id}>{u.username}</option>
              ))}
            </select>
          </div>
        </div>
        <div>
          <label className="field-label">Tags</label>
          <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
            {tags.map((t) => {
              const on = selectedTags.has(t.id);
              return (
                <button
                  key={t.id}
                  type="button"
                  onClick={() => {
                    const next = new Set(selectedTags);
                    if (on) next.delete(t.id);
                    else next.add(t.id);
                    setSelectedTags(next);
                  }}
                  style={{
                    padding: 0,
                    border: "none",
                    background: "transparent",
                    cursor: "pointer",
                    opacity: on ? 1 : 0.45,
                    boxShadow: on ? `inset 0 0 0 2px ${t.color_fg}` : "none",
                    borderRadius: 6,
                  }}
                  aria-pressed={on}
                >
                  <TagChip tag={t} />
                </button>
              );
            })}
            {tags.length === 0 ? (
              <span className="muted">No tags configured.</span>
            ) : null}
          </div>
        </div>
      </div>
    </Modal>
  );
}

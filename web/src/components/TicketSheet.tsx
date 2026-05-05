import { useEffect, useMemo, useState } from "react";
import { Paperclip, Send, X } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import toast from "react-hot-toast";
import {
  attachmentsApi,
  commentsApi,
  projectsApi,
  ticketsApi,
  usersApi,
} from "../lib/api";
import type { Priority, TicketStatus, TicketType } from "../lib/types";
import { priorityLabel, statusLabel } from "../lib/types";
import { Avatar } from "./Avatar";
import { StatusPill } from "./StatusPill";
import { PriorityBadge } from "./PriorityBadge";
import { useAuth } from "../state/auth";

interface Props {
  ticketId: string;
  onClose: () => void;
}

const STATUS_OPTIONS: TicketStatus[] = ["Open", "InProgress", "Review", "Done", "Closed"];
const PRIORITY_OPTIONS: Priority[] = ["Low", "Medium", "High", "Critical"];
const TYPE_OPTIONS: TicketType[] = ["Task", "Bug", "Feature", "Epic"];

export function TicketSheet({ ticketId, onClose }: Props) {
  const qc = useQueryClient();
  const { user } = useAuth();
  const ticketQ = useQuery({
    queryKey: ["ticket", ticketId],
    queryFn: () => ticketsApi.get(ticketId),
  });
  const commentsQ = useQuery({
    queryKey: ["ticket", ticketId, "comments"],
    queryFn: () => commentsApi.listForTicket(ticketId),
  });
  const attachmentsQ = useQuery({
    queryKey: ["ticket", ticketId, "attachments"],
    queryFn: () => attachmentsApi.listForTicket(ticketId),
  });
  const usersQ = useQuery({ queryKey: ["users"], queryFn: () => usersApi.list() });

  const ticket = ticketQ.data;
  const projectId = ticket?.project_id;
  const projectQ = useQuery({
    queryKey: ["project", projectId],
    queryFn: () => projectsApi.get(projectId!),
    enabled: !!projectId,
  });
  const membersQ = useQuery({
    queryKey: ["project", projectId, "members"],
    queryFn: () => projectsApi.members(projectId!),
    enabled: !!projectId,
  });

  const userMap = useMemo(
    () => new Map((usersQ.data ?? []).map((u) => [u.id, u])),
    [usersQ.data],
  );

  const updateMut = useMutation({
    mutationFn: async (patch: { status?: TicketStatus; priority?: Priority; type?: TicketType; assignee_id?: string | "" }) => {
      if (!ticket) throw new Error("No ticket");
      return ticketsApi.update(
        ticket.id,
        {
          ...(patch.status !== undefined && { status: patch.status }),
          ...(patch.priority !== undefined && { priority: patch.priority }),
          ...(patch.type !== undefined && { ticket_type: patch.type }),
          ...(patch.assignee_id !== undefined && { assignee_id: patch.assignee_id }),
        },
        ticket.updated_at,
      );
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["ticket", ticketId] });
      qc.invalidateQueries({ queryKey: ["tickets"] });
    },
    onError: (err: Error) => toast.error(err.message || "Could not update ticket"),
  });

  const [draft, setDraft] = useState("");
  const commentMut = useMutation({
    mutationFn: () => commentsApi.create(ticketId, draft.trim()),
    onSuccess: () => {
      setDraft("");
      qc.invalidateQueries({ queryKey: ["ticket", ticketId, "comments"] });
    },
    onError: (err: Error) => toast.error(err.message || "Could not post comment"),
  });

  const uploadMut = useMutation({
    mutationFn: (file: File) => attachmentsApi.upload(ticketId, file),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["ticket", ticketId, "attachments"] });
      toast.success("Attachment uploaded");
    },
    onError: (err: Error) => toast.error(err.message || "Upload failed"),
  });

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  if (!ticket || !projectQ.data) {
    return (
      <>
        <div className="scrim" onClick={onClose} />
        <div className="sheet">
          <div className="sheet-head">
            <button className="theme-toggle" onClick={onClose} aria-label="Close">
              <X size={16} />
            </button>
          </div>
          <div className="center-page" style={{ flex: 1 }}>
            <span className="spinner" />
          </div>
        </div>
      </>
    );
  }

  const assigneeName = ticket.assignee_id
    ? userMap.get(ticket.assignee_id)?.username ?? "Unknown"
    : "Unassigned";
  const reporterName = userMap.get(ticket.created_by)?.username ?? "Unknown";

  return (
    <>
      <div className="scrim" onClick={onClose} />
      <aside className="sheet" role="dialog" aria-label={ticket.title}>
        <div className="sheet-head">
          <span className="id-pill">{ticket.id.slice(0, 8)}</span>
          <StatusPill status={ticket.status} />
          <span className="muted" style={{ fontSize: 12 }}>
            updated {timeAgo(ticket.updated_at)}
          </span>
          <div className="right">
            <button className="theme-toggle" onClick={onClose} aria-label="Close">
              <X size={16} />
            </button>
          </div>
        </div>
        <div className="sheet-body">
          <div className="sheet-main">
            <h1>{ticket.title}</h1>
            <div className="desc">
              {ticket.description?.trim() || (
                <span className="muted">No description yet.</span>
              )}
            </div>

            {(attachmentsQ.data ?? []).length > 0 ? (
              <>
                <h3 className="section-h">Attachments</h3>
                <div className="attach-row">
                  {(attachmentsQ.data ?? []).map((a) => (
                    <a
                      key={a.id}
                      className="attach"
                      href={attachmentsApi.downloadUrl(a.id)}
                      target="_blank"
                      rel="noopener noreferrer"
                      download={a.filename}
                    >
                      <span className="ico">
                        <Paperclip size={14} />
                      </span>
                      <div>
                        <div className="name">{a.filename}</div>
                        <div className="size">{formatSize(a.file_size)}</div>
                      </div>
                    </a>
                  ))}
                </div>
              </>
            ) : null}

            <h3 className="section-h">
              Activity · {commentsQ.data?.length ?? 0} comment
              {(commentsQ.data?.length ?? 0) === 1 ? "" : "s"}
            </h3>
            {(commentsQ.data ?? []).map((c) => {
              const author = userMap.get(c.user_id);
              return (
                <div key={c.id} className="comment">
                  <Avatar
                    id={c.user_id}
                    name={author?.username ?? "User"}
                    size="sm"
                  />
                  <div className="body">
                    <div className="meta">
                      <strong>{author?.username ?? "Unknown"}</strong>
                      <span>{timeAgo(c.created_at)}</span>
                    </div>
                    <div>{c.content}</div>
                  </div>
                </div>
              );
            })}

            <div className="composer" style={{ marginTop: 16 }}>
              {user ? <Avatar id={user.id} name={user.username} size="sm" /> : null}
              <div className="composer-input">
                <textarea
                  value={draft}
                  onChange={(e) => setDraft(e.target.value)}
                  placeholder="Leave a comment…"
                />
                <div className="composer-actions">
                  <label className="theme-toggle" title="Attach a file" style={{ cursor: "pointer" }}>
                    <Paperclip size={14} />
                    <input
                      type="file"
                      style={{ display: "none" }}
                      onChange={(e) => {
                        const f = e.target.files?.[0];
                        if (f) uploadMut.mutate(f);
                        e.currentTarget.value = "";
                      }}
                    />
                  </label>
                  <button
                    type="button"
                    className="btn primary"
                    disabled={!draft.trim() || commentMut.isPending}
                    onClick={() => commentMut.mutate()}
                  >
                    <Send size={12} />
                    Comment
                  </button>
                </div>
              </div>
            </div>
          </div>
          <div className="sheet-side">
            <Prop label="Status">
              <select
                className="select"
                value={ticket.status}
                onChange={(e) => updateMut.mutate({ status: e.target.value as TicketStatus })}
              >
                {STATUS_OPTIONS.map((s) => (
                  <option key={s} value={s}>{statusLabel(s)}</option>
                ))}
              </select>
            </Prop>
            <Prop label="Priority">
              <select
                className="select"
                value={ticket.priority}
                onChange={(e) => updateMut.mutate({ priority: e.target.value as Priority })}
              >
                {PRIORITY_OPTIONS.map((p) => (
                  <option key={p} value={p}>{priorityLabel(p)}</option>
                ))}
              </select>
              <PriorityBadge priority={ticket.priority} />
            </Prop>
            <Prop label="Type">
              <select
                className="select"
                value={ticket.ticket_type}
                onChange={(e) => updateMut.mutate({ type: e.target.value as TicketType })}
              >
                {TYPE_OPTIONS.map((t) => (
                  <option key={t} value={t}>{t.charAt(0).toUpperCase() + t.slice(1)}</option>
                ))}
              </select>
            </Prop>
            <Prop label="Assignee">
              <select
                className="select"
                value={ticket.assignee_id ?? ""}
                onChange={(e) =>
                  updateMut.mutate({
                    assignee_id: e.target.value === "" ? "" : e.target.value,
                  })
                }
              >
                <option value="">Unassigned</option>
                {(membersQ.data ?? [])
                  .map((m) => userMap.get(m.user_id))
                  .filter((u): u is NonNullable<typeof u> => Boolean(u))
                  .map((u) => (
                    <option key={u.id} value={u.id}>{u.username}</option>
                  ))}
              </select>
              <span style={{ fontSize: 12.5 }}>{assigneeName}</span>
            </Prop>
            <Prop label="Reporter">
              <span style={{ fontSize: 12.5 }}>{reporterName}</span>
            </Prop>
            <Prop label="Due">
              <span style={{ fontSize: 12.5 }}>
                {ticket.due_date
                  ? new Date(ticket.due_date).toLocaleDateString()
                  : "—"}
              </span>
            </Prop>
            <Prop label="Project">
              <span style={{ fontSize: 12.5 }}>{projectQ.data.name}</span>
            </Prop>
          </div>
        </div>
      </aside>
    </>
  );
}

function Prop({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="prop" style={{ alignItems: "start" }}>
      <span className="key" style={{ paddingTop: 6 }}>{label}</span>
      <span className="val" style={{ flexDirection: "column", alignItems: "stretch", gap: 4 }}>
        {children}
      </span>
    </div>
  );
}

function timeAgo(iso: string): string {
  const ms = Date.now() - new Date(iso).getTime();
  const min = Math.floor(ms / 60_000);
  if (min < 1) return "just now";
  if (min < 60) return `${min}m ago`;
  const h = Math.floor(min / 60);
  if (h < 24) return `${h}h ago`;
  const d = Math.floor(h / 24);
  if (d < 7) return `${d}d ago`;
  return new Date(iso).toLocaleDateString();
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

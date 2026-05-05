import { useMemo } from "react";
import { Plus } from "lucide-react";
import type { PublicUser, Ticket, TicketStatus } from "../lib/types";
import { PriorityBadge } from "./PriorityBadge";
import { Avatar } from "./Avatar";
import { TagList } from "./Tag";

interface Column {
  status: TicketStatus;
  label: string;
  color: string;
}

const COLUMNS: Column[] = [
  { status: "Open", label: "Open", color: "var(--status-open)" },
  { status: "InProgress", label: "In Progress", color: "var(--status-progress)" },
  { status: "Done", label: "Done", color: "var(--status-done)" },
  { status: "Closed", label: "Closed", color: "var(--status-blocked)" },
];

interface Props {
  tickets: Ticket[];
  shortId: (id: string) => string;
  users: PublicUser[];
  onTicketClick: (id: string) => void;
  onAddTicket: () => void;
}

export function KanbanBoard({ tickets, shortId, users, onTicketClick, onAddTicket }: Props) {
  const userMap = useMemo(() => new Map(users.map((u) => [u.id, u])), [users]);
  const grouped = useMemo(() => {
    const m = new Map<TicketStatus, Ticket[]>();
    for (const c of COLUMNS) m.set(c.status, []);
    for (const t of tickets) {
      // collapse Review → InProgress (4-column design), Closed stays as its own.
      const key: TicketStatus = t.status === "Review" ? "InProgress" : t.status;
      const arr = m.get(key);
      if (arr) arr.push(t);
    }
    return m;
  }, [tickets]);

  return (
    <div className="board">
      {COLUMNS.map((c) => {
        const items = grouped.get(c.status) ?? [];
        return (
          <div key={c.status} className="board-col">
            <div className="board-col-head">
              <div className="name">
                <span className="dot" style={{ background: c.color }} />
                <span>{c.label}</span>
                <span className="count">{items.length}</span>
              </div>
              <button
                type="button"
                className="theme-toggle"
                onClick={onAddTicket}
                title="New ticket"
              >
                <Plus size={14} />
              </button>
            </div>
            {items.map((t) => {
              const assignee = t.assignee_id ? userMap.get(t.assignee_id) : null;
              return (
                <div
                  key={t.id}
                  className="board-card"
                  onClick={() => onTicketClick(t.id)}
                  role="button"
                  tabIndex={0}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      onTicketClick(t.id);
                    }
                  }}
                >
                  <div className="bc-id">{shortId(t.id)}</div>
                  <div className="bc-title">{t.title}</div>
                  {t.tags.length > 0 && (
                    <div className="bc-tags" style={{ marginBottom: 8 }}>
                      <TagList tags={t.tags} max={3} />
                    </div>
                  )}
                  <div className="bc-meta">
                    <PriorityBadge priority={t.priority} />
                    {assignee ? (
                      <Avatar id={assignee.id} name={assignee.username} size="sm" />
                    ) : (
                      <span className="muted" style={{ fontSize: 11.5 }}>Unassigned</span>
                    )}
                  </div>
                </div>
              );
            })}
            <button type="button" className="add-card-btn" onClick={onAddTicket}>
              <Plus size={12} /> Add ticket
            </button>
          </div>
        );
      })}
    </div>
  );
}

import type { Ticket } from "../lib/types";
import { StatusPill } from "./StatusPill";
import { PriorityBadge } from "./PriorityBadge";
import { Avatar } from "./Avatar";

interface Props {
  ticket: Ticket;
  shortId: string;
  assigneeName?: string;
  onClick: () => void;
}

function formatDue(iso: string | null): { text: string; overdue: boolean } {
  if (!iso) return { text: "—", overdue: false };
  const d = new Date(iso);
  const now = new Date();
  const overdue = d < now;
  const text = d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
  return { text, overdue };
}

export function TicketRow({ ticket, shortId, assigneeName, onClick }: Props) {
  const due = formatDue(ticket.due_date);
  return (
    <div
      className="tl-row"
      onClick={onClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onClick();
        }
      }}
    >
      <span className="id">{shortId}</span>
      <span className="title" title={ticket.title}>
        {ticket.title}
      </span>
      <StatusPill status={ticket.status} />
      <PriorityBadge priority={ticket.priority} />
      <span>
        {ticket.assignee_id && assigneeName ? (
          <span style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
            <Avatar id={ticket.assignee_id} name={assigneeName} size="sm" />
            <span style={{ fontSize: 12.5 }}>{assigneeName}</span>
          </span>
        ) : (
          <span className="muted" style={{ fontSize: 12.5 }}>Unassigned</span>
        )}
      </span>
      <span className={`due${due.overdue ? " overdue" : ""}`}>{due.text}</span>
      <span className="muted" style={{ fontSize: 12 }}>—</span>
    </div>
  );
}

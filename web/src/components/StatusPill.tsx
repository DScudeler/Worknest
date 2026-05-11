import { statusLabel, toDisplayStatus, type TicketStatus } from "../lib/types";

interface Props {
  status: TicketStatus;
}

export function StatusPill({ status }: Props) {
  const display = toDisplayStatus(status);
  return (
    <span className={`pill status-${display}`}>
      <span className="dot" />
      {statusLabel(status)}
    </span>
  );
}

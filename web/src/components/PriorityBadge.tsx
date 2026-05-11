import { priorityLabel, toDisplayPriority, type Priority } from "../lib/types";

interface Props {
  priority: Priority;
}

export function PriorityBadge({ priority }: Props) {
  const display = toDisplayPriority(priority);
  return (
    <span className={`pri ${display}`}>
      <span className="pri-bars">
        <span />
        <span />
        <span />
      </span>
      {priorityLabel(priority)}
    </span>
  );
}

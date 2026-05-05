import { avatarColor, initials } from "../lib/colors";

interface Props {
  id: string;
  name: string;
  size?: "sm" | "" | "lg" | "xl";
  title?: string;
}

export function Avatar({ id, name, size = "", title }: Props) {
  const cls = ["avatar", "av", size].filter(Boolean).join(" ");
  return (
    <span className={cls} style={{ background: avatarColor(id) }} title={title ?? name}>
      {initials(name)}
    </span>
  );
}

interface StackProps {
  users: { id: string; name: string }[];
  max?: number;
  size?: "sm" | "" | "lg";
}

export function AvatarStack({ users, max = 4, size = "sm" }: StackProps) {
  const shown = users.slice(0, max);
  const extra = users.length - shown.length;
  return (
    <div className="avatar-stack">
      {shown.map((u) => (
        <Avatar key={u.id} id={u.id} name={u.name} size={size} />
      ))}
      {extra > 0 && (
        <span
          className={`avatar av ${size}`}
          style={{ background: "var(--surface-3)", color: "var(--text-2)" }}
          title={`${extra} more`}
        >
          +{extra}
        </span>
      )}
    </div>
  );
}

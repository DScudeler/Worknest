import { avatarColor, initials } from "../lib/colors";

interface Props {
  id: string;
  name: string;
  /// Optional avatar image URL. When set, the image fills the circle and the
  /// initials are hidden (still set as alt text for screen readers).
  avatarUrl?: string | null;
  size?: "sm" | "" | "lg" | "xl";
  title?: string;
}

export function Avatar({ id, name, avatarUrl, size = "", title }: Props) {
  const cls = ["avatar", "av", size].filter(Boolean).join(" ");
  if (avatarUrl) {
    return (
      <span
        className={cls}
        style={{ background: avatarColor(id), overflow: "hidden", padding: 0 }}
        title={title ?? name}
      >
        <img
          src={avatarUrl}
          alt={initials(name)}
          style={{ width: "100%", height: "100%", objectFit: "cover" }}
        />
      </span>
    );
  }
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

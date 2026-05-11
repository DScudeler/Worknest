import { Bell } from "lucide-react";
import { useAuth } from "../state/auth";
import { Avatar } from "./Avatar";
import { ThemeToggle } from "./ThemeToggle";

interface Crumb {
  label: string;
  href?: string;
}

interface Props {
  crumbs: Crumb[];
}

export function Topbar({ crumbs }: Props) {
  const { user } = useAuth();
  return (
    <header className="topbar">
      <div className="crumbs">
        {crumbs.map((c, i) => (
          <span key={i} style={{ display: "inline-flex", alignItems: "center", gap: 8 }}>
            {i > 0 && <span className="sep">/</span>}
            {c.href ? <a href={c.href}>{c.label}</a> : <span>{c.label}</span>}
          </span>
        ))}
      </div>
      <div className="right">
        <button type="button" className="theme-toggle" title="Notifications">
          <Bell size={16} />
        </button>
        <ThemeToggle />
        {user ? (
          <Avatar
            id={user.id}
            name={user.full_name?.trim() || user.username}
            avatarUrl={user.avatar_url}
            size="sm"
          />
        ) : null}
      </div>
    </header>
  );
}

import type { CSSProperties } from "react";

interface Props {
  emoji: string;
  color: string;
  size?: "sm" | "md" | "lg" | "xl";
}

const SIZES: Record<NonNullable<Props["size"]>, number> = {
  sm: 28,
  md: 36,
  lg: 56,
  xl: 72,
};

export function AgentAvatar({ emoji, color, size = "md" }: Props) {
  const px = SIZES[size];
  const style: CSSProperties = {
    width: px,
    height: px,
    background: color,
    fontSize: Math.round(px * 0.5),
    borderRadius: `${Math.round(px * 0.28)}px`,
    boxShadow: "inset 0 0 0 1px rgba(15, 23, 42, 0.06)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    flex: "0 0 auto",
  };
  return (
    <span className="agent-avatar" style={style}>
      {emoji}
    </span>
  );
}

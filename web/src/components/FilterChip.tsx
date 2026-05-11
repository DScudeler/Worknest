import { useEffect, useRef, useState, type ReactNode } from "react";
import { X } from "lucide-react";

interface Option<V extends string> {
  value: V;
  label: string;
}

interface Props<V extends string> {
  label: string;
  value: V | null;
  options: Option<V>[];
  onChange: (v: V | null) => void;
  icon?: ReactNode;
}

export function FilterChip<V extends string>({ label, value, options, onChange, icon }: Props<V>) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDoc = (e: MouseEvent) => {
      if (!ref.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", onDoc);
    return () => document.removeEventListener("mousedown", onDoc);
  }, [open]);

  const active = value !== null;
  const display = active ? options.find((o) => o.value === value)?.label ?? value : label;

  return (
    <div ref={ref} style={{ position: "relative" }}>
      <button
        type="button"
        className={`chip${active ? " active" : ""}`}
        onClick={() => setOpen((o) => !o)}
      >
        {icon}
        <span>
          {active ? `${label}: ${display}` : label}
        </span>
        {active ? (
          <X
            size={11}
            className="x"
            onClick={(e) => {
              e.stopPropagation();
              onChange(null);
            }}
          />
        ) : null}
      </button>
      {open ? (
        <div
          style={{
            position: "absolute",
            top: "calc(100% + 6px)",
            left: 0,
            background: "var(--surface)",
            border: "1px solid var(--border-strong)",
            borderRadius: 10,
            boxShadow: "var(--shadow-md)",
            padding: 4,
            minWidth: 160,
            zIndex: 20,
          }}
        >
          {options.map((o) => (
            <button
              key={o.value}
              type="button"
              className="nav-item"
              onClick={() => {
                onChange(o.value);
                setOpen(false);
              }}
              style={{ fontSize: 13 }}
            >
              {o.label}
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}

interface ToggleChipProps {
  label: string;
  active: boolean;
  onChange: (v: boolean) => void;
  variant?: "fill" | "tint";
}

export function ToggleChip({ label, active, onChange, variant = "fill" }: ToggleChipProps) {
  return (
    <button
      type="button"
      className={`chip${active ? " active" : ""}${variant === "tint" ? " toggle-mine" : ""}`}
      onClick={() => onChange(!active)}
    >
      {label}
    </button>
  );
}

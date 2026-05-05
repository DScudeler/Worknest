import type { TagSlug } from "../lib/types";

interface Props {
  name: string;
  // Optional explicit slug — if absent, slug is derived from name. Phase 6
  // wires this to a real backend tags table; until then UI-only.
  slug?: TagSlug | string;
}

const KNOWN: TagSlug[] = ["bug", "feature", "design", "research", "docs", "chore"];

function classify(name: string, explicit?: string): string {
  const candidate = (explicit ?? name).toLowerCase();
  return KNOWN.includes(candidate as TagSlug) ? candidate : "";
}

export function Tag({ name, slug }: Props) {
  const cls = classify(name, slug);
  return <span className={`tag${cls ? ` ${cls}` : ""}`}>{name}</span>;
}

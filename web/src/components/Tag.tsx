import type { Tag as TagModel } from "../lib/types";
import { tagSlug } from "../lib/types";

interface Props {
  tag: TagModel;
}

/// Render a single tag chip. Uses the CSS palette class for known design
/// slugs (`.tag.bug`, `.tag.feature`, ...) and falls back to the tag's own
/// stored bg/fg colors for unknown names.
export function TagChip({ tag }: Props) {
  const slug = tagSlug(tag);
  if (slug) return <span className={`tag ${slug}`}>{tag.name}</span>;
  return (
    <span
      className="tag"
      style={{ background: tag.color_bg, color: tag.color_fg }}
    >
      {tag.name}
    </span>
  );
}

interface ListProps {
  tags: TagModel[];
  max?: number;
}

export function TagList({ tags, max }: ListProps) {
  const shown = max !== undefined ? tags.slice(0, max) : tags;
  const extra = tags.length - shown.length;
  return (
    <span style={{ display: "inline-flex", gap: 4, flexWrap: "wrap" }}>
      {shown.map((t) => (
        <TagChip key={t.id} tag={t} />
      ))}
      {extra > 0 ? <span className="tag">+{extra}</span> : null}
    </span>
  );
}

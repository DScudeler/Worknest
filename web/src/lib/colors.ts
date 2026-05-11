// Stable per-id color/icon picker. Used until the backend stores explicit
// project covers/icons. Hashes the id to one of the design's 6 cover swatches
// + one of a small emoji set.

const COVERS = ["#fde68a", "#c4b5fd", "#a7f3d0", "#fbcfe8", "#bae6fd", "#fed7aa"] as const;
const EMOJI_POOL = [
  "🌐", "📱", "⚙️", "🎨", "📈", "💬", "🚀", "🔬", "🧪", "📦",
  "🛠️", "🧭", "📊", "🗂️", "🎯", "💡",
];

function hash(input: string): number {
  let h = 2166136261 >>> 0;
  for (let i = 0; i < input.length; i++) {
    h ^= input.charCodeAt(i);
    h = Math.imul(h, 16777619) >>> 0;
  }
  return h;
}

export function projectCover(id: string, explicit?: string | null): string {
  if (explicit) return explicit;
  return COVERS[hash(id) % COVERS.length];
}

export function projectIcon(id: string): string {
  return EMOJI_POOL[hash(`icon-${id}`) % EMOJI_POOL.length];
}

// Color for a person avatar — derived from their id so it's stable across renders.
const AVATAR_COLORS = [
  "#5b5fc7", "#0ea5e9", "#10b981", "#f59e0b",
  "#ec4899", "#8b5cf6", "#14b8a6", "#f43f5e",
];

export function avatarColor(id: string): string {
  return AVATAR_COLORS[hash(id) % AVATAR_COLORS.length];
}

export function initials(name: string): string {
  const parts = name.trim().split(/\s+/);
  if (parts.length === 0) return "?";
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
}

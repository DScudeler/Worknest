// Static palettes used by the persona editor's avatar picker. The full
// catalogue of personas comes from the backend (`personasApi.list()`); this
// file is just the design's emoji + colour swatches plus a helper to pick a
// sensible default for the create flow.

export const AGENT_EMOJIS: string[] = [
  "🛎️",
  "🔍",
  "🐞",
  "📝",
  "☕️",
  "🔬",
  "🧭",
  "🎨",
  "⚙️",
  "✨",
  "🚀",
  "🛠️",
  "📊",
  "🧪",
  "🧹",
  "🔐",
];

export const AGENT_COLORS: string[] = [
  "#bae6fd",
  "#c4b5fd",
  "#fecaca",
  "#a7f3d0",
  "#fde68a",
  "#fbcfe8",
  "#fed7aa",
  "#cbd5e1",
];

/// Pick the first emoji + colour as the default for "from scratch".
export function defaultEmoji(): string {
  return AGENT_EMOJIS[0]!;
}

export function defaultColor(): string {
  return AGENT_COLORS[0]!;
}

import type { KeyBinding } from "./api/types";

export type ShortcutCategory = "Navigation" | "Zoom" | "Panels" | "General";

export interface ShortcutDef {
  id: string;
  label: string;
  category: ShortcutCategory;
  default: KeyBinding;
}

/** Rebindable shortcuts, editable from the settings modal. */
export const SHORTCUTS: ShortcutDef[] = [
  {
    id: "prev-page",
    label: "Previous page",
    category: "Navigation",
    default: { key: "ArrowUp", meta: true },
  },
  {
    id: "next-page",
    label: "Next page",
    category: "Navigation",
    default: { key: "ArrowDown", meta: true },
  },
  {
    id: "zoom-in",
    label: "Zoom in",
    category: "Zoom",
    default: { key: "=", meta: true },
  },
  {
    id: "zoom-out",
    label: "Zoom out",
    category: "Zoom",
    default: { key: "-", meta: true },
  },
  {
    id: "zoom-reset",
    label: "Fit width",
    category: "Zoom",
    default: { key: "0", meta: true },
  },
  {
    id: "toggle-sidebar",
    label: "Toggle files panel",
    category: "Panels",
    default: { key: "ArrowLeft", meta: true },
  },
  {
    id: "toggle-bookmarks",
    label: "Toggle bookmarks panel",
    category: "Panels",
    default: { key: "ArrowRight", meta: true },
  },
  {
    id: "focus-search",
    label: "Focus search",
    category: "General",
    default: { key: "f", meta: true },
  },
];

/** Shortcuts that exist but can't be remapped (fixed keys or native menu accelerators). */
export interface FixedShortcut {
  label: string;
  category: ShortcutCategory;
  keys: string;
}

export const FIXED_SHORTCUTS: FixedShortcut[] = [
  { label: "Open pinned file", category: "General", keys: "⌘1 – ⌘9" },
  { label: "Open PDF…", category: "General", keys: "⌘O" },
  { label: "Change library folder…", category: "General", keys: "⌘⇧O" },
  { label: "Settings…", category: "General", keys: "⌘," },
];

const KEY_LABELS: Record<string, string> = {
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  " ": "Space",
  "=": "+",
};

export function formatBinding(binding: KeyBinding): string {
  const key =
    KEY_LABELS[binding.key] ??
    (binding.key.length === 1 ? binding.key.toUpperCase() : binding.key);
  return binding.meta ? `⌘${key}` : key;
}

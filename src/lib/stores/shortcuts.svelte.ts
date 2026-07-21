import * as api from "../api/commands";
import type { KeyBinding } from "../api/types";
import { SHORTCUTS } from "../shortcuts";

/** Pressing "=" without shift also matches "+", since they share a key on most keyboards. */
const SHIFT_ALIAS: Record<string, string> = { "=": "+" };

function sameBinding(a: KeyBinding, b: KeyBinding): boolean {
  return a.meta === b.meta && a.key.toLowerCase() === b.key.toLowerCase();
}

class ShortcutsStore {
  overrides = $state<Record<string, KeyBinding>>({});

  /** Hydrates overrides from the backend; called once during settings.init(). */
  load(overrides: Record<string, KeyBinding>) {
    this.overrides = overrides;
  }

  get(id: string): KeyBinding {
    return (
      this.overrides[id] ??
      SHORTCUTS.find((s) => s.id === id)?.default ?? {
        key: "",
        meta: true,
      }
    );
  }

  isCustom(id: string): boolean {
    return id in this.overrides;
  }

  /** Id of the other shortcut already using this binding, if any ("pinned-files" for the ⌘1-9 range). */
  findConflict(binding: KeyBinding, excludeId: string): string | null {
    for (const def of SHORTCUTS) {
      if (def.id === excludeId) continue;
      if (sameBinding(this.get(def.id), binding)) return def.id;
    }
    if (binding.meta && /^[1-9]$/.test(binding.key)) return "pinned-files";
    return null;
  }

  set(id: string, binding: KeyBinding) {
    this.overrides = { ...this.overrides, [id]: binding };
    this.persist();
  }

  reset(id: string) {
    const next = { ...this.overrides };
    delete next[id];
    this.overrides = next;
    this.persist();
  }

  resetAll() {
    this.overrides = {};
    this.persist();
  }

  matches(id: string, e: KeyboardEvent): boolean {
    const binding = this.get(id);
    const meta = e.metaKey || e.ctrlKey;
    if (meta !== binding.meta) return false;
    const key = e.key.toLowerCase();
    const target = binding.key.toLowerCase();
    const alias = (SHIFT_ALIAS[binding.key] ?? "").toLowerCase();
    return key === target || (alias !== "" && key === alias);
  }

  private persist() {
    void api.setShortcuts($state.snapshot(this.overrides)).catch(() => {
      // best effort, like reading-position saves
    });
  }
}

export const shortcuts = new ShortcutsStore();

import { open } from "@tauri-apps/plugin-dialog";
import * as api from "../api/commands";
import type { FileEntry, Theme } from "../api/types";
import { shortcuts } from "./shortcuts.svelte";

// "system" is resolved here (rather than left to a CSS media query) so an
// explicit light/dark choice can override the OS preference; `data-theme`
// on <html> is always concrete and drives the overrides in app.css.
const darkMedia = window.matchMedia("(prefers-color-scheme: dark)");

function resolveTheme(theme: Theme): "light" | "dark" {
  return theme === "system" ? (darkMedia.matches ? "dark" : "light") : theme;
}

function applyTheme(theme: Theme) {
  document.documentElement.dataset.theme = resolveTheme(theme);
}

class SettingsStore {
  libraryPath = $state<string | null>(null);
  lastFile = $state<string | null>(null);
  theme = $state<Theme>("system");
  ready = $state(false);

  async init() {
    const settings = await api.getSettings();
    this.libraryPath = settings.libraryPath;
    this.lastFile = settings.lastFile;
    this.theme = settings.theme;
    applyTheme(this.theme);
    darkMedia.addEventListener("change", () => {
      if (this.theme === "system") applyTheme(this.theme);
    });
    shortcuts.load(settings.shortcuts);
    this.ready = true;
  }

  async setTheme(theme: Theme) {
    this.theme = theme;
    applyTheme(theme);
    await api.setTheme(theme);
  }

  /** Opens the native folder picker; returns the fresh file list on success. */
  async chooseLibraryFolder(): Promise<FileEntry[] | null> {
    const dir = await open({
      directory: true,
      title: "Choose your PDF folder",
    });
    if (typeof dir !== "string") return null;
    const files = await api.setLibraryFolder(dir);
    this.libraryPath = dir;
    return files;
  }
}

export const settings = new SettingsStore();

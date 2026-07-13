import { open } from "@tauri-apps/plugin-dialog";
import * as api from "../api/commands";
import type { FileEntry } from "../api/types";

class SettingsStore {
  libraryPath = $state<string | null>(null);
  lastFile = $state<string | null>(null);
  ready = $state(false);

  async init() {
    const settings = await api.getSettings();
    this.libraryPath = settings.libraryPath;
    this.lastFile = settings.lastFile;
    this.ready = true;
  }

  /** Opens the native folder picker; returns the fresh file list on success. */
  async chooseLibraryFolder(): Promise<FileEntry[] | null> {
    const dir = await open({ directory: true, title: "Choose your PDF folder" });
    if (typeof dir !== "string") return null;
    const files = await api.setLibraryFolder(dir);
    this.libraryPath = dir;
    return files;
  }
}

export const settings = new SettingsStore();

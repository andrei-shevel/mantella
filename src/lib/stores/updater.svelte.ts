import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";
import { message } from "@tauri-apps/plugin-dialog";
import { setCheckUpdatesEnabled } from "../api/commands";

export type UpdaterStatus =
  | "idle"
  | "checking"
  | "available"
  | "up-to-date"
  | "downloading"
  | "installing"
  | "error";

function isBusy(status: UpdaterStatus): boolean {
  return (
    status === "checking" || status === "downloading" || status === "installing"
  );
}

class UpdaterStore {
  status = $state<UpdaterStatus>("idle");
  currentVersion = $state("");
  availableVersion = $state<string | null>(null);
  notes = $state<string | null>(null);
  error = $state<string | null>(null);
  downloadedBytes = $state(0);
  totalBytes = $state<number | null>(null);
  /** True when an update was found by a background check (show install prompt). */
  promptOpen = $state(false);

  private pending: Update | null = null;

  get progress(): number | null {
    if (this.totalBytes == null || this.totalBytes === 0) return null;
    return Math.min(1, this.downloadedBytes / this.totalBytes);
  }

  get busy(): boolean {
    return isBusy(this.status);
  }

  async init() {
    try {
      this.currentVersion = await getVersion();
    } catch {
      this.currentVersion = "";
    }
  }

  private setStatus(status: UpdaterStatus) {
    const wasBusy = isBusy(this.status);
    this.status = status;
    const nowBusy = isBusy(status);
    if (wasBusy !== nowBusy) {
      void setCheckUpdatesEnabled(!nowBusy).catch(() => {
        /* menu sync is best-effort */
      });
    }
  }

  /** Quiet startup check — opens a prompt only when an update exists. */
  async checkOnStartup() {
    if (this.busy) return;
    this.setStatus("checking");
    try {
      await this.pending?.close();
      this.pending = null;
      const update = await check();
      if (!update) {
        this.setStatus("idle");
        return;
      }
      this.pending = update;
      this.availableVersion = update.version;
      this.notes = update.body ?? null;
      this.setStatus("available");
      this.promptOpen = true;
    } catch {
      // No release yet, offline, etc. — keep quiet until the user checks manually.
      this.setStatus("idle");
    }
  }

  /**
   * User-initiated check. Opens the install prompt when an update exists;
   * returns the outcome so the caller can show a dialog for other results.
   */
  async checkManually(): Promise<
    "available" | "up-to-date" | "error" | "busy"
  > {
    if (this.busy) return "busy";
    this.setStatus("checking");
    this.error = null;
    try {
      await this.pending?.close();
      this.pending = null;
      const update = await check();
      if (!update) {
        this.availableVersion = null;
        this.notes = null;
        this.setStatus("up-to-date");
        return "up-to-date";
      }
      this.pending = update;
      this.availableVersion = update.version;
      this.notes = update.body ?? null;
      this.setStatus("available");
      this.promptOpen = true;
      return "available";
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      this.setStatus("error");
      return "error";
    }
  }

  dismissPrompt() {
    this.promptOpen = false;
  }

  async installAndRelaunch() {
    if (this.busy) return;
    if (!this.pending) {
      const result = await this.checkManually();
      if (result !== "available" || !this.pending) return;
    }
    // Another click may have started install while we were checking.
    if (this.busy) return;
    this.promptOpen = false;
    this.setStatus("downloading");
    this.error = null;
    this.downloadedBytes = 0;
    this.totalBytes = null;
    try {
      await this.pending.downloadAndInstall((event) => {
        if (event.event === "Started") {
          this.totalBytes = event.data.contentLength ?? null;
          this.downloadedBytes = 0;
        } else if (event.event === "Progress") {
          this.downloadedBytes += event.data.chunkLength;
        } else if (event.event === "Finished") {
          this.setStatus("installing");
        }
      });
      this.pending = null;
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      this.setStatus("error");
      return;
    }

    // Install already applied; relaunch is best-effort so a failure here must
    // not look like a failed update.
    try {
      await relaunch();
    } catch (e) {
      const detail = e instanceof Error ? e.message : String(e);
      const restartHint = `Update installed, but the app could not restart (${detail}). Quit and reopen Mantella.`;
      this.error = restartHint;
      this.setStatus("idle");
      await message(restartHint, {
        title: "Restart required",
        kind: "info",
      }).catch(() => {
        /* dialog is best-effort */
      });
    }
  }
}

export const updater = new UpdaterStore();

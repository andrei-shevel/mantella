import * as api from "../api/commands";
import type { PageSize } from "../api/types";

const SAVE_DEBOUNCE_MS = 800;

class ReaderStore {
  path = $state<string | null>(null);
  docId = $state<number | null>(null);
  pages = $state<PageSize[]>([]);
  /** null = fit to width */
  zoom = $state<number | null>(null);
  currentPage = $state(1);
  loading = $state(false);
  error = $state<string | null>(null);

  name = $derived(this.path?.split("/").pop()?.replace(/\.pdf$/i, "") ?? "");

  /** Position to restore, consumed by the Viewer once layout is ready. */
  pendingRestore: { page: number; offset: number } | null = null;

  private saveTimer: ReturnType<typeof setTimeout> | null = null;
  private lastPageOffset = 0;

  async open(path: string) {
    if (path === this.path) return;
    await this.flushSave();
    if (this.docId !== null) void api.closeDocument(this.docId);

    this.path = path;
    this.docId = null;
    this.pages = [];
    this.error = null;
    this.loading = true;

    try {
      const result = await api.openDocument(path);
      if (this.path !== path) {
        // user switched to another file while this one was loading
        void api.closeDocument(result.docId);
        return;
      }
      this.zoom = result.state.zoom;
      this.currentPage = Math.min(Math.max(result.state.page, 1), result.pageCount || 1);
      this.pendingRestore = { page: this.currentPage, offset: result.state.pageOffset };
      this.lastPageOffset = result.state.pageOffset;
      this.pages = result.pages;
      this.docId = result.docId;
    } catch (e) {
      if (this.path === path) this.error = String(e);
    } finally {
      if (this.path === path) this.loading = false;
    }
  }

  /** Drop the current document (e.g. the file vanished from the library). */
  close() {
    if (this.docId !== null) void api.closeDocument(this.docId);
    this.path = null;
    this.docId = null;
    this.pages = [];
    this.error = null;
    this.loading = false;
  }

  reportScroll(page: number, pageOffset: number) {
    this.currentPage = page;
    this.lastPageOffset = pageOffset;
    this.scheduleSave();
  }

  setZoom(zoom: number | null) {
    this.zoom = zoom;
    this.scheduleSave();
  }

  private scheduleSave() {
    if (!this.path) return;
    if (this.saveTimer) clearTimeout(this.saveTimer);
    this.saveTimer = setTimeout(() => void this.flushSave(), SAVE_DEBOUNCE_MS);
  }

  async flushSave() {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer);
      this.saveTimer = null;
    }
    if (!this.path || this.docId === null) return;
    try {
      await api.saveReadingState(this.path, this.currentPage, this.lastPageOffset, this.zoom);
    } catch {
      // best effort; losing one position save is fine
    }
  }
}

export const reader = new ReaderStore();

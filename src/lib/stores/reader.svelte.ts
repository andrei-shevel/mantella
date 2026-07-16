import * as api from "../api/commands";
import type { Bookmark, PageSize } from "../api/types";

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
  bookmarks = $state<Bookmark[]>([]);

  name = $derived(this.path?.split("/").pop()?.replace(/\.pdf$/i, "") ?? "");

  /**
   * Bookmarks in document order, for display. Offsets may be negative (an
   * anchor just above its page's top edge), so sorting by (page, offset)
   * alone would misplace those; sort by the absolute document position.
   */
  bookmarksSorted = $derived.by(() => {
    const tops: number[] = [];
    let y = 0;
    for (const p of this.pages) {
      tops.push(y);
      y += p.height;
    }
    const pos = (b: Bookmark) => {
      const i = Math.min(Math.max(b.page, 1), this.pages.length) - 1;
      return (tops[i] ?? 0) + b.pageOffset * (this.pages[i]?.height ?? 0);
    };
    return [...this.bookmarks].sort((a, b) => pos(a) - pos(b));
  });

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
    this.bookmarks = [];
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
      this.bookmarks = result.state.bookmarks ?? [];
      this.pages = result.pages;
      this.docId = result.docId;
      void api.setLastFile(path);
    } catch (e) {
      if (this.path === path) this.error = String(e);
    } finally {
      if (this.path === path) this.loading = false;
    }
  }

  /** Drop the current document (e.g. the file vanished from the library). */
  close() {
    if (this.docId !== null) void api.closeDocument(this.docId);
    void api.setLastFile(null);
    this.path = null;
    this.docId = null;
    this.pages = [];
    this.bookmarks = [];
    this.error = null;
    this.loading = false;
  }

  /** The current reading position, in the same anchor form as bookmarks. */
  get currentAnchor(): { page: number; offset: number } {
    return { page: this.currentPage, offset: this.lastPageOffset };
  }

  addBookmark() {
    const n =
      Math.max(0, ...this.bookmarks.map((b) => Number(/^Bookmark #(\d+)$/.exec(b.title)?.[1] ?? 0))) + 1;
    const anchor = this.currentAnchor;
    this.bookmarks.push({
      id: crypto.randomUUID(),
      title: `Bookmark #${n}`,
      page: anchor.page,
      pageOffset: anchor.offset,
    });
    this.persistBookmarks();
  }

  renameBookmark(id: string, title: string) {
    const trimmed = title.trim();
    const bookmark = this.bookmarks.find((b) => b.id === id);
    if (!bookmark || !trimmed || bookmark.title === trimmed) return;
    bookmark.title = trimmed;
    this.persistBookmarks();
  }

  removeBookmark(id: string) {
    this.bookmarks = this.bookmarks.filter((b) => b.id !== id);
    this.persistBookmarks();
  }

  moveBookmark(id: string, page: number, pageOffset: number) {
    const bookmark = this.bookmarks.find((b) => b.id === id);
    if (!bookmark) return;
    bookmark.page = page;
    bookmark.pageOffset = pageOffset;
    this.persistBookmarks();
  }

  private persistBookmarks() {
    if (!this.path) return;
    void api.saveBookmarks(this.path, $state.snapshot(this.bookmarks)).catch(() => {
      // best effort, like reading-position saves
    });
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

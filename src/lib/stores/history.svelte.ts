import { reader } from "./reader.svelte";

export interface HistoryEntry {
  path: string;
  /**
   * Position within the file (offset = fraction of the page's height, same
   * anchor form as bookmarks). null = the file's own persisted position — the
   * file never finished loading, so its initial position stands.
   */
  anchor: { page: number; offset: number } | null;
}

type Anchor = { page: number; offset: number };

const MAX_ENTRIES = 100;

/**
 * Navigation history over file opens, bookmark jumps and internal links: a
 * list of visited locations with a cursor, browser-like. Every arrival — a
 * file's open position, a jump target, a back/forward landing — is a fixed
 * item that scrolling never redefines. Instead, when the user navigates away
 * after scrolling, the scrolled-to position is inserted as its own item, so
 * both the arrival point and the reading spot stay reachable.
 *
 * Note: this singleton shadows window.history in importing modules.
 */
class HistoryStore {
  private entries = $state<HistoryEntry[]>([]);
  private idx = $state(-1);

  canGoBack = $derived(this.idx > 0);
  canGoForward = $derived(this.idx < this.entries.length - 1);

  /** File open by the user (not via back/forward): becomes the new current item. */
  recordOpen(path: string) {
    this.commit({ path, anchor: null });
  }

  /**
   * Called when an open resolves: fixes the entry's position to the one the
   * file actually opened at, so later saves/scrolls don't move it.
   */
  confirmOpen(path: string, anchor: Anchor) {
    const entry = this.entries[this.idx];
    if (entry && entry.path === path && entry.anchor === null) {
      entry.anchor = { ...anchor };
    }
  }

  /** Bookmark or internal-link jump within the open document. */
  navigate(anchor: Anchor) {
    if (reader.path === null || reader.docId === null) return;
    this.commit({ path: reader.path, anchor: { ...anchor } });
    reader.jumpTo(anchor);
  }

  async goBack() {
    if (this.idx <= 0) return;
    this.materializeDrift(); // inserts after idx: the back target is unaffected
    this.idx -= 1;
    this.trim();
    await this.navigateTo(this.entries[this.idx]);
  }

  async goForward() {
    if (this.idx < 0 || this.idx >= this.entries.length - 1) return;
    const inserted = this.materializeDrift();
    this.idx += inserted ? 2 : 1;
    this.trim();
    await this.navigateTo(this.entries[this.idx]);
  }

  /** Append a new current item; any forward tail is discarded. */
  private commit(target: HistoryEntry) {
    this.entries.splice(this.idx + 1);
    this.materializeDrift();
    const current = this.entries[this.idx];
    // jumping to where we already are shouldn't add an item
    if (
      current &&
      current.path === target.path &&
      current.anchor !== null &&
      target.anchor !== null &&
      this.samePosition(current.anchor, target.anchor)
    ) {
      return;
    }
    this.entries.push(target);
    this.idx = this.entries.length - 1;
    this.trim();
  }

  /**
   * If the user scrolled away from the current item since arriving, insert
   * the live position as its own item right after it — the arrival item
   * itself is never redefined. Returns whether an item was inserted.
   */
  private materializeDrift(): boolean {
    const current = this.entries[this.idx];
    if (!current || reader.path !== current.path || reader.docId === null) return false;
    const live = { ...reader.currentAnchor };
    if (current.anchor === null) {
      // open never confirmed (e.g. it errored before loading finished, then
      // recovered); adopt the live position as the arrival
      current.anchor = live;
      return false;
    }
    if (this.samePosition(current.anchor, live)) return false;
    this.entries.splice(this.idx + 1, 0, { path: current.path, anchor: live });
    return true;
  }

  private async navigateTo(entry: HistoryEntry) {
    if (entry.path === reader.path && reader.docId !== null) {
      if (entry.anchor) reader.jumpTo(entry.anchor);
      return;
    }
    await reader.open(entry.path, {
      restore: entry.anchor ?? undefined,
      fromHistory: true,
    });
  }

  /**
   * Whether two anchors are effectively the same reading position. Compared
   * by absolute document position (in PDF points, ignoring inter-page gaps),
   * since the viewer may express the same scroll offset relative to a
   * different page than the jump target used.
   */
  private samePosition(a: Anchor, b: Anchor): boolean {
    if (a.page === b.page) return Math.abs(a.offset - b.offset) < 0.02;
    const pages = reader.pages;
    if (pages.length === 0) return false;
    const docY = (anchor: Anchor) => {
      const page = Math.min(Math.max(anchor.page, 1), pages.length);
      let y = 0;
      for (let i = 0; i < page - 1; i++) y += pages[i].height;
      return y + anchor.offset * pages[page - 1].height;
    };
    const h = pages[Math.min(Math.max(a.page, 1), pages.length) - 1].height;
    return Math.abs(docY(a) - docY(b)) < h * 0.1;
  }

  private trim() {
    const overflow = this.entries.length - MAX_ENTRIES;
    if (overflow <= 0) return;
    this.entries.splice(0, overflow);
    this.idx = Math.max(0, this.idx - overflow);
  }
}

export const history = new HistoryStore();

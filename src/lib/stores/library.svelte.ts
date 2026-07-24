import { SvelteSet } from "svelte/reactivity";
import * as api from "../api/commands";
import { onLibraryChanged } from "../api/events";
import type { FileEntry } from "../api/types";
import { reader } from "./reader.svelte";
import { settings } from "./settings.svelte";

export interface DirNode {
  name: string;
  relPath: string;
  /** Absolute path; used for the "Open in Finder" context menu action. */
  path: string;
  dirs: DirNode[];
  files: FileEntry[];
}

/** A row in the sidebar as keyboard navigation sees it, in on-screen order. */
export type FlatRow =
  | { key: string; type: "file"; file: FileEntry }
  | { key: string; type: "dir"; dir: DirNode };

export function pinnedKey(file: FileEntry): string {
  return `pin:${file.path}`;
}
export function fileKey(file: FileEntry): string {
  return `file:${file.path}`;
}
export function dirKey(dir: DirNode): string {
  return `dir:${dir.relPath}`;
}

/** DOM id for a row, referenced by the tree container's aria-activedescendant. */
export function rowDomId(key: string): string {
  return `row-${encodeURIComponent(key)}`;
}

/** Walks the tree in the same depth-first order FolderTree.svelte renders it. */
function flattenTree(
  node: DirNode,
  expanded: ReadonlySet<string>,
  out: FlatRow[],
): void {
  for (const dir of node.dirs) {
    out.push({ key: dirKey(dir), type: "dir", dir });
    if (expanded.has(dir.relPath)) flattenTree(dir, expanded, out);
  }
  for (const file of node.files) {
    out.push({ key: fileKey(file), type: "file", file });
  }
}

const EXPANDED_KEY = "mantella.expanded-dirs";

function loadExpanded(): string[] {
  try {
    // fall back to the pre-rename key so expanded state carries over
    const raw =
      localStorage.getItem(EXPANDED_KEY) ??
      localStorage.getItem("magpie.expanded-dirs");
    return JSON.parse(raw ?? "[]");
  } catch {
    return [];
  }
}

function buildTree(files: FileEntry[], root: string): DirNode {
  const rootNode: DirNode = {
    name: "",
    relPath: "",
    path: root,
    dirs: [],
    files: [],
  };
  const dirMap = new Map<string, DirNode>([["", rootNode]]);

  const ensureDir = (relPath: string): DirNode => {
    const existing = dirMap.get(relPath);
    if (existing) return existing;
    const slash = relPath.lastIndexOf("/");
    const parent = ensureDir(slash === -1 ? "" : relPath.slice(0, slash));
    const node: DirNode = {
      name: relPath.slice(slash + 1),
      relPath,
      path: `${root}/${relPath}`,
      dirs: [],
      files: [],
    };
    parent.dirs.push(node);
    dirMap.set(relPath, node);
    return node;
  };

  for (const file of files) {
    const slash = file.relPath.lastIndexOf("/");
    ensureDir(slash === -1 ? "" : file.relPath.slice(0, slash)).files.push(
      file,
    );
  }

  const sortDirs = (node: DirNode) => {
    node.dirs.sort((a, b) => a.name.localeCompare(b.name));
    node.dirs.forEach(sortDirs);
  };
  sortDirs(rootNode); // files are already name-sorted by the scanner

  return rootNode;
}

class LibraryStore {
  files = $state<FileEntry[]>([]);
  query = $state("");

  /** Folders expanded by the user; everything else renders collapsed. */
  expanded = new SvelteSet<string>(loadExpanded());

  filtered = $derived.by(() => {
    const q = this.query.trim().toLowerCase();
    if (!q) return this.files;
    return this.files.filter((f) => f.name.toLowerCase().includes(q));
  });
  pinned = $derived(this.files.filter((f) => f.pinned));
  /** Files under the library root; excludes pinned outsiders. */
  libraryFiles = $derived(this.files.filter((f) => f.inLibrary));
  tree = $derived(buildTree(this.libraryFiles, settings.libraryPath ?? ""));

  /** Row the keyboard cursor points at; key scheme matches flatRows. */
  cursor = $state<string | null>(null);
  /** Whether the sidebar's tree/list container currently holds keyboard focus. */
  listFocused = $state(false);
  /** The sidebar's tree/list container; set by Sidebar, focused from elsewhere (e.g. ⌘←). */
  listEl: HTMLElement | null = null;

  focusList() {
    this.listEl?.focus();
  }

  /** Rows in on-screen order, for arrow-key traversal (see flattenTree). */
  flatRows = $derived.by((): FlatRow[] => {
    if (this.query.trim()) {
      return this.filtered.map(
        (file): FlatRow => ({ key: fileKey(file), type: "file", file }),
      );
    }
    const rows: FlatRow[] = this.pinned.map(
      (file): FlatRow => ({ key: pinnedKey(file), type: "file", file }),
    );
    flattenTree(this.tree, this.expanded, rows);
    return rows;
  });

  cursorRow = $derived(this.flatRows.find((r) => r.key === this.cursor));

  setCursor(key: string) {
    this.cursor = key;
  }

  /** Points the cursor at something sensible if it's unset or no longer visible. */
  ensureCursor() {
    if (this.cursor && this.flatRows.some((r) => r.key === this.cursor)) return;
    const activeFile = this.flatRows.find(
      (r) => r.type === "file" && r.file.path === reader.path,
    );
    this.cursor = activeFile?.key ?? this.flatRows[0]?.key ?? null;
  }

  moveCursor(delta: number) {
    const rows = this.flatRows;
    if (rows.length === 0) return;
    const idx = rows.findIndex((r) => r.key === this.cursor);
    if (idx === -1) {
      this.cursor = delta > 0 ? rows[0].key : rows[rows.length - 1].key;
      return;
    }
    const next = Math.min(Math.max(idx + delta, 0), rows.length - 1);
    this.cursor = rows[next].key;
  }

  moveCursorToEdge(start: boolean) {
    const rows = this.flatRows;
    if (rows.length === 0) return;
    this.cursor = start ? rows[0].key : rows[rows.length - 1].key;
  }

  /** Right arrow: expand a collapsed folder, or step into an expanded one's first child. */
  expandCursor() {
    const row = this.cursorRow;
    if (!row || row.type !== "dir") return;
    if (!this.expanded.has(row.dir.relPath)) this.toggleDir(row.dir.relPath);
    else this.moveCursor(1);
  }

  /** Left arrow: collapse an expanded folder, or step out to the parent folder. */
  collapseCursor() {
    const row = this.cursorRow;
    if (!row) return;
    if (row.type === "dir" && this.expanded.has(row.dir.relPath)) {
      this.toggleDir(row.dir.relPath);
      return;
    }
    const relPath = row.type === "dir" ? row.dir.relPath : row.file.relPath;
    const slash = relPath.lastIndexOf("/");
    if (slash === -1) return;
    const parentKey = `dir:${relPath.slice(0, slash)}`;
    if (this.flatRows.some((r) => r.key === parentKey)) this.cursor = parentKey;
  }

  /** Enter/Space: open a file and move focus to the reader, or toggle a folder. */
  activateCursor() {
    const row = this.cursorRow;
    if (!row) return;
    if (row.type === "file") {
      void reader.open(row.file.path);
      reader.focusViewer();
    } else {
      this.toggleDir(row.dir.relPath);
    }
  }

  toggleDir(relPath: string) {
    if (this.expanded.has(relPath)) this.expanded.delete(relPath);
    else this.expanded.add(relPath);
    try {
      localStorage.setItem(EXPANDED_KEY, JSON.stringify([...this.expanded]));
    } catch {
      // persistence is best-effort
    }
  }

  async refresh() {
    this.files = await api.getLibrary();
  }

  setFiles(files: FileEntry[]) {
    this.files = files;
  }

  /** Subscribe to watcher pushes from the backend. */
  async listen() {
    await onLibraryChanged((files) => {
      const hadOpenFile =
        reader.path !== null && this.files.some((f) => f.path === reader.path);
      this.files = files;
      if (hadOpenFile && !files.some((f) => f.path === reader.path)) {
        reader.close();
      }
    });
  }

  async togglePin(file: FileEntry) {
    const pinned = !file.pinned;
    await api.setPinned(file.id, file.path, pinned);
    // an outside-the-library file is only listed because of its pin
    // (matched by id, not path: duplicate-content files share identity and
    // flip together)
    this.files =
      pinned || file.inLibrary
        ? this.files.map((f) => (f.id === file.id ? { ...f, pinned } : f))
        : this.files.filter((f) => f.id !== file.id);
  }

  /** Pin a file that isn't in the list yet (opened from outside the library). */
  async pinExternal(id: string, path: string) {
    await api.setPinned(id, path, true);
    await this.refresh();
  }
}

export const library = new LibraryStore();

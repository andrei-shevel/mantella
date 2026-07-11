import { SvelteSet } from "svelte/reactivity";
import * as api from "../api/commands";
import { onLibraryChanged } from "../api/events";
import type { FileEntry } from "../api/types";
import { reader } from "./reader.svelte";

export interface DirNode {
  name: string;
  relPath: string;
  dirs: DirNode[];
  files: FileEntry[];
}

const EXPANDED_KEY = "mantella.expanded-dirs";

function loadExpanded(): string[] {
  try {
    // fall back to the pre-rename key so expanded state carries over
    const raw = localStorage.getItem(EXPANDED_KEY) ?? localStorage.getItem("magpie.expanded-dirs");
    return JSON.parse(raw ?? "[]");
  } catch {
    return [];
  }
}

function buildTree(files: FileEntry[]): DirNode {
  const root: DirNode = { name: "", relPath: "", dirs: [], files: [] };
  const dirMap = new Map<string, DirNode>([["", root]]);

  const ensureDir = (relPath: string): DirNode => {
    const existing = dirMap.get(relPath);
    if (existing) return existing;
    const slash = relPath.lastIndexOf("/");
    const parent = ensureDir(slash === -1 ? "" : relPath.slice(0, slash));
    const node: DirNode = { name: relPath.slice(slash + 1), relPath, dirs: [], files: [] };
    parent.dirs.push(node);
    dirMap.set(relPath, node);
    return node;
  };

  for (const file of files) {
    const slash = file.relPath.lastIndexOf("/");
    ensureDir(slash === -1 ? "" : file.relPath.slice(0, slash)).files.push(file);
  }

  const sortDirs = (node: DirNode) => {
    node.dirs.sort((a, b) => a.name.localeCompare(b.name));
    node.dirs.forEach(sortDirs);
  };
  sortDirs(root); // files are already name-sorted by the scanner

  return root;
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
  tree = $derived(buildTree(this.libraryFiles));

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
    await api.setPinned(file.path, pinned);
    // an outside-the-library file is only listed because of its pin
    this.files =
      pinned || file.inLibrary
        ? this.files.map((f) => (f.path === file.path ? { ...f, pinned } : f))
        : this.files.filter((f) => f.path !== file.path);
  }

  /** Pin a file that isn't in the list yet (opened from outside the library). */
  async pinExternal(path: string) {
    await api.setPinned(path, true);
    await this.refresh();
  }
}

export const library = new LibraryStore();

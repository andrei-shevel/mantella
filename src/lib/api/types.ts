// Mirrors the Rust DTOs (serde camelCase).

export interface Settings {
  libraryPath: string | null;
}

export interface FileEntry {
  path: string;
  name: string;
  relPath: string;
  size: number;
  modified: number | null;
  pinned: boolean;
}

/**
 * zoom === null means "fit to width".
 * The reading position is page + offset within that page (fraction of page
 * height), which is independent of window size and zoom.
 */
export interface FileState {
  pinned: boolean;
  page: number;
  pageOffset: number;
  zoom: number | null;
  lastOpened: number | null;
}

/** Page dimensions in PDF points (1/72 inch). */
export interface PageSize {
  width: number;
  height: number;
}

export interface OpenResult {
  docId: number;
  pageCount: number;
  pages: PageSize[];
  state: FileState;
}

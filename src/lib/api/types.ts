// Mirrors the Rust DTOs (serde camelCase).

export interface Settings {
  libraryPath: string | null;
  lastFile: string | null;
}

export interface FileEntry {
  path: string;
  name: string;
  /** Relative to the library root; the absolute path for files outside it. */
  relPath: string;
  size: number;
  modified: number | null;
  pinned: boolean;
  /** False for pinned files living outside the library folder. */
  inLibrary: boolean;
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

/**
 * A rectangular run of text on a page. Coordinates are PDF points with a
 * top-left origin, matching the page layout.
 */
export interface TextRun {
  text: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * A clickable link region on a page. Coordinates are PDF points with a
 * top-left origin. Exactly one of `uri` (external target) or `page`
 * (zero-based internal target) is set.
 */
export interface PageLink {
  x: number;
  y: number;
  width: number;
  height: number;
  uri: string | null;
  page: number | null;
}

export interface OpenResult {
  docId: number;
  pageCount: number;
  pages: PageSize[];
  state: FileState;
}

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { FileEntry } from "./types";

export const onLibraryChanged = (
  callback: (files: FileEntry[]) => void,
): Promise<UnlistenFn> => listen<FileEntry[]>("library-changed", (e) => callback(e.payload));

/** The OS asked us to open files; drain them via takePendingOpenFiles. */
export const onOpenFile = (callback: () => void): Promise<UnlistenFn> =>
  listen("open-file", () => callback());

/** File → Open PDF… in the native menu. */
export const onMenuOpenFile = (callback: () => void): Promise<UnlistenFn> =>
  listen("menu-open-file", () => callback());

/** File → Change Library Folder… in the native menu. */
export const onMenuChangeFolder = (callback: () => void): Promise<UnlistenFn> =>
  listen("menu-change-folder", () => callback());

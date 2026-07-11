import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { FileEntry } from "./types";

export const onLibraryChanged = (
  callback: (files: FileEntry[]) => void,
): Promise<UnlistenFn> => listen<FileEntry[]>("library-changed", (e) => callback(e.payload));

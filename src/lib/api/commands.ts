import { invoke } from "@tauri-apps/api/core";
import type { FileEntry, OpenResult, Settings } from "./types";

export const getSettings = () => invoke<Settings>("get_settings");

export const setLibraryFolder = (path: string) =>
  invoke<FileEntry[]>("set_library_folder", { path });

export const getLibrary = () => invoke<FileEntry[]>("get_library");

export const setPinned = (path: string, pinned: boolean) =>
  invoke<void>("set_pinned", { path, pinned });

export const openDocument = (path: string) => invoke<OpenResult>("open_document", { path });

export const takePendingOpenFiles = () => invoke<string[]>("take_pending_open_files");

export const closeDocument = (docId: number) => invoke<void>("close_document", { docId });

export const saveReadingState = (
  path: string,
  page: number,
  pageOffset: number,
  zoom: number | null,
) => invoke<void>("save_reading_state", { path, page, pageOffset, zoom });

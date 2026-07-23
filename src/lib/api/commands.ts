import { invoke } from "@tauri-apps/api/core";
import type {
  Bookmark,
  FileEntry,
  KeyBinding,
  OpenResult,
  PageLink,
  Settings,
  TextRun,
} from "./types";

export const getSettings = () => invoke<Settings>("get_settings");

export const setLibraryFolder = (path: string) =>
  invoke<FileEntry[]>("set_library_folder", { path });

export const getLibrary = () => invoke<FileEntry[]>("get_library");

export const setPinned = (id: string, path: string, pinned: boolean) =>
  invoke<void>("set_pinned", { id, path, pinned });

export const setLastFile = (path: string | null) =>
  invoke<void>("set_last_file", { path });

export const openDocument = (path: string) =>
  invoke<OpenResult>("open_document", { path });

export const takePendingOpenFiles = () =>
  invoke<string[]>("take_pending_open_files");

export const closeDocument = (docId: number) =>
  invoke<void>("close_document", { docId });

export const getPageText = (docId: number, pageIndex: number) =>
  invoke<TextRun[]>("get_page_text", { docId, pageIndex });

export const getPageLinks = (docId: number, pageIndex: number) =>
  invoke<PageLink[]>("get_page_links", { docId, pageIndex });

export const openUrl = (url: string) => invoke<void>("open_url", { url });

export const saveReadingState = (
  id: string,
  page: number,
  pageOffset: number,
  zoom: number | null,
) => invoke<void>("save_reading_state", { id, page, pageOffset, zoom });

export const saveBookmarks = (id: string, bookmarks: Bookmark[]) =>
  invoke<void>("save_bookmarks", { id, bookmarks });

export const setShortcuts = (shortcuts: Record<string, KeyBinding>) =>
  invoke<void>("set_shortcuts", { shortcuts });

export const revealInFinder = (path: string) =>
  invoke<void>("reveal_in_finder", { path });

/** Enable/disable the native Check for Updates… menu item. */
export const setCheckUpdatesEnabled = (enabled: boolean) =>
  invoke<void>("set_check_updates_enabled", { enabled });

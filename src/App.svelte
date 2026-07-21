<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { message, open } from "@tauri-apps/plugin-dialog";
  import { takePendingOpenFiles } from "./lib/api/commands";
  import {
    onMenuChangeFolder,
    onMenuCheckUpdates,
    onMenuOpenFile,
    onMenuOpenSettings,
    onOpenFile,
  } from "./lib/api/events";
  import { settings } from "./lib/stores/settings.svelte";
  import { library } from "./lib/stores/library.svelte";
  import { reader } from "./lib/stores/reader.svelte";
  import { ui } from "./lib/stores/ui.svelte";
  import { shortcuts } from "./lib/stores/shortcuts.svelte";
  import { updater } from "./lib/stores/updater.svelte";
  import Welcome from "./lib/components/onboarding/Welcome.svelte";
  import Sidebar from "./lib/components/library/Sidebar.svelte";
  import Viewer from "./lib/components/reader/Viewer.svelte";
  import ContextMenu from "./lib/components/common/ContextMenu.svelte";
  import SettingsModal from "./lib/components/settings/SettingsModal.svelte";
  import UpdatePrompt from "./lib/components/settings/UpdatePrompt.svelte";

  // Open files handed to us by the OS (Finder "Open With", double-click).
  // Returns whether a file was opened.
  async function openExternal(): Promise<boolean> {
    const paths = await takePendingOpenFiles();
    const last = paths.at(-1);
    if (last) await reader.open(last);
    return last !== undefined;
  }

  // File → Open PDF…
  async function openFileDialog() {
    const file = await open({
      multiple: false,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (typeof file === "string") await reader.open(file);
  }

  // File → Change Library Folder…
  async function changeFolder() {
    const files = await settings.chooseLibraryFolder();
    if (files) library.setFiles(files);
  }

  // App menu → Check for Updates…
  async function checkForUpdates() {
    const result = await updater.checkManually();
    if (result === "busy") {
      // Menu should already be disabled; keep a fallback for races.
      await message("Already checking for updates.", {
        title: "Check for Updates",
        kind: "info",
      });
    } else if (result === "up-to-date") {
      const version = updater.currentVersion
        ? `Mantella ${updater.currentVersion} is up to date.`
        : "Mantella is up to date.";
      await message(version, { title: "Check for Updates", kind: "info" });
    } else if (result === "error") {
      await message(updater.error ?? "Could not check for updates.", {
        title: "Check for Updates",
        kind: "error",
      });
    }
  }

  function onGlobalKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    if (target.tagName === "INPUT" || target.tagName === "TEXTAREA") return;
    if (ui.settingsOpen) return; // the settings modal owns keydown while open
    const meta = e.metaKey || e.ctrlKey;
    if (!meta) return;

    if (shortcuts.matches("toggle-sidebar", e)) {
      e.preventDefault();
      ui.toggleSidebar();
    } else if (shortcuts.matches("toggle-bookmarks", e)) {
      if (reader.docId === null) return; // no doc open: true no-op
      e.preventDefault();
      ui.toggleBookmarksPanel();
    } else if (e.key >= "1" && e.key <= "9") {
      const file = library.pinned[Number(e.key) - 1];
      if (!file) return; // out of range: no-op, not an error
      e.preventDefault();
      void reader.open(file.path);
    }
  }

  onMount(() => {
    const unlisteners: (() => void)[] = [];
    void (async () => {
      await settings.init();
      await updater.init();
      if (settings.libraryPath) await library.refresh();
      await library.listen();
      unlisteners.push(
        await onOpenFile(() => void openExternal()),
        await onMenuOpenFile(() => void openFileDialog()),
        await onMenuChangeFolder(() => void changeFolder()),
        await onMenuOpenSettings(() => ui.openSettings()),
        await onMenuCheckUpdates(() => void checkForUpdates()),
      );
      // A file passed by the OS wins; otherwise restore the last open document.
      const opened = await openExternal();
      if (!opened && settings.lastFile) await reader.open(settings.lastFile);
      // Don't block startup on the network check.
      void updater.checkOnStartup();
    })();

    // Persist the reading position before the window closes.
    const unlisten = getCurrentWindow().onCloseRequested(async () => {
      await reader.flushSave();
    });
    return () => {
      unlisteners.forEach((fn) => fn());
      void unlisten.then((f) => f());
    };
  });
</script>

<svelte:window onkeydown={onGlobalKeydown} />

{#if settings.ready}
  {#if !settings.libraryPath && !reader.path}
    <Welcome />
  {:else}
    <div class="app">
      {#if ui.sidebarOpen}
        <Sidebar />
      {/if}
      <main>
        <Viewer />
      </main>
    </div>
  {/if}
{/if}

<ContextMenu />
<SettingsModal />
<UpdatePrompt />

<style>
  .app {
    display: flex;
    height: 100%;
  }

  main {
    flex: 1;
    min-width: 0;
    position: relative;
  }
</style>

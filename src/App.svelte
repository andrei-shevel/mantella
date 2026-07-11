<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open } from "@tauri-apps/plugin-dialog";
  import { takePendingOpenFiles } from "./lib/api/commands";
  import { onMenuChangeFolder, onMenuOpenFile, onOpenFile } from "./lib/api/events";
  import { settings } from "./lib/stores/settings.svelte";
  import { library } from "./lib/stores/library.svelte";
  import { reader } from "./lib/stores/reader.svelte";
  import { ui } from "./lib/stores/ui.svelte";
  import Welcome from "./lib/components/onboarding/Welcome.svelte";
  import Sidebar from "./lib/components/library/Sidebar.svelte";
  import Viewer from "./lib/components/reader/Viewer.svelte";

  // Open files handed to us by the OS (Finder "Open With", double-click).
  async function openExternal() {
    const paths = await takePendingOpenFiles();
    const last = paths.at(-1);
    if (last) await reader.open(last);
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

  onMount(() => {
    const unlisteners: (() => void)[] = [];
    void (async () => {
      await settings.init();
      if (settings.libraryPath) await library.refresh();
      await library.listen();
      unlisteners.push(
        await onOpenFile(() => void openExternal()),
        await onMenuOpenFile(() => void openFileDialog()),
        await onMenuChangeFolder(() => void changeFolder()),
      );
      await openExternal();
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

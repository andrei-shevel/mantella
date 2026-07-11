<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { settings } from "./lib/stores/settings.svelte";
  import { library } from "./lib/stores/library.svelte";
  import { reader } from "./lib/stores/reader.svelte";
  import { ui } from "./lib/stores/ui.svelte";
  import Welcome from "./lib/components/onboarding/Welcome.svelte";
  import Sidebar from "./lib/components/library/Sidebar.svelte";
  import Viewer from "./lib/components/reader/Viewer.svelte";

  onMount(() => {
    void (async () => {
      await settings.init();
      if (settings.libraryPath) await library.refresh();
      await library.listen();
    })();

    // Persist the reading position before the window closes.
    const unlisten = getCurrentWindow().onCloseRequested(async () => {
      await reader.flushSave();
    });
    return () => void unlisten.then((f) => f());
  });
</script>

{#if settings.ready}
  {#if !settings.libraryPath}
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

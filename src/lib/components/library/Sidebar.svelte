<script lang="ts">
  import { homeDir } from "@tauri-apps/api/path";
  import Icon from "../common/Icon.svelte";
  import FileItem from "./FileItem.svelte";
  import FolderTree from "./FolderTree.svelte";
  import { library } from "../../stores/library.svelte";
  import { settings } from "../../stores/settings.svelte";
  import { ui } from "../../stores/ui.svelte";

  const isMac = navigator.userAgent.includes("Mac");
  let searchEl = $state<HTMLInputElement>();

  let home = $state("");
  void homeDir().then((dir) => (home = dir.replace(/\/+$/, "")));
  /** Library path, with the home directory shortened to "~". */
  let folderPath = $derived.by(() => {
    const path = settings.libraryPath;
    if (!path) return "";
    if (home && (path === home || path.startsWith(home + "/"))) {
      return "~" + path.slice(home.length);
    }
    return path;
  });

  function onKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "f") {
      e.preventDefault();
      searchEl?.focus();
      searchEl?.select();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<aside class:mac={isMac}>
  <header data-tauri-drag-region>
    <button
      class="icon-btn toggle"
      title="Toggle sidebar"
      onclick={() => ui.toggleSidebar()}
    >
      <Icon name="panel-left" />
    </button>
  </header>

  <div class="search">
    <span class="search-icon"><Icon name="search" size={14} /></span>
    <input
      bind:this={searchEl}
      bind:value={library.query}
      placeholder="Search files  ⌘F"
      spellcheck="false"
      autocomplete="off"
    />
  </div>

  <div class="list">
    {#if library.query.trim()}
      <div class="section">Results</div>
      {#each library.filtered as file (file.path)}
        <FileItem {file} />
      {/each}
      {#if library.filtered.length === 0}
        <div class="none">No matches</div>
      {/if}
    {:else}
      {#if library.pinned.length > 0}
        <div class="section">Pinned</div>
        {#each library.pinned as file (file.path)}
          <FileItem {file} showDir={false} />
        {/each}
      {/if}

      <div class="section">Files</div>
      {#if library.libraryFiles.length > 0}
        <FolderTree node={library.tree} />
      {:else}
        <div class="none">No PDFs in this folder yet</div>
      {/if}
    {/if}
  </div>

  <footer>
    <span class="folder-name" title={settings.libraryPath}>{folderPath}</span>
    <span class="file-count">
      {library.libraryFiles.length}
      {library.libraryFiles.length === 1 ? "file" : "files"}
    </span>
  </footer>
</aside>

<style>
  aside {
    display: flex;
    flex-direction: column;
    width: 272px;
    flex-shrink: 0;
    height: 100%;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border);
  }

  header {
    display: flex;
    align-items: center;
    height: 48px;
    padding: 0 14px;
    flex-shrink: 0;
  }

  aside.mac header {
    /* leave room for the macOS traffic lights (overlay title bar) */
    padding-left: 84px;
  }

  .toggle {
    /* keep the toggle on the right, clear of the traffic lights */
    margin-left: auto;
  }

  .search {
    position: relative;
    padding: 0 10px 10px;
    flex-shrink: 0;
  }

  .search-icon {
    position: absolute;
    left: 19px;
    top: 6px;
    color: var(--text-3);
    pointer-events: none;
  }

  .search input {
    width: 100%;
    height: 28px;
    padding: 0 10px 0 30px;
    border: none;
    border-radius: var(--radius);
    background: var(--bg-input);
    outline: none;
    user-select: text;
  }

  .search input:focus {
    box-shadow: 0 0 0 2px var(--accent-soft), 0 0 0 1px var(--accent) inset;
  }

  .search input::placeholder {
    color: var(--text-3);
  }

  .list {
    flex: 1;
    overflow-y: auto;
    padding: 0 8px 8px;
  }

  .section {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-3);
    padding: 12px 8px 5px;
  }

  .none {
    color: var(--text-3);
    text-align: center;
    padding: 28px 12px;
  }

  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex-shrink: 0;
    padding: 8px 16px;
    border-top: 1px solid var(--border);
    color: var(--text-3);
    font-size: 11px;
  }

  .folder-name {
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: 500;
  }

  .file-count {
    flex-shrink: 0;
  }
</style>

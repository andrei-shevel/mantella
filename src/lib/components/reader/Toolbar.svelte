<script lang="ts">
  import Icon from "../common/Icon.svelte";
  import { library } from "../../stores/library.svelte";
  import { reader } from "../../stores/reader.svelte";
  import { ui } from "../../stores/ui.svelte";

  let {
    zoomBy,
    setZoom,
    scrollToPage,
    zoomPercent,
    pageCount,
  }: {
    zoomBy: (factor: number) => void;
    setZoom: (zoom: number | null) => void;
    scrollToPage: (page: number) => void;
    zoomPercent: number;
    pageCount: number;
  } = $props();

  const isMac = navigator.userAgent.includes("Mac");
  let hasDoc = $derived(reader.docId !== null);
  /** The open document's list entry; absent for unpinned files outside the library. */
  let currentFile = $derived(library.files.find((f) => f.path === reader.path));

  async function togglePin() {
    if (currentFile) await library.togglePin(currentFile);
    else if (reader.path) await library.pinExternal(reader.path);
  }

  async function closeDoc() {
    await reader.flushSave(); // keep the reading position before dropping the doc
    reader.close();
  }

  function onPageInput(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const n = parseInt(input.value, 10);
    if (!Number.isNaN(n)) scrollToPage(n);
    input.blur();
  }
</script>

<div
  class="toolbar"
  class:pad-mac={isMac && !ui.sidebarOpen}
  data-tauri-drag-region
>
  {#if !ui.sidebarOpen}
    <button
      class="icon-btn"
      title="Toggle sidebar"
      onclick={() => ui.toggleSidebar()}
    >
      <Icon name="panel-left" />
    </button>
  {/if}

  <span class="title" data-tauri-drag-region>{reader.name}</span>

  {#if reader.path !== null}
    {@const pinned = currentFile?.pinned ?? false}
    <button
      class="icon-btn pin"
      class:pinned
      title={pinned ? "Unpin" : "Pin for quick access"}
      onclick={() => void togglePin()}
    >
      <Icon name="pin" size={14} filled={pinned} />
    </button>
  {/if}

  <div class="spacer" data-tauri-drag-region></div>

  {#if hasDoc}
    <div class="pages">
      <input
        type="text"
        inputmode="numeric"
        value={reader.currentPage}
        onchange={onPageInput}
        aria-label="Current page"
      />
      <span class="count">/ {pageCount}</span>
    </div>

    <div class="zoom">
      <button
        class="icon-btn"
        title="Zoom out (⌘−)"
        onclick={() => zoomBy(1 / 1.15)}
      >
        <Icon name="minus" />
      </button>
      <button class="pct" title="Reset to 100%" onclick={() => setZoom(1)}>
        {zoomPercent}%
      </button>
      <button
        class="icon-btn"
        title="Zoom in (⌘+)"
        onclick={() => zoomBy(1.15)}
      >
        <Icon name="plus" />
      </button>
      <button
        class="icon-btn"
        class:on={reader.zoom === null}
        title="Fit width (⌘0)"
        onclick={() => setZoom(null)}
      >
        <Icon name="fit-width" />
      </button>
    </div>

    <button
      class="icon-btn"
      class:on={ui.bookmarksPanelOpen}
      title="Bookmarks"
      onclick={() => ui.toggleBookmarksPanel()}
    >
      <Icon name="bookmark" size={15} />
    </button>
  {/if}

  {#if reader.path !== null}
    <button
      class="icon-btn"
      title="Close document"
      onclick={() => void closeDoc()}
    >
      <Icon name="x" />
    </button>
  {/if}
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 48px;
    padding: 0 12px;
    flex-shrink: 0;
    background: var(--bg-toolbar);
    backdrop-filter: blur(14px);
    border-bottom: 1px solid var(--border);
    z-index: 2;
  }

  .toolbar.pad-mac {
    /* room for macOS traffic lights when the sidebar is collapsed */
    padding-left: 84px;
  }

  .title {
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: 600;
    color: var(--text-1);
  }

  .pin {
    flex-shrink: 0;
    color: var(--text-3);
  }

  .pin.pinned {
    color: var(--accent);
  }

  .spacer {
    flex: 1;
    align-self: stretch;
  }

  .pages {
    display: flex;
    align-items: center;
    gap: 5px;
    flex-shrink: 0;
    white-space: nowrap;
    color: var(--text-2);
  }

  .pages input {
    width: 42px;
    height: 24px;
    text-align: center;
    border: none;
    border-radius: 6px;
    background: var(--bg-input);
    outline: none;
    user-select: text;
  }

  .pages input:focus {
    box-shadow: 0 0 0 1.5px var(--accent);
  }

  .count {
    font-variant-numeric: tabular-nums;
  }

  .zoom {
    display: flex;
    align-items: center;
    gap: 1px;
    flex-shrink: 0;
  }

  .pct {
    min-width: 46px;
    height: 26px;
    padding: 0 6px;
    border-radius: 6px;
    color: var(--text-2);
    font-variant-numeric: tabular-nums;
    text-align: center;
  }

  .pct:hover {
    background: var(--hover);
    color: var(--text-1);
  }

  .icon-btn.on {
    color: var(--accent);
    background: var(--accent-soft);
  }
</style>

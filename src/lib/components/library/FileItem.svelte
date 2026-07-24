<script lang="ts">
  import Icon from "../common/Icon.svelte";
  import * as api from "../../api/commands";
  import { reader } from "../../stores/reader.svelte";
  import { ui } from "../../stores/ui.svelte";
  import { formatBytes } from "../../utils/format";
  import {
    library,
    pinnedKey,
    fileKey,
    rowDomId,
  } from "../../stores/library.svelte";
  import type { FileEntry } from "../../api/types";

  let {
    file,
    depth = 0,
    showDir = true,
    pinned = false,
  }: {
    file: FileEntry;
    depth?: number;
    showDir?: boolean;
    pinned?: boolean;
  } = $props();

  let key = $derived(pinned ? pinnedKey(file) : fileKey(file));
  let active = $derived(reader.path === file.path);
  let isCursor = $derived(library.cursor === key && library.listFocused);
  let dir = $derived(
    showDir && file.relPath.includes("/")
      ? file.relPath.slice(0, file.relPath.lastIndexOf("/"))
      : "",
  );
</script>

<!-- svelte-ignore a11y_interactive_supports_focus -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- keyboard interaction is owned by the tree container (roving cursor via aria-activedescendant), not this row -->
<div
  class="item"
  class:active
  class:cursor={isCursor}
  style="padding-left: {8 + depth * 14}px"
  role="treeitem"
  id={rowDomId(key)}
  aria-selected={isCursor}
  title="{file.relPath} · {formatBytes(file.size)}"
  onclick={() => {
    library.setCursor(key);
    void reader.open(file.path);
  }}
  oncontextmenu={(e) =>
    ui.openContextMenu(e, [
      {
        label: "Open in Finder",
        action: () => void api.revealInFinder(file.path),
      },
    ])}
>
  <span class="doc-icon"><Icon name="file" size={15} /></span>
  <span class="labels">
    <span class="name">{file.name}</span>
    {#if dir}<span class="dir">{dir}</span>{/if}
  </span>
</div>

<style>
  .item {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 34px; /* keeps rows the height they had with the old pin button */
    padding: 5px 8px;
    border-radius: var(--radius);
    outline: none;
  }

  .item:hover {
    background: var(--hover);
  }

  .item.cursor {
    box-shadow: 0 0 0 2px var(--accent);
  }

  .item.active {
    background: var(--accent-soft);
  }

  .doc-icon {
    display: flex;
    color: var(--text-3);
    flex-shrink: 0;
  }

  .item.active .doc-icon {
    color: var(--accent);
  }

  .labels {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    line-height: 1.3;
  }

  .name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item.active .name {
    color: var(--accent-strong);
    font-weight: 500;
  }

  .dir {
    font-size: 11px;
    color: var(--text-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>

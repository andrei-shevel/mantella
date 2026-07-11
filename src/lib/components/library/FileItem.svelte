<script lang="ts">
  import Icon from "../common/Icon.svelte";
  import { library } from "../../stores/library.svelte";
  import { reader } from "../../stores/reader.svelte";
  import { formatBytes } from "../../utils/format";
  import type { FileEntry } from "../../api/types";

  let {
    file,
    depth = 0,
    showDir = true,
  }: { file: FileEntry; depth?: number; showDir?: boolean } = $props();

  let active = $derived(reader.path === file.path);
  let dir = $derived(
    showDir && file.relPath.includes("/")
      ? file.relPath.slice(0, file.relPath.lastIndexOf("/"))
      : "",
  );
</script>

<div
  class="item"
  class:active
  style="padding-left: {8 + depth * 14}px"
  role="button"
  tabindex="0"
  title="{file.relPath} · {formatBytes(file.size)}"
  onclick={() => void reader.open(file.path)}
  onkeydown={(e) => {
    if (e.key === "Enter") void reader.open(file.path);
  }}
>
  <span class="doc-icon"><Icon name="file" size={15} /></span>
  <span class="labels">
    <span class="name">{file.name}</span>
    {#if dir}<span class="dir">{dir}</span>{/if}
  </span>
  <button
    class="icon-btn pin"
    class:pinned={file.pinned}
    title={file.pinned ? "Unpin" : "Pin for quick access"}
    tabindex="-1"
    onclick={(e) => {
      e.stopPropagation();
      void library.togglePin(file);
    }}
  >
    <Icon name="pin" size={13} filled={file.pinned} />
  </button>
</div>

<style>
  .item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 4px 5px 8px;
    border-radius: var(--radius);
    outline: none;
  }

  .item:hover {
    background: var(--hover);
  }

  .item:focus-visible {
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

  .pin {
    width: 24px;
    height: 24px;
    visibility: hidden;
  }

  .item:hover .pin,
  .pin.pinned {
    visibility: visible;
  }

  .pin.pinned {
    color: var(--accent);
  }
</style>

<script lang="ts">
  import Icon from "../common/Icon.svelte";
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
</style>

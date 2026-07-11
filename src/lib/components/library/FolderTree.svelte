<script lang="ts">
  import FolderTree from "./FolderTree.svelte";
  import Icon from "../common/Icon.svelte";
  import FileItem from "./FileItem.svelte";
  import { library, type DirNode } from "../../stores/library.svelte";

  let { node, depth = 0 }: { node: DirNode; depth?: number } = $props();
</script>

{#each node.dirs as dir (dir.relPath)}
  {@const open = library.expanded.has(dir.relPath)}
  <div
    class="folder"
    style="padding-left: {8 + depth * 14}px"
    role="button"
    tabindex="0"
    onclick={() => library.toggleDir(dir.relPath)}
    onkeydown={(e) => {
      if (e.key === "Enter") library.toggleDir(dir.relPath);
    }}
  >
    <span class="folder-icon"><Icon name="folder" size={15} filled /></span>
    <span class="fname">{dir.name}</span>
  </div>
  {#if open}
    <FolderTree node={dir} depth={depth + 1} />
  {/if}
{/each}

{#each node.files as file (file.path)}
  <FileItem {file} depth={depth} showDir={false} />
{/each}

<style>
  .folder {
    display: flex;
    align-items: center;
    gap: 8px; /* matches FileItem so folder and file labels align */
    min-height: 34px; /* matches FileItem row height */
    padding-top: 5px;
    padding-bottom: 5px;
    padding-right: 8px;
    border-radius: var(--radius);
    outline: none;
  }

  .folder:hover {
    background: var(--hover);
  }

  .folder:focus-visible {
    box-shadow: 0 0 0 2px var(--accent);
  }

  .folder-icon {
    display: flex;
    color: var(--folder-icon);
    flex-shrink: 0;
  }

  .fname {
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: 500;
  }
</style>

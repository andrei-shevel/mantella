<script lang="ts">
  import Icon from "../common/Icon.svelte";
  import { reader } from "../../stores/reader.svelte";
  import { ui } from "../../stores/ui.svelte";
  import type { Bookmark } from "../../api/types";

  let { bookmark, onactivate }: { bookmark: Bookmark; onactivate: () => void } =
    $props();

  let editing = $state(false);
  let draft = $state("");
  let inputEl = $state<HTMLInputElement>();

  function startEditing() {
    draft = bookmark.title;
    editing = true;
  }

  $effect(() => {
    if (editing && inputEl) {
      inputEl.focus();
      inputEl.select();
    }
  });

  function commit() {
    if (!editing) return;
    editing = false;
    reader.renameBookmark(bookmark.id, draft);
  }

  function cancel() {
    editing = false;
  }
</script>

<div
  class="item"
  role="button"
  tabindex="0"
  title={bookmark.title}
  onclick={() => {
    if (!editing) onactivate();
  }}
  ondblclick={startEditing}
  onkeydown={(e) => {
    if (e.key === "Enter" && !editing) onactivate();
  }}
  oncontextmenu={(e) =>
    ui.openContextMenu(e, [
      { label: "Rename", action: startEditing },
      {
        label: "Remove",
        danger: true,
        action: () => reader.removeBookmark(bookmark.id),
      },
    ])}
>
  <span class="mark-icon"><Icon name="bookmark" size={13} /></span>
  {#if editing}
    <input
      bind:this={inputEl}
      bind:value={draft}
      spellcheck="false"
      autocomplete="off"
      aria-label="Bookmark title"
      onkeydown={(e) => {
        if (e.key === "Enter") commit();
        else if (e.key === "Escape") cancel();
      }}
      onblur={commit}
      onclick={(e) => e.stopPropagation()}
      ondblclick={(e) => e.stopPropagation()}
    />
  {:else}
    <span class="title">{bookmark.title}</span>
    <span class="page">p. {bookmark.page}</span>
  {/if}
</div>

<style>
  .item {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 30px;
    padding: 4px 8px;
    border-radius: var(--radius);
    outline: none;
  }

  .item:hover {
    background: var(--hover);
  }

  .item:focus-visible {
    box-shadow: 0 0 0 2px var(--accent);
  }

  .mark-icon {
    display: flex;
    color: var(--text-3);
    flex-shrink: 0;
  }

  .title {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .page {
    flex-shrink: 0;
    font-size: 11px;
    color: var(--text-3);
    font-variant-numeric: tabular-nums;
  }

  input {
    flex: 1;
    min-width: 0;
    height: 22px;
    padding: 0 6px;
    border: none;
    border-radius: 5px;
    background: var(--bg-input);
    outline: none;
    user-select: text;
  }

  input:focus {
    box-shadow: 0 0 0 1.5px var(--accent);
  }
</style>

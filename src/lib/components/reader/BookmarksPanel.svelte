<script lang="ts">
  import Icon from "../common/Icon.svelte";
  import BookmarkItem from "./BookmarkItem.svelte";
  import { reader } from "../../stores/reader.svelte";
  import { history } from "../../stores/history.svelte";

  let listEl = $state<HTMLDivElement>();

  function onListKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    if (target.tagName === "INPUT" || target.tagName === "TEXTAREA") return;
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        reader.moveBookmarkCursor(1);
        break;
      case "ArrowUp":
        e.preventDefault();
        reader.moveBookmarkCursor(-1);
        break;
      case "Enter":
      case " ":
        e.preventDefault();
        reader.activateBookmarkCursor();
        break;
      case "Home":
        e.preventDefault();
        reader.moveBookmarkCursorToEdge(true);
        break;
      case "End":
        e.preventDefault();
        reader.moveBookmarkCursorToEdge(false);
        break;
      case "Escape":
        e.preventDefault();
        reader.focusViewer();
        break;
    }
  }

  $effect(() => {
    const id = reader.bookmarkCursor;
    if (!id || !listEl) return;
    listEl
      .querySelector(`#bookmark-${CSS.escape(id)}`)
      ?.scrollIntoView({ block: "nearest" });
  });

  $effect(() => {
    reader.bookmarksListEl = listEl ?? null;
  });
</script>

<aside>
  <div class="head">
    <span class="section">Bookmarks</span>
    <button
      class="icon-btn"
      title="Add bookmark at current position"
      onclick={() => reader.addBookmark()}
    >
      <Icon name="bookmark-plus" size={15} />
    </button>
  </div>

  <div
    class="list"
    role="listbox"
    tabindex="0"
    aria-activedescendant={reader.bookmarkCursor
      ? `bookmark-${reader.bookmarkCursor}`
      : undefined}
    bind:this={listEl}
    onkeydown={onListKeydown}
    onclick={() => listEl?.focus()}
    onfocus={() => {
      reader.bookmarksListFocused = true;
      reader.ensureBookmarkCursor();
    }}
    onblur={() => (reader.bookmarksListFocused = false)}
  >
    {#each reader.bookmarksSorted as bm (bm.id)}
      <BookmarkItem
        bookmark={bm}
        onactivate={() =>
          history.navigate({ page: bm.page, offset: bm.pageOffset })}
      />
    {/each}
    {#if reader.bookmarks.length === 0}
      <div class="none">No bookmarks yet</div>
    {/if}
  </div>
</aside>

<style>
  aside {
    display: flex;
    flex-direction: column;
    width: 232px;
    flex-shrink: 0;
    background: var(--bg-sidebar);
    border-left: 1px solid var(--border);
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
    padding: 8px 8px 2px 16px;
  }

  .section {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-3);
  }

  .list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 8px 8px;
    outline: none;
  }

  .none {
    color: var(--text-3);
    text-align: center;
    padding: 28px 12px;
  }
</style>

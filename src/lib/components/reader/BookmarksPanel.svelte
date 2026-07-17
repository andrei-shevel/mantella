<script lang="ts">
  import Icon from "../common/Icon.svelte";
  import BookmarkItem from "./BookmarkItem.svelte";
  import { reader } from "../../stores/reader.svelte";
  import { history } from "../../stores/history.svelte";
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

  <div class="list">
    {#each reader.bookmarksSorted as bm (bm.id)}
      <BookmarkItem
        bookmark={bm}
        onactivate={() => history.navigate({ page: bm.page, offset: bm.pageOffset })}
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
  }

  .none {
    color: var(--text-3);
    text-align: center;
    padding: 28px 12px;
  }
</style>

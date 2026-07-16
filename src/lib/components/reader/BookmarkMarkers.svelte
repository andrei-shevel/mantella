<script lang="ts">
  // Bookmark pointers pinned to the right edge of the document area.
  // Vertically they track their anchor in canvas space, so they scroll with
  // the content; dragging one up/down moves the bookmark's anchor.
  import Icon from "../common/Icon.svelte";
  import { reader } from "../../stores/reader.svelte";
  import { ui } from "../../stores/ui.svelte";
  import type { Bookmark } from "../../api/types";

  let {
    layout,
    scrollTop,
    viewportHeight,
    gap,
    scrollToAnchor,
  }: {
    layout: { items: { top: number; width: number; height: number }[]; totalHeight: number };
    scrollTop: number;
    viewportHeight: number;
    gap: number;
    scrollToAnchor: (anchor: { page: number; offset: number }) => void;
  } = $props();

  const DRAG_THRESHOLD = 3;

  let railEl = $state<HTMLDivElement>();
  let drag = $state<{ id: string; y: number; moved: boolean; startClientY: number } | null>(null);

  function canvasY(bm: Bookmark): number {
    const items = layout.items;
    const page = Math.min(Math.max(bm.page, 1), items.length);
    const item = items[page - 1];
    return item.top + bm.pageOffset * item.height;
  }

  /** Inverse of canvasY: which page band a canvas Y lands in, clamped to the document. */
  function anchorAtCanvasY(y: number): { page: number; offset: number } {
    const items = layout.items;
    const last = items[items.length - 1];
    const clamped = Math.max(items[0].top, Math.min(y, last.top + last.height));
    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      if (clamped < item.top + item.height + gap / 2 || i === items.length - 1) {
        return {
          page: i + 1,
          offset: Math.max(0, Math.min(1, (clamped - item.top) / item.height)),
        };
      }
    }
    return { page: items.length, offset: 1 };
  }

  function onPointerDown(e: PointerEvent, bm: Bookmark) {
    if (e.button !== 0) return;
    // keep the drag from starting a text selection in the page underneath
    e.preventDefault();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    drag = { id: bm.id, y: canvasY(bm), moved: false, startClientY: e.clientY };
  }

  function onPointerMove(e: PointerEvent, bm: Bookmark) {
    if (!drag || drag.id !== bm.id || !railEl) return;
    if (!drag.moved && Math.abs(e.clientY - drag.startClientY) < DRAG_THRESHOLD) return;
    drag.moved = true;
    const y = e.clientY - railEl.getBoundingClientRect().top + scrollTop;
    const anchor = anchorAtCanvasY(y);
    const item = layout.items[anchor.page - 1];
    drag.y = item.top + anchor.offset * item.height;
  }

  function onPointerUp(bm: Bookmark) {
    if (!drag || drag.id !== bm.id) return;
    const { moved, y } = drag;
    drag = null;
    if (moved) {
      const anchor = anchorAtCanvasY(y);
      reader.moveBookmark(bm.id, anchor.page, anchor.offset);
    } else {
      scrollToAnchor({ page: bm.page, offset: bm.pageOffset });
    }
  }
</script>

<div class="rail" bind:this={railEl}>
  {#if layout.items.length > 0}
    {#each reader.bookmarks as bm (bm.id)}
      {@const dragging = drag?.id === bm.id && drag.moved}
      {@const viewY = (dragging && drag ? drag.y : canvasY(bm)) - scrollTop}
      {#if viewY > -16 && viewY < viewportHeight + 16}
        <div
          class="marker"
          class:dragging
          style="translate: 0 {viewY - 9}px"
          title={bm.title}
          role="button"
          tabindex="-1"
          aria-label={bm.title}
          onpointerdown={(e) => onPointerDown(e, bm)}
          onpointermove={(e) => onPointerMove(e, bm)}
          onpointerup={() => onPointerUp(bm)}
          onpointercancel={() => (drag = null)}
          oncontextmenu={(e) =>
            ui.openContextMenu(e, [
              { label: "Remove", danger: true, action: () => reader.removeBookmark(bm.id) },
            ])}
        >
          <Icon name="bookmark" size={14} filled />
        </div>
      {/if}
    {/each}
  {/if}
</div>

<style>
  .rail {
    position: absolute;
    top: 0;
    bottom: 0;
    right: 10px;
    width: 24px;
    pointer-events: none;
    z-index: 3;
    overflow: hidden;
  }

  .marker {
    position: absolute;
    top: 0;
    right: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 20px;
    pointer-events: auto;
    touch-action: none;
    cursor: grab;
    color: var(--accent);
    filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.25));
    transition: scale 0.12s ease;
  }

  .marker:hover {
    scale: 1.15;
  }

  .marker.dragging {
    cursor: grabbing;
    scale: 1.15;
  }
</style>

<script lang="ts">
  import { tick } from "svelte";
  import Page from "./Page.svelte";
  import Toolbar from "./Toolbar.svelte";
  import BookmarkMarkers from "./BookmarkMarkers.svelte";
  import BookmarksPanel from "./BookmarksPanel.svelte";
  import EmptyState from "../common/EmptyState.svelte";
  import { reader } from "../../stores/reader.svelte";
  import { ui } from "../../stores/ui.svelte";
  import { history } from "../../stores/history.svelte";

  const PT_TO_PX = 96 / 72; // 100% zoom = 96dpi CSS pixels for 72dpi PDF points
  const PADDING = 28;
  const GAP = 20;
  const MIN_ZOOM = 0.25;
  const MAX_ZOOM = 6;
  const VISIBLE_BUFFER = 700; // px of pages kept rendered beyond the viewport

  const dpr = window.devicePixelRatio || 1;

  // Mantella head (from mantella.png), peeking out of the corner.
  const MASCOT = `                                    ....
                                .-=======--:.
                              :===============-::.....
                           :-===========================--.
                      ..:-==================================:
                 .::-===========+*#%%%##+==============++====
        ...::--================*%@@#:..+@%+===========#%%+==-
.::---========================+@@@@#.  =@@%===============+=
=====================++++=====+@@@@@%%%@@@%============+*#+.
===========+++**##%%%@@@@@*====#@@@@@@@@@%+========+*#%@%-
++**###%%%%@@@@@@@@@@@@@@@#=====+#%%%%%#*======+*#%@@@#-
@@@@@@@@@@@@@@@@@@@@@@@@@@%=================*#%@@@@%*:
@@@@@@@@@@@%%%@@@@@@@@@@@@%=============+*%%@@@@@*-.
@@@@@%%#*++===+@@@@@@@@@@@%==========+#%@@@@@@#=.
@%#*+=========*@@@@@@@@@@@@#======+#%@@@@@@%+:
+===========*%@@@@@@@@@@@@@@%#*##%@@@@@@@#=.
=========+#%@@@@@@@@@@@@@@@@@@@@@@@@@@@#-
=====+*#%@@@@@@@@@@@%###%@@@@@@@@@@@@#-
==+*#%@@@@@@@@@@@@#+====+@@@@@@@@@@%=`;

  let container = $state<HTMLDivElement>();
  let containerWidth = $state(0);
  let containerHeight = $state(0);
  let scrollTop = $state(0);

  let maxPageWidth = $derived(
    reader.pages.reduce((m, p) => Math.max(m, p.width), 0) || 612,
  );

  /** CSS pixels per PDF point at the current zoom (fit-width when zoom is null). */
  let scale = $derived(
    reader.zoom === null
      ? containerWidth > 0
        ? Math.max(0.05, (containerWidth - PADDING * 2) / maxPageWidth)
        : PT_TO_PX
      : PT_TO_PX * reader.zoom,
  );

  // Bitmaps re-render at a debounced scale, so rapid zooming only CSS-scales
  // the already-loaded images until the gesture settles.
  let renderScale = $state(0);
  $effect(() => {
    const target = scale;
    if (containerWidth === 0) return;
    if (renderScale === 0 || renderScale === target) {
      renderScale = target;
      return;
    }
    const timer = setTimeout(() => (renderScale = target), 300);
    return () => clearTimeout(timer);
  });

  let layout = $derived.by(() => {
    let y = PADDING;
    const items = reader.pages.map((p) => {
      const item = { top: y, width: p.width * scale, height: p.height * scale };
      y += item.height + GAP;
      return item;
    });
    return { items, totalHeight: items.length > 0 ? y - GAP + PADDING : 0 };
  });

  // Widest page + padding: sizing the canvas to the content keeps pages
  // inside it, so zoomed-in pages can scroll all the way left (overflow left
  // of a scroll container's origin is unreachable).
  let contentWidth = $derived(maxPageWidth * scale + PADDING * 2);

  let visible = $derived.by(() => {
    const from = scrollTop - VISIBLE_BUFFER;
    const to = scrollTop + containerHeight + VISIBLE_BUFFER;
    const indices: number[] = [];
    for (let i = 0; i < layout.items.length; i++) {
      const item = layout.items[i];
      if (item.top >= to) break;
      if (item.top + item.height > from) indices.push(i);
    }
    return indices;
  });

  function currentPageAt(top: number): number {
    const anchor = top + containerHeight * 0.35;
    for (let i = 0; i < layout.items.length; i++) {
      const item = layout.items[i];
      if (anchor < item.top + item.height + GAP) return i + 1;
    }
    return layout.items.length || 1;
  }

  /**
   * The reading position as page + offset within that page (fraction of page
   * height). Layout-independent: survives window resizes and zoom changes.
   */
  function pageAnchorAt(top: number): { page: number; offset: number } {
    const page = currentPageAt(top);
    const item = layout.items[page - 1];
    if (!item) return { page: 1, offset: 0 };
    return { page, offset: (top - item.top) / item.height };
  }

  function scrollToAnchor(anchor: { page: number; offset: number }) {
    if (!container || layout.items.length === 0) return;
    const page = Math.min(Math.max(anchor.page, 1), layout.items.length);
    const item = layout.items[page - 1];
    container.scrollTop = Math.max(0, item.top + anchor.offset * item.height);
    scrollTop = container.scrollTop;
  }

  // Last position the user actually scrolled to, in layout-independent form.
  // Used to hold the position steady when the layout rescales under us.
  let lastAnchor = { page: 1, offset: 0 };

  function onScroll() {
    if (!container) return;
    scrollTop = container.scrollTop;
    // don't clobber the saved position while the restore is still pending
    if (reader.docId !== null && reader.pendingRestore === null) {
      lastAnchor = pageAnchorAt(scrollTop);
      reader.reportScroll(lastAnchor.page, lastAnchor.offset);
    }
  }

  // Restore the saved position once a freshly opened document has laid out.
  $effect(() => {
    if (reader.docId === null || !container) return;
    const restore = reader.pendingRestore;
    if (restore === null) return;
    void tick().then(() => {
      scrollToAnchor(restore);
      lastAnchor = restore;
      reader.pendingRestore = null;
    });
  });

  // Same-document jumps requested via the store (bookmarks, history).
  $effect(() => {
    const jump = reader.pendingJump;
    if (jump === null || reader.docId === null || !container) return;
    scrollToAnchor(jump);
    lastAnchor = jump;
    reader.pendingJump = null;
  });

  // Keep the reading position when the viewer is resized (fullscreen, window
  // resize, sidebar toggle): the fit-width layout rescales but scrollTop is a
  // pixel value, so re-derive it from the anchor.
  let prevWidth = 0;
  $effect(() => {
    const width = containerWidth;
    if (width === prevWidth) return;
    const isFirstMeasure = prevWidth === 0;
    prevWidth = width;
    if (
      isFirstMeasure ||
      reader.docId === null ||
      reader.pendingRestore !== null
    )
      return;
    const anchor = lastAnchor;
    void tick().then(() => scrollToAnchor(anchor));
  });

  function setZoomPreservingPosition(zoom: number | null) {
    const anchor = container ? pageAnchorAt(container.scrollTop) : null;
    reader.setZoom(zoom);
    void tick().then(() => {
      if (anchor) scrollToAnchor(anchor);
    });
  }

  function zoomBy(factor: number) {
    const current = reader.zoom ?? scale / PT_TO_PX;
    setZoomPreservingPosition(
      Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, current * factor)),
    );
  }

  function scrollToPage(page: number) {
    if (!container || layout.items.length === 0) return;
    const clamped = Math.min(Math.max(page, 1), layout.items.length);
    container.scrollTop = Math.max(0, layout.items[clamped - 1].top - 12);
  }

  // Internal PDF links are navigation jumps: they go into the history
  // (unlike toolbar/keyboard paging, which behaves like scrolling). The
  // anchor reproduces scrollToPage's landing (12px above the page top).
  function followLink(page: number) {
    const clamped = Math.min(Math.max(page, 1), layout.items.length);
    const item = layout.items[clamped - 1];
    if (!item) return;
    history.navigate({ page: clamped, offset: -12 / item.height });
  }

  // ctrl/cmd + wheel zoom needs a non-passive listener to preventDefault
  $effect(() => {
    const el = container;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      if (!(e.ctrlKey || e.metaKey)) return;
      e.preventDefault();
      if (reader.docId !== null) zoomBy(Math.exp(-e.deltaY * 0.0022));
    };
    el.addEventListener("wheel", onWheel, { passive: false });
    return () => el.removeEventListener("wheel", onWheel);
  });

  function onKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    if (target.tagName === "INPUT" || target.tagName === "TEXTAREA") return;
    if (reader.docId === null) return;
    const meta = e.metaKey || e.ctrlKey;

    if (meta && (e.key === "=" || e.key === "+")) {
      e.preventDefault();
      zoomBy(1.15);
    } else if (meta && e.key === "-") {
      e.preventDefault();
      zoomBy(1 / 1.15);
    } else if (meta && e.key === "0") {
      e.preventDefault();
      setZoomPreservingPosition(null);
    } else if (e.key === "ArrowRight" || e.key === "PageDown") {
      e.preventDefault();
      scrollToPage(reader.currentPage + 1);
    } else if (e.key === "ArrowLeft" || e.key === "PageUp") {
      e.preventDefault();
      scrollToPage(reader.currentPage - 1);
    } else if (e.key === "Home") {
      e.preventDefault();
      scrollToPage(1);
    } else if (e.key === "End") {
      e.preventDefault();
      scrollToPage(layout.items.length);
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="viewer">
  <Toolbar
    {zoomBy}
    setZoom={setZoomPreservingPosition}
    {scrollToPage}
    zoomPercent={Math.round((scale / PT_TO_PX) * 100)}
    pageCount={layout.items.length}
  />

  <div class="body">
    <div class="doc-area">
      <div
        class="scroll"
        bind:this={container}
        bind:clientWidth={containerWidth}
        bind:clientHeight={containerHeight}
        onscroll={onScroll}
      >
        <div
          class="canvas"
          style="height: {layout.totalHeight}px; width: {contentWidth}px"
        >
          {#if reader.docId !== null}
            {#each visible as i (i)}
              <Page
                docId={reader.docId}
                index={i}
                top={layout.items[i].top}
                width={layout.items[i].width}
                height={layout.items[i].height}
                renderWidth={reader.pages[i].width *
                  (renderScale || scale) *
                  dpr}
                pointWidth={reader.pages[i].width}
                pointHeight={reader.pages[i].height}
                goToPage={followLink}
              />
            {/each}
          {/if}
        </div>
      </div>

      {#if reader.docId !== null && reader.bookmarks.length > 0}
        <BookmarkMarkers
          {layout}
          {scrollTop}
          viewportHeight={containerHeight}
          gap={GAP}
        />
      {/if}
    </div>

    {#if ui.bookmarksPanelOpen && reader.docId !== null}
      <BookmarksPanel />
    {/if}
  </div>

  {#if reader.docId === null}
    <div class="overlay">
      {#if reader.loading}
        <div class="spinner" aria-label="Loading"></div>
      {:else if reader.error}
        <EmptyState
          icon="file"
          title="Couldn't open this file"
          subtitle={reader.error}
        />
      {:else}
        <EmptyState
          icon="file"
          title="No document open"
          subtitle="Select a PDF from the sidebar, or press ⌘F to search your library."
        />
        <pre class="mascot" aria-hidden="true">{MASCOT}</pre>
      {/if}
    </div>
  {/if}
</div>

<style>
  .viewer {
    position: relative;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg-main);
  }

  .body {
    flex: 1;
    min-height: 0;
    display: flex;
  }

  .doc-area {
    position: relative;
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .scroll {
    flex: 1;
    overflow: auto;
    min-height: 0;
  }

  .canvas {
    position: relative;
    min-width: 100%;
  }

  .overlay {
    position: absolute;
    inset: 44px 0 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-main);
  }

  .mascot {
    position: absolute;
    left: 0;
    bottom: 0;
    margin: 0;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 13px;
    line-height: 1.1;
    color: var(--text-3);
    opacity: 0.55;
    user-select: none;
    pointer-events: none;
  }

  .spinner {
    width: 26px;
    height: 26px;
    border: 2.5px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>

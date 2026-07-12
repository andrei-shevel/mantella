<script lang="ts" module>
  import type { TextRun } from "../../api/types";
  import { withHitPadding, type LayerRun } from "./textLayer";

  // Text runs are zoom-independent (PDF points), so each page is fetched once
  // per document and kept across mount/unmount as the page scrolls in and out.
  const textCache = new Map<string, Promise<LayerRun[]>>();
  const TEXT_CACHE_MAX = 400;

  function cachedPageText(
    docId: number,
    index: number,
    pageWidth: number,
    pageHeight: number,
  ): Promise<LayerRun[]> {
    const key = `${docId}:${index}`;
    let promise = textCache.get(key);
    if (!promise) {
      promise = getPageText(docId, index)
        .then((runs) => withHitPadding(runs, pageWidth, pageHeight))
        .catch(() => []);
      if (textCache.size >= TEXT_CACHE_MAX) {
        const oldest = textCache.keys().next().value;
        if (oldest !== undefined) textCache.delete(oldest);
      }
      textCache.set(key, promise);
    }
    return promise;
  }

  // The layer's font never renders (the text is transparent), so glyph widths
  // can be measured off-DOM with a canvas and scale linearly with font size.
  const measurer = document.createElement("canvas").getContext("2d")!;

  function textScaleX(run: TextRun, fontPx: number, targetPx: number): number {
    measurer.font = `${fontPx}px sans-serif`;
    const natural = measurer.measureText(run.text).width;
    return natural > 0 ? targetPx / natural : 1;
  }

  function runStyle(run: LayerRun, scale: number): string {
    const fontPx = run.height * scale;
    const scaleX = textScaleX(run, fontPx, run.width * scale);
    // scaleX stretches the box horizontally, so horizontal padding is given in
    // pre-transform units to land at its target size; vertical is unaffected.
    return (
      `left: ${(run.x - run.padLeft) * scale}px; ` +
      `top: ${(run.y - run.padTop) * scale}px; ` +
      `padding: ${run.padTop * scale}px ${(run.padRight * scale) / scaleX}px ` +
      `${run.padBottom * scale}px ${(run.padLeft * scale) / scaleX}px; ` +
      `font-size: ${fontPx}px; transform: scaleX(${scaleX})`
    );
  }
</script>

<script lang="ts">
  import { pageUrl } from "../../api/protocol";
  import { getPageText } from "../../api/commands";

  let {
    docId,
    index,
    top,
    width,
    height,
    renderWidth,
    pointWidth,
    pointHeight,
  }: {
    docId: number;
    index: number;
    top: number;
    width: number;
    height: number;
    renderWidth: number;
    pointWidth: number;
    pointHeight: number;
  } = $props();

  // Preload the bitmap off-screen and only swap the visible <img> once it has
  // decoded, so zoom/re-render never flashes a blank page.
  let displayedSrc = $state<string | null>(null);

  $effect(() => {
    const url = pageUrl(docId, index, renderWidth);
    if (url === displayedSrc) return;
    let cancelled = false;
    const img = new Image();
    img.onload = () => {
      if (!cancelled) displayedSrc = url;
    };
    img.src = url;
    return () => {
      cancelled = true;
    };
  });

  let runs = $state<LayerRun[]>([]);

  $effect(() => {
    const id = docId;
    const page = index;
    let cancelled = false;
    void cachedPageText(id, page, pointWidth, pointHeight).then((result) => {
      if (!cancelled) runs = result;
    });
    return () => {
      cancelled = true;
    };
  });

  /** CSS pixels per PDF point at the current layout size. */
  let scale = $derived(width / pointWidth);
</script>

<div class="page" style="top: {top}px; width: {width}px; height: {height}px">
  {#if displayedSrc}
    <img src={displayedSrc} alt="Page {index + 1}" draggable="false" />
  {/if}
  <div class="text-layer">
    {#each runs as run}
      <span style={runStyle(run, scale)}>{run.text}</span>
    {/each}
  </div>
</div>

<style>
  .page {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    background: #fff;
    border-radius: 2px;
    box-shadow: var(--page-shadow);
    overflow: hidden;
  }

  img {
    display: block;
    width: 100%;
    height: 100%;
    -webkit-user-drag: none;
  }

  .text-layer {
    position: absolute;
    inset: 0;
    user-select: text;
  }

  .text-layer span {
    position: absolute;
    color: transparent;
    white-space: pre;
    line-height: 1;
    font-family: sans-serif;
    transform-origin: 0 0;
    cursor: text;
  }

  .text-layer span::selection {
    background: color-mix(in srgb, var(--accent) 32%, transparent);
  }
</style>

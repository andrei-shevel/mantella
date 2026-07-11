<script lang="ts">
  import { pageUrl } from "../../api/protocol";

  let {
    docId,
    index,
    top,
    width,
    height,
    renderWidth,
  }: {
    docId: number;
    index: number;
    top: number;
    width: number;
    height: number;
    renderWidth: number;
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
</script>

<div class="page" style="top: {top}px; width: {width}px; height: {height}px">
  {#if displayedSrc}
    <img src={displayedSrc} alt="Page {index + 1}" draggable="false" />
  {/if}
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
</style>

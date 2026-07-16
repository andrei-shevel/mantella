<script lang="ts">
  // App-wide singleton context menu, driven by ui.openContextMenu(event, items).
  import { ui } from "../../stores/ui.svelte";

  let menuEl = $state<HTMLDivElement>();
  let menuWidth = $state(0);
  let menuHeight = $state(0);

  // Keep the menu inside the window near the edges.
  let pos = $derived.by(() => {
    const menu = ui.contextMenu;
    if (!menu) return { left: 0, top: 0 };
    return {
      left: Math.max(4, Math.min(menu.x, window.innerWidth - menuWidth - 4)),
      top: Math.max(4, Math.min(menu.y, window.innerHeight - menuHeight - 4)),
    };
  });

  function onPointerDown(e: PointerEvent) {
    if (ui.contextMenu && menuEl && !menuEl.contains(e.target as Node)) {
      ui.closeContextMenu();
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") ui.closeContextMenu();
  }
</script>

<svelte:window
  onpointerdowncapture={onPointerDown}
  onkeydown={onKeydown}
  onwheel={() => ui.closeContextMenu()}
  onresize={() => ui.closeContextMenu()}
  onblur={() => ui.closeContextMenu()}
/>

{#if ui.contextMenu}
  <div
    class="menu"
    role="menu"
    tabindex="-1"
    bind:this={menuEl}
    bind:clientWidth={menuWidth}
    bind:clientHeight={menuHeight}
    style="left: {pos.left}px; top: {pos.top}px"
    oncontextmenu={(e) => e.preventDefault()}
  >
    {#each ui.contextMenu.items as item (item.label)}
      <button
        class="entry"
        class:danger={item.danger}
        role="menuitem"
        onclick={() => {
          item.action();
          ui.closeContextMenu();
        }}
      >
        {item.label}
      </button>
    {/each}
  </div>
{/if}

<style>
  .menu {
    position: fixed;
    z-index: 100;
    min-width: 150px;
    padding: 4px;
    background: var(--bg-raised);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--page-shadow);
    outline: none;
  }

  .entry {
    display: block;
    width: 100%;
    height: 26px;
    padding: 0 10px;
    border-radius: 5px;
    text-align: left;
    color: var(--text-1);
  }

  .entry:hover {
    background: var(--hover);
  }

  .entry.danger {
    color: #e5484d;
  }

  .entry.danger:hover {
    background: rgba(229, 72, 77, 0.1);
  }
</style>

<script lang="ts">
  import Icon from "./Icon.svelte";

  let {
    icon,
    title,
    subtitle = "",
  }: { icon: string; title: string; subtitle?: string } = $props();
</script>

<div class="empty">
  <div class="glyph"><Icon name={icon} size={44} /></div>
  <h2>{title}</h2>
  {#if subtitle}<p>{subtitle}</p>{/if}
</div>

<style>
  .empty {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    color: var(--text-3);
  }

  .glyph {
    margin-bottom: 8px;
    opacity: 0.7;
  }

  h2 {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-2);
  }

  p {
    font-size: 13px;
    max-width: 300px;
    text-align: center;
    line-height: 1.45;
  }
</style>

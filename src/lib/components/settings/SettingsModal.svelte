<script lang="ts">
  // App-wide singleton settings dialog, driven by ui.settingsOpen.
  import Icon from "../common/Icon.svelte";
  import { ui } from "../../stores/ui.svelte";
  import { shortcuts } from "../../stores/shortcuts.svelte";
  import {
    SHORTCUTS,
    FIXED_SHORTCUTS,
    formatBinding,
    type ShortcutCategory,
  } from "../../shortcuts";

  const CATEGORIES: ShortcutCategory[] = [
    "Navigation",
    "Zoom",
    "Panels",
    "General",
  ];

  const IGNORED_KEYS = new Set(["Meta", "Control", "Shift", "Alt", "CapsLock"]);

  let modalEl = $state<HTMLDivElement>();
  let recordingId = $state<string | null>(null);
  let captureError = $state("");

  function close() {
    ui.closeSettings();
    recordingId = null;
    captureError = "";
  }

  function startRecording(id: string) {
    recordingId = id;
    captureError = "";
  }

  function stopRecording() {
    recordingId = null;
    captureError = "";
  }

  function conflictLabel(id: string): string {
    if (id === "pinned-files") return "Open pinned file";
    return SHORTCUTS.find((s) => s.id === id)?.label ?? id;
  }

  function onWindowKeydown(e: KeyboardEvent) {
    if (!ui.settingsOpen) return;

    if (recordingId) {
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") {
        stopRecording();
        return;
      }
      if (IGNORED_KEYS.has(e.key)) return;
      const meta = e.metaKey || e.ctrlKey;
      if (!meta) {
        captureError = "Hold ⌘ (or Ctrl) and press a key";
        return;
      }
      const binding = { key: e.key, meta: true };
      const conflictId = shortcuts.findConflict(binding, recordingId);
      if (conflictId) {
        captureError = `Already used by "${conflictLabel(conflictId)}"`;
        return;
      }
      shortcuts.set(recordingId, binding);
      stopRecording();
      return;
    }

    if (e.key === "Escape") close();
  }

  function onOutsidePointerDown(e: PointerEvent) {
    if (!ui.settingsOpen) return;
    if (modalEl && !modalEl.contains(e.target as Node)) {
      if (recordingId) stopRecording();
      else close();
    }
  }
</script>

<svelte:window
  onkeydown={onWindowKeydown}
  onpointerdowncapture={onOutsidePointerDown}
/>

{#if ui.settingsOpen}
  <div class="backdrop">
    <div class="modal" role="dialog" aria-label="Settings" bind:this={modalEl}>
      <header>
        <h2>Settings</h2>
        <button class="icon-btn" title="Close" onclick={close}>
          <Icon name="x" />
        </button>
      </header>

      <div class="body">
        <h3>Keyboard shortcuts</h3>
        {#each CATEGORIES as category (category)}
          {@const defs = SHORTCUTS.filter((s) => s.category === category)}
          {@const fixed = FIXED_SHORTCUTS.filter(
            (s) => s.category === category,
          )}
          <div class="section">{category}</div>
          {#each defs as def (def.id)}
            <div class="row">
              <span class="label">{def.label}</span>
              <div class="control">
                {#if recordingId === def.id}
                  <span class="recording" class:error={captureError}>
                    {captureError || "Press a key…"}
                  </span>
                {:else}
                  <button
                    class="binding"
                    onclick={() => startRecording(def.id)}
                  >
                    {formatBinding(shortcuts.get(def.id))}
                  </button>
                  {#if shortcuts.isCustom(def.id)}
                    <button
                      class="icon-btn"
                      title="Reset to default"
                      onclick={() => shortcuts.reset(def.id)}
                    >
                      <Icon name="rotate-ccw" size={13} />
                    </button>
                  {/if}
                {/if}
              </div>
            </div>
          {/each}
          {#each fixed as f (f.label)}
            <div class="row fixed">
              <span class="label">{f.label}</span>
              <span class="binding fixed">{f.keys}</span>
            </div>
          {/each}
        {/each}
      </div>

      <footer>
        <button class="text-btn" onclick={() => shortcuts.resetAll()}>
          Reset all to defaults
        </button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .modal {
    display: flex;
    flex-direction: column;
    width: 460px;
    max-height: 80vh;
    background: var(--bg-raised);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: var(--page-shadow);
    overflow: hidden;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  header h2 {
    font-size: 15px;
    font-weight: 600;
  }

  .body {
    flex: 1;
    overflow-y: auto;
    padding: 4px 16px 16px;
  }

  .body h3 {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-2);
    padding: 12px 0 6px;
  }

  .section {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-3);
    padding: 12px 0 4px;
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    min-height: 30px;
  }

  .row.fixed {
    opacity: 0.7;
  }

  .label {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .control {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .binding {
    min-width: 64px;
    height: 24px;
    padding: 0 8px;
    border-radius: 6px;
    background: var(--bg-input);
    color: var(--text-1);
    font-variant-numeric: tabular-nums;
    text-align: center;
  }

  .binding:hover {
    background: var(--hover);
  }

  .binding.fixed {
    color: var(--text-3);
  }

  .recording {
    min-width: 64px;
    height: 24px;
    padding: 0 8px;
    border-radius: 6px;
    background: var(--accent-soft);
    color: var(--accent-strong);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    white-space: nowrap;
  }

  .recording.error {
    background: rgba(229, 72, 77, 0.12);
    color: #e5484d;
  }

  footer {
    flex-shrink: 0;
    padding: 10px 16px;
    border-top: 1px solid var(--border);
    display: flex;
    justify-content: flex-end;
  }

  .text-btn {
    padding: 6px 10px;
    border-radius: 6px;
    color: var(--text-2);
    font-size: 12px;
  }

  .text-btn:hover {
    background: var(--hover);
    color: var(--text-1);
  }
</style>

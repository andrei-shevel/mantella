<script lang="ts">
  import { updater } from "../../stores/updater.svelte";
</script>

{#if updater.promptOpen && updater.status === "available"}
  <div class="backdrop">
    <div class="modal" role="dialog" aria-label="Update available">
      <header>
        <h2>Update available</h2>
      </header>
      <div class="body">
        <p>
          Mantella {updater.availableVersion} is ready to install
          {#if updater.currentVersion}
            <span class="muted">(you have {updater.currentVersion})</span>
          {/if}.
        </p>
        {#if updater.notes}
          <pre class="notes">{updater.notes}</pre>
        {/if}
      </div>
      <footer>
        <button class="text-btn" onclick={() => updater.dismissPrompt()}>
          Later
        </button>
        <button
          class="primary"
          disabled={updater.busy}
          onclick={() => void updater.installAndRelaunch()}
        >
          Install and restart
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
    z-index: 210;
  }

  .modal {
    display: flex;
    flex-direction: column;
    width: 400px;
    max-height: 70vh;
    background: var(--bg-raised);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: var(--page-shadow);
    overflow: hidden;
  }

  header {
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
  }

  header h2 {
    font-size: 15px;
    font-weight: 600;
  }

  .body {
    padding: 14px 16px;
    overflow-y: auto;
  }

  .body p {
    font-size: 13px;
    line-height: 1.45;
    color: var(--text-1);
  }

  .muted {
    color: var(--text-3);
  }

  .notes {
    margin-top: 10px;
    padding: 10px;
    border-radius: 8px;
    background: var(--bg-input);
    font-family: inherit;
    font-size: 12px;
    line-height: 1.4;
    color: var(--text-2);
    white-space: pre-wrap;
    max-height: 160px;
    overflow-y: auto;
  }

  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 10px 16px;
    border-top: 1px solid var(--border);
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

  .primary {
    padding: 6px 12px;
    border-radius: 6px;
    background: var(--accent);
    color: white;
    font-size: 12px;
    font-weight: 500;
  }

  .primary:hover:not(:disabled) {
    background: var(--accent-strong);
  }

  .primary:disabled {
    opacity: 0.55;
  }
</style>

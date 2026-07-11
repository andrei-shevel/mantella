<script lang="ts">
  import Icon from "../common/Icon.svelte";
  import { settings } from "../../stores/settings.svelte";
  import { library } from "../../stores/library.svelte";

  let busy = $state(false);

  async function choose() {
    busy = true;
    try {
      const files = await settings.chooseLibraryFolder();
      if (files) library.setFiles(files);
    } finally {
      busy = false;
    }
  }
</script>

<div class="welcome" data-tauri-drag-region>
  <div class="card">
    <div class="logo"><Icon name="folder" size={42} /></div>
    <h1>Welcome to Mantella</h1>
    <p>
      Pick the folder where you keep your PDFs. It will be scanned — subfolders
      included — and kept in sync automatically as files come and go.
    </p>
    <button class="primary" onclick={choose} disabled={busy}>Choose Folder…</button>
  </div>
</div>

<style>
  .welcome {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .card {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    max-width: 380px;
    padding: 32px;
  }

  .logo {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 84px;
    height: 84px;
    border-radius: 22px;
    background: var(--accent-soft);
    color: var(--accent);
    margin-bottom: 22px;
  }

  h1 {
    font-size: 21px;
    font-weight: 700;
    margin-bottom: 10px;
  }

  p {
    color: var(--text-2);
    line-height: 1.5;
    margin-bottom: 26px;
  }

  .primary {
    background: var(--accent);
    color: #fff;
    font-weight: 600;
    padding: 9px 22px;
    border-radius: 8px;
  }

  .primary:hover {
    background: var(--accent-strong);
  }

  .primary:disabled {
    opacity: 0.6;
  }
</style>

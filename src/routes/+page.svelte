<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";

  let messages: string[] = ([]);
  let loading = (false);

  async function refresh() {
    loading = true;
    try {
      const list = await invoke('get_raw_messages');
      messages = list as string[];
    } catch (e) {
      console.error(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    refresh();
  });
</script>

<div class="container">
  <h1>Pha 1 - WS Test</h1>
  <p>Nhấn Refresh để xem 10 message WebSocket cuối (nếu có).</p>
  <button onclick={refresh} disabled={loading}>{loading ? 'Loading...' : 'Refresh'}</button>

  {#if messages.length === 0}
    <p style="margin-top:1rem;">Chưa có message.</p>
  {:else}
    <ul>
      {#each messages as m, i}
        <li><pre>{i+1}: {m}</pre></li>
      {/each}
    </ul>
  {/if}
</div>

<style>
.container {
  padding: 1rem;
  font-family: system-ui, sans-serif;
}
pre {
  background:#111;
  color:#eee;
  padding:6px 8px;
  border-radius:4px;
  font-size:12px;
  max-width: 680px;
  overflow-x:auto;
}
button {
  background:#2563eb;
  color:#fff;
  border:none;
  border-radius:4px;
  padding:6px 12px;
  cursor:pointer;
  font-size:14px;
}
button:disabled { opacity:0.6; cursor:default; }
h1 { margin-top:0; font-size:20px; }
</style>

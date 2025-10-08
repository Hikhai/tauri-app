<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  let apiKey = "";
  let apiSecret = "";
  let label = "default";
  let orders:any[] = [];
  let syncDays = 7;
  let activeTab:'orders'|'settings' = 'orders';
  let loading = false;
  let testResult = "";
  let errorMsg = "";

  async function loadOrders() {
    try { orders = await invoke('list_orders_from_db', { limit: 200 }); }
    catch (e:any) { errorMsg = e.toString(); }
  }
  async function saveCreds() {
    errorMsg=""; testResult="";
    try { await invoke('store_api_credentials', { label, apiKey, apiSecret }); testResult = "Đã lưu key."; }
    catch (e:any) { errorMsg = e.toString(); }
  }
  async function testCreds() {
    errorMsg=""; testResult="";
    try { const res = await invoke<string>('test_api_credentials'); testResult = "OK: "+res.slice(0,120)+"..."; }
    catch (e:any) { errorMsg = e.toString(); }
  }
  async function doForceSync() {
    loading = true; errorMsg="";
    try { await invoke('force_initial_sync', { days: syncDays }); await loadOrders(); }
    catch (e:any) { errorMsg = e.toString(); }
    finally { loading = false; }
  }
  onMount(() => { loadOrders(); });
</script>

<style>
:global(html) { background:#121212; color:#eee; font-family: system-ui, sans-serif; }
nav button { margin-right:8px; }
table { border-collapse: collapse; width:100%; font-size:12px; margin-top:10px; }
th,td { border:1px solid #333; padding:4px 6px; }
thead { background:#1e1e1e; }
.status-Đang\ chờ\ xử\ lý { color:#60a5fa; }
.status-Người\ mua\ đã\ thanh\ toán { color:#fbbf24; }
.status-Đã\ hoàn\ thành { color:#10b981; }
.status-Đã\ hủy { color:#ef4444; }
.error { color:#f87171; font-size:13px; margin-top:4px; }
.msg { color:#38bdf8; font-size:13px; margin-top:4px; }
input { background:#1f1f1f; color:#eee; border:1px solid #333; padding:4px 6px; margin:4px 0; }
button { background:#2563eb; color:#fff; border:none; padding:6px 10px; border-radius:4px; cursor:pointer; font-size:12px; }
button:disabled { opacity:0.6; cursor:default; }
button:hover:not(:disabled) { background:#1d4ed8; }
</style>

<nav>
  <button on:click={()=>activeTab='orders'} disabled={activeTab==='orders'}>Orders</button>
  <button on:click={()=>activeTab='settings'} disabled={activeTab==='settings'}>Settings</button>
</nav>

{#if activeTab==='orders'}
  <h2>Orders (API)</h2>
  <button on:click={loadOrders}>Reload</button>
  <span style="margin-left:12px;">Tổng: {orders.length}</span>
  {#if errorMsg}<div class="error">{errorMsg}</div>{/if}
  <table>
    <thead>
      <tr>
        <th>#</th><th>Type</th><th>Status</th><th>Fiat</th><th>Asset Amt</th><th>Price</th><th>Buyer</th><th>Seller</th><th>Payment?</th>
      </tr>
    </thead>
    <tbody>
      {#each orders as o}
        <tr>
          <td>{o.order_number.slice(-6)}</td>
          <td>{o.trade_type}</td>
          <td class={"status-"+o.status_label}>{o.status_label}</td>
          <td>{o.total_fiat} {o.fiat}</td>
          <td>{o.amount_asset} {o.asset}</td>
          <td>{o.price}</td>
          <td>{o.buyer_nickname}</td>
          <td>{o.seller_nickname}</td>
          <td>{o.has_payment_detail ? 'Yes':'No'}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

{#if activeTab==='settings'}
  <h2>API Credentials</h2>
  <div>
    <label>Label: <input bind:value={label} /></label><br/>
    <label>API Key: <input bind:value={apiKey} style="width:400px"/></label><br/>
    <label>API Secret: <input bind:value={apiSecret} style="width:400px"/></label><br/>
    <button on:click={saveCreds}>Lưu</button>
    <button on:click={testCreds}>Test</button>
  </div>
  <h3>Force Initial Sync</h3>
  <label>Days: <input type="number" bind:value={syncDays} min="1" max="30"/></label>
  <button disabled={loading} on:click={doForceSync}>{loading?'Syncing...':'Force Sync'}</button>
  {#if errorMsg}<div class="error">{errorMsg}</div>{/if}
  {#if testResult}<div class="msg">{testResult}</div>{/if}
{/if}
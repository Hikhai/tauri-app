<script lang="ts">
  import { onMount } from 'svelte';

  let orders:any[] = [];
  let nickname = 'User-a8bd4';
  let timer: any;
  let auto = true;

  async function load() {
    // @ts-ignore
    orders = await window.__TAURI__.invoke('list_orders');
  }
  async function setNick() {
    // @ts-ignore
    await window.__TAURI__.invoke('set_my_nickname', { nickname });
    load();
  }
  function copy(o:any) {
    const block = buildCopyBlock(o);
    navigator.clipboard.writeText(block);
  }
  function buildCopyBlock(o:any) {
    return [
      `ORDER: ${o.order_number}`,
      `ROLE: ${o.side_role}`,
      `TOTAL: ${o.total_fiat} ${o.fiat}`,
      `ASSET: ${o.amount_asset} ${o.asset} @ ${o.price}`,
      `BANK: ${o.bank_name}`,
      `ACCOUNT: ${o.account_no}`,
      `NAME: ${o.account_name}`
    ].filter(Boolean).join('\n');
  }

  onMount(() => {
    setNick();
    timer = setInterval(()=> { if(auto) load(); }, 3000);
    return () => clearInterval(timer);
  });
</script>

<style>
:global(html) { background:#121212; color:#eee; font-family: system-ui, sans-serif; }
h2 { margin: 8px 0 4px; }
.controls { display:flex; gap:8px; align-items:center; margin-bottom:8px; flex-wrap:wrap; }
input { background:#1f1f1f; color:#fff; border:1px solid #333; padding:4px 6px; border-radius:4px; }
button { background:#2563eb; border:none; color:#fff; padding:6px 10px; border-radius:4px; cursor:pointer; font-size:12px; }
button:hover { background:#1d4ed8; }
table { border-collapse: collapse; width:100%; font-size:12px; }
th, td { border:1px solid #333; padding:4px 6px; }
thead { background:#1e1e1e; }
tr:nth-child(even){ background:#181818; }
.copy-cell button { background:#10b981; }
.copy-cell button:hover { background:#059669; }
.status { font-weight:600; }
.role-YOU_BUY { color:#60a5fa; }
.role-YOU_SELL { color:#fbbf24; }
.role-OTHER { color:#9ca3af; }
</style>

<h2>P2P Orders (Phase 3)</h2>
<div class="controls">
  <label>Nickname: <input bind:value={nickname} style="width:140px" /></label>
  <button on:click={setNick}>Set</button>
  <button on:click={load}>Refresh</button>
  <label><input type="checkbox" bind:checked={auto}/> Auto 3s</label>
  <span>Total: {orders.length}</span>
</div>

<table>
  <thead>
    <tr>
      <th>ID (tail)</th>
      <th>Role</th>
      <th>Status</th>
      <th>Fiat Total</th>
      <th>Asset Amt</th>
      <th>Price</th>
      <th>Bank</th>
      <th>Account</th>
      <th>Name</th>
      <th>Copy</th>
    </tr>
  </thead>
  <tbody>
    {#each orders as o}
      <tr>
        <td>{o.order_number.slice(-6)}</td>
        <td class={"role-"+o.side_role}>{o.side_role}</td>
        <td class="status">{o.stage_label}</td>
        <td>{o.total_fiat} {o.fiat}</td>
        <td>{o.amount_asset} {o.asset}</td>
        <td>{o.price}</td>
        <td>{o.bank_name}</td>
        <td>{o.account_no}</td>
        <td>{o.account_name}</td>
        <td class="copy-cell"><button on:click={() => copy(o)}>Copy</button></td>
      </tr>
    {/each}
  </tbody>
</table>
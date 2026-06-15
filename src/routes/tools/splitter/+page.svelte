<svelte:head>
  <title>Expense Splitter — Kashif</title>
  <meta name="description" content="Split expenses with friends and settle up — a self-hosted Splitwise." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { env } from '$env/dynamic/public';
  import { Plus, Trash2, Users, Receipt, Scale, LogOut } from 'lucide-svelte';
  import {
    splitterApi,
    setToken,
    loadToken,
    AuthError,
    SPLITTER_CRED_KEY
  } from '$lib/splitterApi';
  import {
    formatMoney,
    toCents,
    resolveShares,
    computeBalances,
    settleUp,
    type Group,
    type GroupDetail,
    type SplitMode,
    type ShareInput
  } from '$lib/splitter';

  const clientId = env.PUBLIC_GOOGLE_CLIENT_ID ?? '';

  let signedIn = false;
  let gisButton: HTMLDivElement;

  let groups: Group[] = [];
  let activeId: number | null = null;
  let detail: GroupDetail | null = null;
  let errorMsg = '';
  let loadingDetail = false;

  // create-group form
  let newGroupName = '';
  let newGroupCurrency = 'INR';
  let newGroupMembers = '';

  // add-member form
  let newMemberName = '';

  // add-expense form
  let expDesc = '';
  let expAmount = '';
  let expPayer: number | null = null;
  let expMode: SplitMode = 'equal';
  let participants = new Set<number>();
  let memberInputs: Record<number, string> = {};

  // ---- auth ----------------------------------------------------------------

  function onCredential(resp: { credential: string }) {
    setToken(resp.credential);
    signedIn = true;
    refreshGroups();
  }

  function renderGoogleButton() {
    const google = (window as any).google;
    if (!google?.accounts?.id || !gisButton) return;
    google.accounts.id.initialize({ client_id: clientId, callback: onCredential });
    gisButton.innerHTML = '';
    google.accounts.id.renderButton(gisButton, {
      theme: 'filled_black',
      size: 'large',
      text: 'signin_with'
    });
  }

  function loadGis(): Promise<void> {
    if ((window as any).google?.accounts?.id) return Promise.resolve();
    return new Promise((resolve, reject) => {
      const s = document.createElement('script');
      s.src = 'https://accounts.google.com/gsi/client';
      s.async = true;
      s.onload = () => resolve();
      s.onerror = () => reject();
      document.head.appendChild(s);
    });
  }

  function signOut() {
    setToken(null);
    signedIn = false;
    groups = [];
    detail = null;
    activeId = null;
    setTimeout(renderGoogleButton, 0);
  }

  /** Run an API call, dropping the session and re-showing sign-in on 401. */
  async function guard<T>(fn: () => Promise<T>): Promise<T | undefined> {
    errorMsg = '';
    try {
      return await fn();
    } catch (e) {
      if (e instanceof AuthError) {
        signedIn = false;
        groups = [];
        detail = null;
        setTimeout(renderGoogleButton, 0);
      } else {
        errorMsg = e instanceof Error ? e.message : 'something went wrong';
      }
      return undefined;
    }
  }

  // ---- data ----------------------------------------------------------------

  async function refreshGroups() {
    const list = await guard(() => splitterApi.listGroups());
    if (list) {
      groups = list;
      if (activeId == null && groups.length) selectGroup(groups[0].id);
    }
  }

  async function selectGroup(id: number) {
    activeId = id;
    loadingDetail = true;
    const d = await guard(() => splitterApi.getGroup(id));
    loadingDetail = false;
    if (d) {
      detail = d;
      resetExpenseForm();
    }
  }

  async function createGroup() {
    if (!newGroupName.trim()) return;
    const members = newGroupMembers
      .split(/[\n,]/)
      .map((s) => s.trim())
      .filter(Boolean);
    const res = await guard(() =>
      splitterApi.createGroup(newGroupName.trim(), newGroupCurrency, members)
    );
    if (res) {
      newGroupName = '';
      newGroupMembers = '';
      await refreshGroups();
      selectGroup(res.id);
    }
  }

  async function deleteGroup(id: number) {
    if (!confirm('Delete this group and all its expenses?')) return;
    const res = await guard(() => splitterApi.deleteGroup(id));
    if (res) {
      if (activeId === id) {
        activeId = null;
        detail = null;
      }
      await refreshGroups();
    }
  }

  async function addMember() {
    if (!detail || !newMemberName.trim()) return;
    const res = await guard(() => splitterApi.addMember(detail!.group.id, newMemberName.trim()));
    if (res) {
      newMemberName = '';
      selectGroup(detail.group.id);
    }
  }

  async function deleteMember(memberId: number) {
    const res = await guard(() => splitterApi.deleteMember(memberId));
    if (res && detail) selectGroup(detail.group.id);
  }

  async function deleteExpense(expenseId: number) {
    const res = await guard(() => splitterApi.deleteExpense(expenseId));
    if (res && detail) selectGroup(detail.group.id);
  }

  // ---- add-expense form ----------------------------------------------------

  function resetExpenseForm() {
    expDesc = '';
    expAmount = '';
    expMode = 'equal';
    memberInputs = {};
    if (detail) {
      participants = new Set(detail.members.map((m) => m.id));
      expPayer = detail.members[0]?.id ?? null;
    } else {
      participants = new Set();
      expPayer = null;
    }
  }

  function toggleParticipant(id: number) {
    if (participants.has(id)) participants.delete(id);
    else participants.add(id);
    participants = new Set(participants); // trigger reactivity
  }

  // Live-resolved shares (cents) for the current form state.
  $: participantIds = detail ? detail.members.filter((m) => participants.has(m.id)).map((m) => m.id) : [];
  $: amountCents = toCents(expAmount);
  $: resolvedShares =
    amountCents > 0 && participantIds.length
      ? resolveShares(
          amountCents,
          participantIds,
          expMode,
          buildInputs(expMode, participantIds, memberInputs)
        )
      : ([] as ShareInput[]);
  $: shareSum = resolvedShares.reduce((a, s) => a + s.owed_cents, 0);
  $: expenseValid =
    amountCents > 0 &&
    participantIds.length > 0 &&
    expPayer != null &&
    (expMode !== 'exact' || shareSum === amountCents);

  function buildInputs(
    mode: SplitMode,
    ids: number[],
    raw: Record<number, string>
  ): Record<number, number> {
    const out: Record<number, number> = {};
    for (const id of ids) {
      const v = raw[id] ?? '';
      out[id] = mode === 'exact' ? toCents(v) : parseFloat(v) || 0;
    }
    return out;
  }

  async function addExpense() {
    if (!detail || !expenseValid || expPayer == null) return;
    const res = await guard(() =>
      splitterApi.addExpense(detail!.group.id, {
        payer_id: expPayer!,
        amount_cents: amountCents,
        description: expDesc.trim(),
        split_type: expMode,
        shares: resolvedShares
      })
    );
    if (res) selectGroup(detail.group.id);
  }

  // ---- derived: balances + settle-up --------------------------------------

  $: balances = detail
    ? computeBalances(detail.members, detail.expenses, detail.shares)
    : new Map<number, number>();
  $: settlements = settleUp(balances);
  $: currency = detail?.group.currency ?? 'INR';

  function memberName(id: number): string {
    return detail?.members.find((m) => m.id === id)?.name ?? '?';
  }

  // ---- lifecycle -----------------------------------------------------------

  onMount(async () => {
    if (!clientId) return;
    const stored = loadToken();
    if (stored) {
      signedIn = true;
      refreshGroups();
    } else {
      try {
        await loadGis();
        renderGoogleButton();
      } catch {
        /* GIS blocked */
      }
    }
  });
</script>

<div class="splitter">
  <header class="head">
    <div>
      <h1>Expense Splitter</h1>
      <p class="sub">Split with friends, track balances, settle up — backed by your account.</p>
    </div>
    {#if signedIn}
      <button class="ghost" on:click={signOut}><LogOut size={15} /> Sign out</button>
    {/if}
  </header>

  {#if errorMsg}
    <p class="error">{errorMsg}</p>
  {/if}

  {#if !clientId}
    <p class="note">Set <code>PUBLIC_GOOGLE_CLIENT_ID</code> to enable sign-in.</p>
  {:else if !signedIn}
    <div class="gate">
      <p class="note">Sign in with Google to manage your groups. Friends don't need accounts — you add them as names.</p>
      <div class="gis" bind:this={gisButton}></div>
    </div>
  {:else}
    <div class="layout">
      <!-- groups sidebar -->
      <aside class="sidebar">
        <h2 class="panel-title"><Users size={15} /> Groups</h2>
        <ul class="group-list">
          {#each groups as g}
            <li>
              <button class="group-item" class:active={g.id === activeId} on:click={() => selectGroup(g.id)}>
                {g.name}
                <span class="cur">{g.currency}</span>
              </button>
              <button class="icon-btn" title="Delete group" on:click={() => deleteGroup(g.id)}>
                <Trash2 size={14} />
              </button>
            </li>
          {/each}
          {#if !groups.length}
            <li class="empty">No groups yet.</li>
          {/if}
        </ul>

        <div class="create">
          <input class="in" placeholder="New group name" bind:value={newGroupName} />
          <div class="row">
            <select class="in" bind:value={newGroupCurrency}>
              <option>INR</option><option>USD</option><option>EUR</option><option>GBP</option>
            </select>
          </div>
          <textarea class="in" rows="2" placeholder="Members (comma or newline separated)" bind:value={newGroupMembers}></textarea>
          <button class="primary" on:click={createGroup}><Plus size={15} /> Create group</button>
        </div>
      </aside>

      <!-- group detail -->
      <main class="detail">
        {#if loadingDetail}
          <p class="note">Loading…</p>
        {:else if !detail}
          <p class="note">Select or create a group to get started.</p>
        {:else}
          <h2 class="group-name">{detail.group.name}</h2>

          <!-- members -->
          <section class="card">
            <h3 class="panel-title"><Users size={15} /> Members</h3>
            <div class="chips">
              {#each detail.members as m}
                <span class="chip">
                  {m.name}
                  <button class="chip-x" title="Remove" on:click={() => deleteMember(m.id)}>×</button>
                </span>
              {/each}
              {#if !detail.members.length}<span class="note">No members yet.</span>{/if}
            </div>
            <div class="row">
              <input class="in" placeholder="Add member name" bind:value={newMemberName}
                on:keydown={(e) => e.key === 'Enter' && addMember()} />
              <button class="primary" on:click={addMember}><Plus size={15} /> Add</button>
            </div>
          </section>

          <!-- balances + settle up -->
          <section class="card">
            <h3 class="panel-title"><Scale size={15} /> Balances & Settle up</h3>
            <ul class="balances">
              {#each detail.members as m}
                {@const bal = balances.get(m.id) ?? 0}
                <li>
                  <span>{m.name}</span>
                  <span class:pos={bal > 0} class:neg={bal < 0}>
                    {bal > 0 ? 'gets back ' : bal < 0 ? 'owes ' : ''}{formatMoney(Math.abs(bal), currency)}
                  </span>
                </li>
              {/each}
            </ul>
            {#if settlements.length}
              <div class="settle">
                <p class="settle-title">Suggested payments</p>
                <ul>
                  {#each settlements as s}
                    <li>{memberName(s.from)} → {memberName(s.to)} <strong>{formatMoney(s.cents, currency)}</strong></li>
                  {/each}
                </ul>
              </div>
            {:else}
              <p class="note">All settled up. 🎉</p>
            {/if}
          </section>

          <!-- add expense -->
          {#if detail.members.length >= 1}
            <section class="card">
              <h3 class="panel-title"><Plus size={15} /> Add expense</h3>
              <div class="row">
                <input class="in grow" placeholder="Description" bind:value={expDesc} />
                <input class="in" type="number" min="0" step="0.01" placeholder="Amount" bind:value={expAmount} />
              </div>
              <div class="row">
                <label class="lbl">Paid by
                  <select class="in" bind:value={expPayer}>
                    {#each detail.members as m}<option value={m.id}>{m.name}</option>{/each}
                  </select>
                </label>
                <label class="lbl">Split
                  <select class="in" bind:value={expMode}>
                    <option value="equal">Equally</option>
                    <option value="exact">Exact amounts</option>
                    <option value="percent">Percentages</option>
                    <option value="shares">Shares</option>
                  </select>
                </label>
              </div>

              <div class="split-grid">
                {#each detail.members as m}
                  {@const sh = resolvedShares.find((s) => s.member_id === m.id)}
                  <div class="split-row">
                    <label class="part">
                      <input type="checkbox" checked={participants.has(m.id)} on:change={() => toggleParticipant(m.id)} />
                      {m.name}
                    </label>
                    {#if expMode !== 'equal' && participants.has(m.id)}
                      <input class="in tiny" type="number" min="0" step="0.01"
                        placeholder={expMode === 'percent' ? '%' : expMode === 'shares' ? 'shares' : 'amount'}
                        bind:value={memberInputs[m.id]} />
                    {/if}
                    <span class="owed">{sh ? formatMoney(sh.owed_cents, currency) : ''}</span>
                  </div>
                {/each}
              </div>

              {#if expMode === 'exact' && amountCents > 0 && shareSum !== amountCents}
                <p class="warn">Exact amounts sum to {formatMoney(shareSum, currency)}, need {formatMoney(amountCents, currency)}.</p>
              {/if}

              <button class="primary" disabled={!expenseValid} on:click={addExpense}>
                <Receipt size={15} /> Add expense
              </button>
            </section>
          {/if}

          <!-- expense log -->
          <section class="card">
            <h3 class="panel-title"><Receipt size={15} /> Expenses</h3>
            <ul class="expenses">
              {#each detail.expenses as e}
                <li>
                  <div class="exp-main">
                    <span class="exp-desc">{e.description || '(no description)'}</span>
                    <span class="exp-meta">{memberName(e.payer_id)} paid · {e.split_type}</span>
                  </div>
                  <span class="exp-amt">{formatMoney(e.amount_cents, currency)}</span>
                  <button class="icon-btn" title="Delete" on:click={() => deleteExpense(e.id)}><Trash2 size={14} /></button>
                </li>
              {/each}
              {#if !detail.expenses.length}<li class="empty">No expenses yet.</li>{/if}
            </ul>
          </section>
        {/if}
      </main>
    </div>
  {/if}
</div>

<style>
  .splitter {
    max-width: 1100px;
    margin: 0 auto;
    padding: 24px 20px 80px;
    font-family: var(--font-body);
    color: var(--ink);
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 16px;
    margin-bottom: 18px;
  }
  h1 {
    font-family: var(--font-display);
    font-size: 2rem;
    text-transform: uppercase;
    letter-spacing: 0.02em;
    margin: 0;
  }
  .sub { color: var(--ink-soft); font-size: 0.95rem; margin: 4px 0 0; }
  .note { color: var(--ink-mute); font-size: 0.9rem; }
  .error { color: #e06c6c; font-size: 0.88rem; }
  .warn { color: #d9a441; font-size: 0.82rem; margin: 6px 0 0; }
  code { font-family: var(--font-mono); font-size: 0.85em; }

  .gate { padding: 32px 0; }
  .gis { margin-top: 14px; }

  .layout {
    display: grid;
    grid-template-columns: 280px minmax(0, 1fr);
    gap: 24px;
    align-items: start;
  }

  .panel-title {
    display: flex; align-items: center; gap: 7px;
    font-family: var(--font-mono);
    font-size: 0.72rem; letter-spacing: 0.12em; text-transform: uppercase;
    color: var(--ink-soft);
    margin: 0 0 12px;
  }

  /* sidebar */
  .group-list { list-style: none; margin: 0 0 16px; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  .group-list li { display: flex; align-items: center; gap: 4px; }
  .group-item {
    flex: 1; text-align: left; background: transparent; color: var(--ink-soft);
    border: 1px solid var(--rule-soft); padding: 9px 11px; cursor: pointer;
    display: flex; justify-content: space-between; align-items: center; gap: 8px;
    transition: color 0.12s, border-color 0.12s;
  }
  .group-item:hover { color: var(--ink); }
  .group-item.active { color: var(--blueprint); border-color: var(--blueprint); }
  .cur { font-family: var(--font-mono); font-size: 0.62rem; color: var(--ink-mute); }
  .empty { color: var(--ink-mute); font-size: 0.85rem; padding: 4px 0; }

  .create { display: flex; flex-direction: column; gap: 8px; border-top: 1px solid var(--rule-soft); padding-top: 14px; }

  /* detail */
  .group-name { font-family: var(--font-display); font-size: 1.5rem; text-transform: uppercase; margin: 0 0 16px; }
  .card { border: 1px solid var(--rule-soft); padding: 16px; margin-bottom: 16px; }

  .chips { display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 12px; }
  .chip {
    display: inline-flex; align-items: center; gap: 6px;
    background: var(--bg-surface); border: 1px solid var(--rule-soft);
    padding: 4px 6px 4px 10px; font-size: 0.85rem;
  }
  .chip-x { background: none; border: none; color: var(--ink-mute); cursor: pointer; font-size: 1.05rem; line-height: 1; }
  .chip-x:hover { color: #e06c6c; }

  .balances { list-style: none; margin: 0; padding: 0; }
  .balances li { display: flex; justify-content: space-between; padding: 5px 0; border-bottom: 1px solid var(--rule-soft); font-size: 0.92rem; }
  .pos { color: #5fae7e; }
  .neg { color: #e06c6c; }
  .settle { margin-top: 14px; }
  .settle-title { font-family: var(--font-mono); font-size: 0.7rem; letter-spacing: 0.1em; text-transform: uppercase; color: var(--ink-soft); margin: 0 0 6px; }
  .settle ul { list-style: none; margin: 0; padding: 0; }
  .settle li { padding: 4px 0; font-size: 0.92rem; }

  .split-grid { display: flex; flex-direction: column; gap: 4px; margin: 12px 0; }
  .split-row { display: grid; grid-template-columns: 1fr auto auto; align-items: center; gap: 10px; }
  .part { display: flex; align-items: center; gap: 8px; font-size: 0.9rem; }
  .owed { font-family: var(--font-mono); font-size: 0.82rem; color: var(--ink-soft); min-width: 80px; text-align: right; }

  .expenses { list-style: none; margin: 0; padding: 0; }
  .expenses li { display: flex; align-items: center; gap: 12px; padding: 9px 0; border-bottom: 1px solid var(--rule-soft); }
  .exp-main { flex: 1; display: flex; flex-direction: column; }
  .exp-desc { font-size: 0.95rem; }
  .exp-meta { font-family: var(--font-mono); font-size: 0.68rem; color: var(--ink-mute); text-transform: uppercase; letter-spacing: 0.05em; }
  .exp-amt { font-family: var(--font-mono); font-size: 0.92rem; }

  /* form controls */
  .row { display: flex; gap: 10px; flex-wrap: wrap; margin-bottom: 10px; }
  .lbl { display: flex; flex-direction: column; gap: 4px; font-size: 0.78rem; color: var(--ink-soft); }
  .in {
    background: var(--bg); color: var(--ink);
    border: 1px solid var(--rule-soft); padding: 8px 10px; font-family: var(--font-body); font-size: 0.9rem;
  }
  .in:focus { outline: none; border-color: var(--blueprint); }
  .grow { flex: 1; min-width: 160px; }
  .tiny { width: 90px; }
  textarea.in { resize: vertical; }

  button.primary, button.ghost {
    display: inline-flex; align-items: center; gap: 7px; cursor: pointer;
    font-family: var(--font-mono); font-size: 0.72rem; letter-spacing: 0.08em; text-transform: uppercase;
    padding: 8px 14px; border: 1px solid var(--blueprint); background: transparent; color: var(--blueprint);
    transition: background 0.12s, color 0.12s;
  }
  button.primary:hover:not(:disabled) { background: var(--blueprint); color: var(--bg); }
  button.primary:disabled { opacity: 0.4; cursor: not-allowed; }
  button.ghost { border-color: var(--rule-soft); color: var(--ink-soft); }
  button.ghost:hover { color: var(--ink); }
  .icon-btn { background: none; border: 1px solid var(--rule-soft); color: var(--ink-mute); cursor: pointer; padding: 7px; display: inline-flex; }
  .icon-btn:hover { color: #e06c6c; border-color: #e06c6c; }

  @media (max-width: 760px) {
    .layout { grid-template-columns: 1fr; }
  }
</style>

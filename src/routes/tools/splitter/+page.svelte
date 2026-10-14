<svelte:head>
  <title>Expense Splitter — Kashif</title>
  <meta name="description" content="Split expenses with friends and settle up — a self-hosted Splitwise." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { env } from '$env/dynamic/public';
  import { ArrowRight, CircleDollarSign, LogOut, Plus, Receipt, Scale, Trash2, Users } from 'lucide-svelte';
  import LoadingState from '$lib/components/LoadingState.svelte';
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
  <header class="appbar">
    <div class="appbar-left">
      <a href="/" class="wordmark" aria-label="ifkash.dev home"><span class="dot" aria-hidden="true"></span> ifkash</a>
      <nav class="crumb" aria-label="Breadcrumb">
        <span class="sep" aria-hidden="true">/</span>
        <a href="/tools" class="crumb-parent">tools</a>
        <span class="sep crumb-parent" aria-hidden="true">/</span>
        <span aria-current="page">splitter</span>
      </nav>
    </div>
    <div class="appbar-actions">
      {#if signedIn}
        <button class="btn btn-sm" type="button" on:click={signOut}><LogOut size={14} /> Sign out</button>
      {/if}
    </div>
  </header>

  <div class="work">
  <header class="head">
    <p class="eyebrow"><CircleDollarSign size={14} /> Shared expenses</p>
    <h1>Keep every shared cost clear.</h1>
    <p class="sub">Add an expense, see who owes what, and settle the group without the spreadsheet.</p>
  </header>

  {#if errorMsg}
    <p class="error" role="alert">{errorMsg}</p>
  {/if}

  {#if !clientId}
    <section class="notice">
      <CircleDollarSign size={20} />
      <div>
        <h2>Connect Google sign-in</h2>
        <p>Set <code>PUBLIC_GOOGLE_CLIENT_ID</code> to start managing shared expenses.</p>
      </div>
    </section>
  {:else if !signedIn}
    <section class="gate">
      <div>
        <p class="eyebrow">One account, any group</p>
        <h2>Split costs without the back-and-forth.</h2>
        <p>Sign in to create groups. You can add friends by name, so they do not need an account.</p>
        <div class="gis" bind:this={gisButton}></div>
      </div>
      <ol class="how-it-works">
        <li><span>1</span><div><strong>Create a group</strong><small>Start a trip, household, or dinner tab.</small></div></li>
        <li><span>2</span><div><strong>Add each expense</strong><small>Choose who paid and who shares it.</small></div></li>
        <li><span>3</span><div><strong>Settle the balance</strong><small>Use the suggested payments when you are ready.</small></div></li>
      </ol>
    </section>
  {:else}
    <div class="layout">
      <aside class="sidebar">
        <div class="sidebar-heading">
          <div>
            <p class="eyebrow">Your groups</p>
            <h2>Where are you splitting?</h2>
          </div>
          <span class="count">{groups.length}</span>
        </div>

        <nav aria-label="Expense groups">
          <ul class="group-list">
            {#each groups as g}
              <li>
                <button class="group-item" class:active={g.id === activeId} type="button" on:click={() => selectGroup(g.id)}>
                  <span>{g.name}</span>
                  <span class="cur">{g.currency}</span>
                </button>
                <button class="icon-btn danger" type="button" aria-label={`Delete ${g.name}`} title={`Delete ${g.name}`} on:click={() => deleteGroup(g.id)}>
                  <Trash2 size={15} />
                </button>
              </li>
            {/each}
          </ul>
        </nav>

        <section class="create" aria-labelledby="new-group-title">
          <p class="eyebrow" id="new-group-title">Start a new group</p>
          <label class="field-label" for="new-group-name">Group name</label>
          <input class="in" id="new-group-name" placeholder="Weekend in Goa" bind:value={newGroupName} on:keydown={(e) => e.key === 'Enter' && createGroup()} />
          <div class="currency-field">
            <label class="field-label" for="new-group-currency">Currency</label>
            <select class="in" id="new-group-currency" bind:value={newGroupCurrency}>
              <option>INR</option><option>USD</option><option>EUR</option><option>GBP</option>
            </select>
          </div>
          <label class="field-label" for="new-group-members">Add people <span>Optional</span></label>
          <textarea class="in" id="new-group-members" rows="2" placeholder="Ava, Sam, Noor" bind:value={newGroupMembers}></textarea>
          <button class="btn btn-primary full" type="button" disabled={!newGroupName.trim()} on:click={createGroup}><Plus size={16} /> Create group</button>
        </section>
      </aside>

      <main class="detail">
        {#if loadingDetail}
          <div class="loading"><LoadingState label="Loading group" /></div>
        {:else if !detail}
          <section class="empty-state">
            <div class="empty-icon"><Users size={24} /></div>
            <h2>Create your first group</h2>
            <p>Give the group a name, add the people involved, and start tracking shared costs.</p>
            <ArrowRight size={18} aria-hidden="true" />
          </section>
        {:else}
          <section class="group-summary">
            <div>
              <p class="eyebrow">Active group</p>
              <h2>{detail.group.name}</h2>
              <p>{detail.members.length} {detail.members.length === 1 ? 'person' : 'people'} · {detail.expenses.length} {detail.expenses.length === 1 ? 'expense' : 'expenses'}</p>
            </div>
            <span class="tag currency-badge">{currency}</span>
          </section>

          {#if detail.members.length >= 1}
            <section class="panel expense-card" aria-labelledby="add-expense-title">
              <div class="section-heading">
                <div>
                  <p class="eyebrow">Add an expense</p>
                  <h3 id="add-expense-title">Who paid for what?</h3>
                </div>
                <Receipt size={21} />
              </div>

              <div class="expense-basics">
                <label class="input-field">
                  <span>Description <em>Optional</em></span>
                  <input class="in" placeholder="Dinner at Olive" bind:value={expDesc} />
                </label>
                <label class="input-field amount-field">
                  <span>Amount</span>
                  <div class="amount-input"><span>{currency}</span><input class="in" type="number" min="0" step="0.01" inputmode="decimal" placeholder="0.00" bind:value={expAmount} /></div>
                </label>
              </div>

              <div class="expense-options">
                <label class="input-field">
                  <span>Paid by</span>
                  <select class="in" bind:value={expPayer}>
                    {#each detail.members as m}<option value={m.id}>{m.name}</option>{/each}
                  </select>
                </label>
                <fieldset class="split-choice">
                  <legend>Split between</legend>
                  <div class="split-modes">
                    <label><input type="radio" value="equal" bind:group={expMode} /> Equal</label>
                    <label><input type="radio" value="exact" bind:group={expMode} /> Exact</label>
                    <label><input type="radio" value="percent" bind:group={expMode} /> Percent</label>
                    <label><input type="radio" value="shares" bind:group={expMode} /> Shares</label>
                  </div>
                </fieldset>
              </div>

              <div class="participant-area">
                <div class="participant-heading">
                  <span>Include in this expense</span>
                  {#if amountCents > 0 && participantIds.length}<strong>{participantIds.length} selected</strong>{/if}
                </div>
                <div class="split-grid">
                  {#each detail.members as m}
                    {@const sh = resolvedShares.find((s) => s.member_id === m.id)}
                    <label class="split-row" class:included={participants.has(m.id)}>
                      <span class="part">
                        <input type="checkbox" checked={participants.has(m.id)} on:change={() => toggleParticipant(m.id)} />
                        <span>{m.name}</span>
                      </span>
                      {#if expMode !== 'equal' && participants.has(m.id)}
                        <input class="in tiny" type="number" min="0" step="0.01" inputmode="decimal"
                          placeholder={expMode === 'percent' ? '%' : expMode === 'shares' ? 'Shares' : 'Amount'}
                          bind:value={memberInputs[m.id]} />
                      {/if}
                      <span class="owed">{sh ? formatMoney(sh.owed_cents, currency) : '—'}</span>
                    </label>
                  {/each}
                </div>
              </div>

              {#if expMode === 'exact' && amountCents > 0 && shareSum !== amountCents}
                <p class="warn" role="status">Amounts add up to {formatMoney(shareSum, currency)}. Enter {formatMoney(amountCents - shareSum, currency)} more.</p>
              {/if}

              <div class="expense-action">
                <p>{expenseValid ? `Ready to split ${formatMoney(amountCents, currency)}.` : 'Enter an amount and choose at least one person to continue.'}</p>
                <button class="btn btn-primary" type="button" disabled={!expenseValid} on:click={addExpense}><Receipt size={16} /> Add expense</button>
              </div>
            </section>
          {:else}
            <section class="notice">
              <Users size={20} />
              <div><h3>Add people first</h3><p>Add at least one person before you record an expense.</p></div>
            </section>
          {/if}

          <div class="overview-grid">
            <section class="panel balance-card">
              <div class="section-heading">
                <div><p class="eyebrow">At a glance</p><h3>Balances</h3></div>
                <Scale size={21} />
              </div>
              <ul class="balances">
                {#each detail.members as m}
                  {@const bal = balances.get(m.id) ?? 0}
                  <li>
                    <span>{m.name}</span>
                    <span class:pos={bal > 0} class:neg={bal < 0}>
                      {bal > 0 ? `Gets back ${formatMoney(bal, currency)}` : bal < 0 ? `Owes ${formatMoney(Math.abs(bal), currency)}` : 'Settled'}
                    </span>
                  </li>
                {/each}
              </ul>
              {#if settlements.length}
                <div class="settle">
                  <p>Suggested payments</p>
                  <ul>
                    {#each settlements as s}
                      <li><span>{memberName(s.from)} <ArrowRight size={14} /> {memberName(s.to)}</span><strong>{formatMoney(s.cents, currency)}</strong></li>
                    {/each}
                  </ul>
                </div>
              {:else if detail.members.length}
                <p class="settled"><span class="led" aria-hidden="true"></span> Everyone is settled up.</p>
              {/if}
            </section>

            <section class="panel members-card">
              <div class="section-heading">
                <div><p class="eyebrow">People</p><h3>Members</h3></div>
                <Users size={21} />
              </div>
              <div class="chips">
                {#each detail.members as m}
                  <span class="tag chip">
                    <span>{m.name}</span>
                    <button class="chip-x" type="button" aria-label={`Remove ${m.name}`} title={`Remove ${m.name}`} on:click={() => deleteMember(m.id)}><Trash2 size={13} /></button>
                  </span>
                {/each}
              </div>
              <div class="member-add">
                <input class="in" placeholder="Add a person" bind:value={newMemberName} on:keydown={(e) => e.key === 'Enter' && addMember()} />
                <button class="btn" type="button" disabled={!newMemberName.trim()} on:click={addMember}><Plus size={16} /> Add</button>
              </div>
            </section>
          </div>

          <section class="panel expenses-card">
            <div class="section-heading">
              <div><p class="eyebrow">History</p><h3>Expenses</h3></div>
              <span class="count">{detail.expenses.length}</span>
            </div>
            {#if detail.expenses.length}
              <ul class="expenses">
                {#each detail.expenses as e}
                  <li>
                    <div class="expense-mark"><Receipt size={17} /></div>
                    <div class="exp-main">
                      <span class="exp-desc">{e.description || 'Untitled expense'}</span>
                      <span class="exp-meta">{memberName(e.payer_id)} paid · {e.split_type} split</span>
                    </div>
                    <span class="exp-amt">{formatMoney(e.amount_cents, currency)}</span>
                    <button class="icon-btn danger" type="button" aria-label={`Delete ${e.description || 'expense'}`} title="Delete expense" on:click={() => deleteExpense(e.id)}><Trash2 size={15} /></button>
                  </li>
                {/each}
              </ul>
            {:else}
              <div class="expense-empty"><Receipt size={20} /><p>Your expenses will appear here.</p></div>
            {/if}
          </section>
        {/if}
      </main>
    </div>
  {/if}
  </div>
</div>

<style>
  /* ── Shell ───────────────────────────────────────────── */

  .splitter {
    min-height: 100vh;
    min-height: 100dvh;
    background: var(--bg);
    color: var(--ink-2);
  }

  .appbar {
    position: sticky;
    top: 0;
    z-index: 30;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-4);
    height: 56px;
    padding: 0 var(--gutter);
    border-bottom: 1px solid var(--line);
    background: color-mix(in srgb, var(--bg) 92%, transparent);
    backdrop-filter: saturate(1.2) blur(12px);
    -webkit-backdrop-filter: saturate(1.2) blur(12px);
  }

  .appbar-left {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    min-width: 0;
  }

  .wordmark {
    display: inline-flex;
    flex-shrink: 0;
    align-items: center;
    gap: 8px;
    color: var(--ink);
    font-size: 16px;
    font-weight: 500;
    letter-spacing: -0.025em;
    white-space: nowrap;
  }
  .wordmark .dot {
    width: 8px;
    height: 8px;
  }
  .wordmark:hover {
    color: var(--ink);
  }

  .crumb {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    min-width: 0;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    white-space: nowrap;
  }
  .crumb a {
    color: var(--ink-3);
  }
  .crumb a:hover {
    color: var(--ink);
  }
  .crumb [aria-current="page"] {
    color: var(--ink);
  }
  .sep {
    color: var(--line-strong);
  }

  .appbar-actions {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    gap: var(--s-2);
  }

  .work {
    max-width: calc(var(--content-max) + 2 * var(--gutter));
    margin: 0 auto;
    padding: var(--s-7) var(--gutter) var(--s-9);
  }

  /* ── Type ────────────────────────────────────────────── */

  h1,
  h2,
  h3,
  p {
    margin-top: 0;
  }

  .eyebrow {
    display: flex;
    margin: 0 0 var(--s-2);
  }
  .eyebrow :global(svg) {
    color: var(--ink-3);
  }

  .head {
    padding-bottom: var(--s-6);
    margin-bottom: var(--s-7);
    border-bottom: 1px solid var(--line);
  }

  h1 {
    max-width: 18ch;
    margin: var(--s-2) 0 var(--s-3);
    color: var(--ink);
    font-size: var(--text-2xl);
    font-weight: 500;
    line-height: 1.05;
    letter-spacing: -0.035em;
  }

  .sub {
    max-width: var(--measure);
    margin-bottom: 0;
    color: var(--ink-2);
    font-size: var(--text-md);
    line-height: 1.55;
  }

  .error {
    margin: calc(-1 * var(--s-5)) 0 var(--s-5);
    padding: var(--s-2) var(--s-3);
    border-left: 2px solid var(--danger);
    color: var(--danger);
    font-size: var(--text-sm);
  }

  code {
    font-size: 0.86em;
  }

  /* ── Notices and the sign-in gate ────────────────────── */

  .notice {
    display: flex;
    gap: var(--s-4);
    max-width: 560px;
    padding: var(--s-5);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--ink-2);
  }
  .notice :global(svg) {
    flex: 0 0 auto;
    margin-top: 2px;
    color: var(--ink-3);
  }
  .notice h2,
  .notice h3 {
    margin-bottom: var(--s-1);
    color: var(--ink);
    font-size: var(--text-base);
    letter-spacing: -0.01em;
  }
  .notice p {
    margin-bottom: 0;
    font-size: var(--text-sm);
    line-height: 1.55;
  }

  .gate {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(280px, 0.8fr);
    align-items: center;
    gap: var(--s-8);
    max-width: 960px;
    padding: var(--s-7);
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface);
  }
  .gate h2 {
    max-width: 16ch;
    margin-bottom: var(--s-4);
    color: var(--ink);
    font-size: var(--text-xl);
    line-height: 1.12;
    letter-spacing: -0.025em;
  }
  .gate > div > p:not(.eyebrow) {
    max-width: 44ch;
    margin-bottom: var(--s-5);
    color: var(--ink-2);
    line-height: 1.6;
  }
  .gis {
    min-height: 40px;
  }

  .how-it-works {
    margin: 0;
    padding: 0;
    border-top: 1px solid var(--line);
    list-style: none;
  }
  .how-it-works li {
    display: grid;
    grid-template-columns: 32px minmax(0, 1fr);
    gap: var(--s-3);
    padding: var(--s-4) 0;
    border-bottom: 1px solid var(--line);
  }
  .how-it-works li > span {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    line-height: 1.9;
  }
  .how-it-works li > span::before {
    content: "0";
  }
  .how-it-works strong,
  .how-it-works small {
    display: block;
  }
  .how-it-works strong {
    margin-bottom: 2px;
    color: var(--ink);
    font-size: 15px;
    font-weight: 500;
  }
  .how-it-works small {
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.45;
  }

  /* ── Layout ──────────────────────────────────────────── */

  .layout {
    display: grid;
    grid-template-columns: 272px minmax(0, 1fr);
    align-items: start;
    gap: var(--s-8);
  }

  .sidebar {
    position: sticky;
    top: calc(56px + var(--s-5));
  }

  .sidebar-heading {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-4);
    margin-bottom: var(--s-4);
  }
  .sidebar-heading h2 {
    max-width: 200px;
    margin-bottom: 0;
    color: var(--ink);
    font-size: var(--text-md);
    line-height: 1.25;
    letter-spacing: -0.015em;
  }

  .count {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  .group-list {
    display: grid;
    margin: 0 0 var(--s-6);
    padding: 0;
    border-top: 1px solid var(--line);
    list-style: none;
  }
  .group-list li {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 32px;
    align-items: center;
    gap: var(--s-1);
    border-bottom: 1px solid var(--line);
  }

  .group-item {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
    min-width: 0;
    min-height: 44px;
    padding: 0 var(--s-2) 0 var(--s-5);
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--ink-2);
    font: inherit;
    font-size: 15px;
    text-align: left;
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease), color var(--dur-fast) var(--ease);
  }
  .group-item::before {
    content: "";
    position: absolute;
    left: var(--s-2);
    top: 50%;
    width: 6px;
    height: 6px;
    margin-top: -3px;
    border-radius: 50%;
    box-shadow: inset 0 0 0 1px var(--line-strong);
  }
  .group-item > span:first-child {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .group-item:hover {
    background: var(--hover-wash);
    color: var(--ink);
  }
  .group-item.active {
    color: var(--ink);
    font-weight: 500;
  }
  .group-item.active::before {
    background: var(--moss);
    box-shadow: none;
  }

  .cur {
    flex: 0 0 auto;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 400;
  }

  .create {
    display: grid;
    gap: var(--s-2);
    padding: var(--s-5);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface);
  }
  .create .eyebrow {
    margin-bottom: var(--s-2);
  }
  .create .field-label {
    margin-top: var(--s-2);
  }
  .create .full {
    margin-top: var(--s-3);
  }
  .currency-field {
    display: grid;
    gap: var(--s-2);
    max-width: 120px;
  }

  .detail {
    min-width: 0;
  }

  /* ── Empty and loading ───────────────────────────────── */

  .loading,
  .empty-state {
    display: grid;
    place-items: center;
    align-content: center;
    min-height: 360px;
    border: 1px dashed var(--line-strong);
    border-radius: var(--radius-sm);
    color: var(--ink-3);
    text-align: center;
  }
  .empty-state {
    gap: var(--s-2);
    padding: var(--s-6);
  }
  .empty-state > :global(svg) {
    color: var(--ink-3);
  }
  .empty-icon {
    display: grid;
    place-items: center;
    width: 48px;
    height: 48px;
    margin-bottom: var(--s-3);
    border: 1px solid var(--line);
    border-radius: 50%;
    color: var(--ink-2);
  }
  .empty-state h2 {
    margin-bottom: 0;
    color: var(--ink);
    font-size: var(--text-lg);
    letter-spacing: -0.015em;
  }
  .empty-state p {
    max-width: 40ch;
    margin-bottom: var(--s-2);
    color: var(--ink-3);
    font-size: 15px;
    line-height: 1.55;
  }

  /* ── Group summary and panels ────────────────────────── */

  .group-summary {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: var(--s-5);
    margin-bottom: var(--s-6);
  }
  .group-summary h2 {
    margin-bottom: var(--s-2);
    color: var(--ink);
    font-size: var(--text-2xl);
    line-height: 1.05;
    letter-spacing: -0.035em;
    overflow-wrap: anywhere;
  }
  .group-summary > div > p:last-child {
    margin-bottom: 0;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }
  .currency-badge {
    flex-shrink: 0;
    color: var(--ink);
  }

  .panel {
    padding: var(--s-6);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface);
  }

  .section-heading {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-4);
    margin-bottom: var(--s-5);
  }
  .section-heading .eyebrow {
    margin-bottom: var(--s-1);
  }
  .section-heading h3 {
    margin-bottom: 0;
    color: var(--ink);
    font-size: var(--text-lg);
    letter-spacing: -0.015em;
  }
  .section-heading > :global(svg) {
    flex-shrink: 0;
    color: var(--ink-3);
  }

  /* ── Expense form ────────────────────────────────────── */

  .expense-card {
    margin-bottom: var(--s-5);
  }

  .expense-basics,
  .expense-options {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 200px;
    gap: var(--s-4);
  }
  .expense-options {
    grid-template-columns: 200px minmax(0, 1fr);
    margin-top: var(--s-5);
  }

  .input-field,
  .field-label,
  .split-choice legend {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 400;
  }
  .input-field {
    display: grid;
    gap: var(--s-2);
  }
  .input-field em,
  .field-label span {
    margin-left: var(--s-1);
    color: var(--ink-3);
    font-style: normal;
    opacity: 0.75;
  }

  .amount-input {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    align-items: center;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--surface);
    transition: border-color var(--dur-fast) var(--ease), box-shadow var(--dur-fast) var(--ease);
  }
  .amount-input:focus-within {
    border-color: var(--fjord);
    box-shadow: 0 0 0 3px var(--fjord-wash);
  }
  .amount-input > span {
    padding-left: var(--s-3);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .amount-input .in {
    border: 0;
    box-shadow: none;
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }

  .split-choice {
    min-width: 0;
    padding: 0;
    border: 0;
  }
  .split-choice legend {
    margin-bottom: var(--s-2);
    padding: 0;
  }

  /* Segmented control; the radios stay focusable but hidden. */
  .split-modes {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 2px;
    padding: 2px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--surface-2);
  }
  .split-modes label {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 34px;
    border-radius: var(--radius-xs);
    color: var(--ink-2);
    font-size: var(--text-sm);
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease), color var(--dur-fast) var(--ease);
  }
  .split-modes label:hover {
    color: var(--ink);
  }
  .split-modes input {
    position: absolute;
    width: 1px;
    height: 1px;
    opacity: 0;
    pointer-events: none;
  }
  .split-modes label:has(input:checked) {
    background: var(--surface);
    color: var(--ink);
    font-weight: 500;
    box-shadow: inset 0 0 0 1px var(--line);
  }
  .split-modes label:has(input:focus-visible) {
    outline: 2px solid var(--fjord);
    outline-offset: 1px;
  }

  .participant-area {
    margin-top: var(--s-5);
  }
  .participant-heading {
    display: flex;
    justify-content: space-between;
    gap: var(--s-3);
    margin-bottom: var(--s-2);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .participant-heading strong {
    color: var(--ink);
    font-weight: 400;
    font-variant-numeric: tabular-nums;
  }

  .split-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--s-2);
  }
  .split-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    align-items: center;
    gap: var(--s-3);
    min-height: 48px;
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    color: var(--ink-2);
    cursor: pointer;
    transition: border-color var(--dur-fast) var(--ease), background-color var(--dur-fast) var(--ease);
  }
  .split-row:hover {
    border-color: var(--line-strong);
  }
  .split-row.included {
    border-color: var(--line-strong);
    background: var(--bg);
    color: var(--ink);
  }
  .part {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    min-width: 0;
    font-size: 15px;
  }
  .part > span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .part input {
    flex-shrink: 0;
    accent-color: var(--fjord);
  }
  .owed {
    min-width: 64px;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .tiny {
    width: 84px;
    min-height: 32px;
    padding: 0 var(--s-2);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
  }

  .warn {
    margin: var(--s-3) 0 0;
    color: var(--clay);
    font-size: var(--text-sm);
    line-height: 1.45;
  }

  .expense-action {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-4);
    margin-top: var(--s-5);
    padding-top: var(--s-5);
    border-top: 1px solid var(--line);
  }
  .expense-action p {
    margin-bottom: 0;
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.45;
  }

  /* ── Balances and members ────────────────────────────── */

  .overview-grid {
    display: grid;
    grid-template-columns: minmax(0, 1.2fr) minmax(0, 0.8fr);
    gap: var(--s-5);
    margin-bottom: var(--s-5);
  }

  .balances {
    margin: 0;
    padding: 0;
    border-top: 1px solid var(--line);
    list-style: none;
  }
  .balances li {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-4);
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
    color: var(--ink);
    font-size: 15px;
  }
  .balances li > span:first-child {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .balances li > span:last-child {
    flex: 0 0 auto;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .balances .pos {
    color: var(--moss);
  }
  .balances .neg {
    color: var(--clay);
  }

  .settle {
    margin-top: var(--s-5);
    padding: var(--s-4);
    border-radius: var(--radius-sm);
    background: var(--surface-2);
  }
  .settle > p {
    margin-bottom: var(--s-3);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .settle ul {
    display: grid;
    gap: var(--s-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .settle li {
    display: flex;
    justify-content: space-between;
    gap: var(--s-3);
    color: var(--ink-2);
    font-size: var(--text-sm);
  }
  .settle li span {
    display: inline-flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-1);
    min-width: 0;
  }
  .settle li span :global(svg) {
    color: var(--ink-3);
  }
  .settle strong {
    flex-shrink: 0;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-weight: 500;
    font-variant-numeric: tabular-nums;
  }

  .settled {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin: var(--s-4) 0 0;
    color: var(--ink-2);
    font-size: var(--text-sm);
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin-bottom: var(--s-4);
  }
  .chip {
    height: 28px;
    padding: 0 var(--s-1) 0 var(--s-3);
    color: var(--ink);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
  }
  .chip-x {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: transparent;
    color: var(--ink-3);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), background-color var(--dur-fast) var(--ease);
  }
  .chip-x:hover {
    background: var(--hover-wash);
    color: var(--danger);
  }

  .member-add {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: var(--s-2);
  }

  /* ── Expense history ─────────────────────────────────── */

  .expenses {
    margin: 0;
    padding: 0;
    border-top: 1px solid var(--line);
    list-style: none;
  }
  .expenses li {
    display: grid;
    grid-template-columns: 32px minmax(0, 1fr) auto 32px;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
  }
  .expenses li:last-child {
    border-bottom: 0;
  }
  .expense-mark {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    color: var(--ink-3);
  }
  .exp-main {
    display: grid;
    gap: 2px;
    min-width: 0;
  }
  .exp-desc {
    overflow: hidden;
    color: var(--ink);
    font-size: 15px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .exp-meta {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .exp-amt {
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .expense-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--s-2);
    min-height: 112px;
    color: var(--ink-3);
    font-size: var(--text-sm);
  }
  .expense-empty p {
    margin: 0;
    color: var(--ink-3);
  }

  /* ── Controls ────────────────────────────────────────── */

  .in {
    width: 100%;
    min-width: 0;
    min-height: 40px;
    padding: 0 var(--s-3);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--ink);
    font: inherit;
    font-size: 15px;
    transition: border-color var(--dur-fast) var(--ease), box-shadow var(--dur-fast) var(--ease);
  }
  .in::placeholder {
    color: var(--ink-3);
  }
  .in:focus {
    outline: none;
    border-color: var(--fjord);
    box-shadow: 0 0 0 3px var(--fjord-wash);
  }
  textarea.in {
    padding: var(--s-2) var(--s-3);
    line-height: 1.5;
    resize: vertical;
  }
  .create .in {
    background: var(--bg);
  }

  .full {
    width: 100%;
  }

  .icon-btn {
    display: inline-grid;
    place-items: center;
    width: 32px;
    height: 32px;
    padding: 0;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-3);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), background-color var(--dur-fast) var(--ease);
  }
  .icon-btn:hover {
    background: var(--hover-wash);
    color: var(--ink);
  }
  .icon-btn.danger:hover {
    color: var(--danger);
  }

  /* ── Responsive ──────────────────────────────────────── */

  @media (max-width: 960px) {
    .layout {
      grid-template-columns: 1fr;
      gap: var(--s-7);
    }
    .sidebar {
      position: static;
    }
    .create {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      align-items: end;
    }
    .create > .eyebrow,
    .create > .full,
    .create > textarea,
    .create > label[for="new-group-members"] {
      grid-column: 1 / -1;
    }
    .currency-field {
      max-width: none;
    }
  }

  @media (max-width: 640px) {
    .work {
      padding-top: var(--s-6);
      padding-bottom: var(--s-8);
    }
    .gate {
      grid-template-columns: 1fr;
      gap: var(--s-6);
      padding: var(--s-5);
    }
    .panel {
      padding: var(--s-4);
    }
    .create,
    .expense-basics,
    .expense-options,
    .overview-grid,
    .split-grid {
      grid-template-columns: 1fr;
    }
    .create > * {
      grid-column: auto;
    }
    .split-modes {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .split-row {
      grid-template-columns: minmax(0, 1fr) auto;
    }
    .split-row .tiny {
      grid-column: 1;
      grid-row: 2;
      width: 100%;
    }
    .split-row .owed {
      grid-column: 2;
      grid-row: 1;
    }
    .expense-action {
      flex-direction: column;
      align-items: stretch;
    }
    .group-summary {
      align-items: flex-start;
    }
    .expenses li {
      grid-template-columns: minmax(0, 1fr) auto 32px;
    }
    .expense-mark {
      display: none;
    }
  }

  @media (max-width: 400px) {
    .crumb-parent {
      display: none;
    }
  }
</style>

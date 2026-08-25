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
  <header class="head">
    <div class="intro">
      <p class="eyebrow"><CircleDollarSign size={14} /> Shared expenses</p>
      <h1>Keep every shared cost clear.</h1>
      <p class="sub">Add an expense, see who owes what, and settle the group without the spreadsheet.</p>
    </div>
    {#if signedIn}
      <button class="ghost" type="button" on:click={signOut}><LogOut size={16} /> Sign out</button>
    {/if}
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
          <button class="primary full" type="button" disabled={!newGroupName.trim()} on:click={createGroup}><Plus size={16} /> Create group</button>
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
            <span class="currency-badge">{currency}</span>
          </section>

          {#if detail.members.length >= 1}
            <section class="card expense-card" aria-labelledby="add-expense-title">
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
                <button class="primary" type="button" disabled={!expenseValid} on:click={addExpense}><Receipt size={16} /> Add expense</button>
              </div>
            </section>
          {:else}
            <section class="notice">
              <Users size={20} />
              <div><h3>Add people first</h3><p>Add at least one person before you record an expense.</p></div>
            </section>
          {/if}

          <div class="overview-grid">
            <section class="card balance-card">
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
                <p class="settled">Everyone is settled up.</p>
              {/if}
            </section>

            <section class="card members-card">
              <div class="section-heading">
                <div><p class="eyebrow">People</p><h3>Members</h3></div>
                <Users size={21} />
              </div>
              <div class="chips">
                {#each detail.members as m}
                  <span class="chip">
                    <span>{m.name}</span>
                    <button class="chip-x" type="button" aria-label={`Remove ${m.name}`} title={`Remove ${m.name}`} on:click={() => deleteMember(m.id)}><Trash2 size={13} /></button>
                  </span>
                {/each}
              </div>
              <div class="member-add">
                <input class="in" placeholder="Add a person" bind:value={newMemberName} on:keydown={(e) => e.key === 'Enter' && addMember()} />
                <button class="secondary" type="button" disabled={!newMemberName.trim()} on:click={addMember}><Plus size={16} /> Add</button>
              </div>
            </section>
          </div>

          <section class="card expenses-card">
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

<style>
  .splitter {
    max-width: 1240px;
    margin: 0 auto;
    padding: clamp(28px, 5vw, 64px) var(--frame-gutter) 96px;
    color: var(--ink);
    font-family: var(--font-body);
  }
  .head { display: flex; justify-content: space-between; align-items: flex-start; gap: 24px; margin-bottom: clamp(32px, 5vw, 56px); }
  .intro { max-width: 700px; }
  .eyebrow { display: flex; align-items: center; gap: 7px; margin: 0 0 9px; color: var(--ink-mute); font-family: var(--font-mono); font-size: 0.67rem; font-weight: 500; letter-spacing: 0.1em; text-transform: uppercase; }
  h1, h2, h3, p { margin-top: 0; }
  h1 { max-width: 650px; margin-bottom: 12px; font-family: var(--font-display); font-size: clamp(2.1rem, 5vw, 4.25rem); font-weight: 600; letter-spacing: -0.055em; line-height: 0.98; }
  .sub { max-width: 590px; margin-bottom: 0; color: var(--ink-soft); font-size: 1rem; line-height: 1.55; }
  .error { margin: -24px 0 24px; color: var(--red); font-size: 0.9rem; }
  .notice { display: flex; max-width: 560px; gap: 14px; padding: 20px; border: 1px solid var(--rule-soft); background: var(--fill); color: var(--ink-soft); }
  .notice :global(svg) { flex: 0 0 auto; color: var(--ink); }
  .notice h2, .notice h3 { margin-bottom: 4px; color: var(--ink); font-size: 1rem; }
  .notice p { margin-bottom: 0; font-size: 0.9rem; line-height: 1.5; }
  code { font-family: var(--font-mono); font-size: 0.85em; }

  .gate { display: grid; grid-template-columns: minmax(0, 1fr) minmax(320px, 0.75fr); gap: 48px; align-items: center; max-width: 940px; padding: clamp(24px, 5vw, 52px); border: 1px solid var(--border); background: linear-gradient(135deg, var(--fill), transparent); }
  .gate h2 { max-width: 430px; margin-bottom: 14px; font-size: clamp(1.8rem, 3vw, 2.55rem); letter-spacing: -0.04em; line-height: 1.05; }
  .gate > div > p:not(.eyebrow) { max-width: 440px; margin-bottom: 24px; color: var(--ink-soft); line-height: 1.6; }
  .gis { min-height: 40px; }
  .how-it-works { display: grid; gap: 20px; margin: 0; padding: 0; list-style: none; }
  .how-it-works li { display: flex; align-items: flex-start; gap: 12px; }
  .how-it-works li > span { display: grid; width: 26px; height: 26px; place-items: center; border: 1px solid var(--border-strong); border-radius: 50%; color: var(--ink); font-family: var(--font-mono); font-size: 0.7rem; }
  .how-it-works strong, .how-it-works small { display: block; }
  .how-it-works strong { margin: 2px 0 4px; color: var(--ink); font-size: 0.93rem; font-weight: 500; }
  .how-it-works small { color: var(--ink-mute); font-size: 0.82rem; line-height: 1.4; }

  .layout { display: grid; grid-template-columns: 284px minmax(0, 1fr); gap: clamp(28px, 5vw, 72px); align-items: start; }
  .sidebar { position: sticky; top: 92px; }
  .sidebar-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; margin-bottom: 16px; }
  .sidebar-heading h2 { max-width: 200px; margin-bottom: 0; font-size: 1.12rem; letter-spacing: -0.025em; line-height: 1.2; }
  .count, .currency-badge { display: inline-flex; align-items: center; justify-content: center; min-width: 28px; height: 24px; border: 1px solid var(--border); color: var(--ink-soft); font-family: var(--font-mono); font-size: 0.68rem; }
  .group-list { display: grid; gap: 6px; margin: 0 0 26px; padding: 0; list-style: none; }
  .group-list li { display: grid; grid-template-columns: minmax(0, 1fr) 35px; gap: 6px; }
  .group-item { display: flex; min-width: 0; align-items: center; justify-content: space-between; gap: 12px; padding: 11px 12px; border: 1px solid transparent; background: transparent; color: var(--ink-soft); cursor: var(--cursor-pointer); font: inherit; text-align: left; transition: background 160ms var(--ease-out), border-color 160ms var(--ease-out), color 160ms var(--ease-out); }
  .group-item > span:first-child { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .group-item:hover, .group-item.active { border-color: var(--border); background: var(--fill); color: var(--ink); }
  .group-item.active { border-color: var(--border-strong); }
  .cur { flex: 0 0 auto; color: var(--ink-mute); font-family: var(--font-mono); font-size: 0.65rem; }
  .create { display: grid; gap: 8px; padding-top: 22px; border-top: 1px solid var(--rule-soft); }
  .create .eyebrow { margin-bottom: 2px; }
  .currency-field { max-width: 112px; }

  .detail { min-width: 0; }
  .loading, .empty-state { display: grid; min-height: 360px; place-items: center; align-content: center; border: 1px solid var(--rule-soft); color: var(--ink-soft); text-align: center; }
  .empty-state { padding: 28px; }
  .empty-icon { display: grid; width: 50px; height: 50px; place-items: center; margin-bottom: 18px; border: 1px solid var(--border); border-radius: 50%; color: var(--ink); }
  .empty-state h2 { margin-bottom: 8px; color: var(--ink); font-size: 1.35rem; letter-spacing: -0.03em; }
  .empty-state p { max-width: 360px; margin-bottom: 18px; font-size: 0.92rem; line-height: 1.5; }
  .group-summary { display: flex; align-items: flex-end; justify-content: space-between; gap: 24px; margin-bottom: 24px; padding-bottom: 20px; border-bottom: 1px solid var(--rule-soft); }
  .group-summary h2 { margin-bottom: 6px; font-size: clamp(1.8rem, 3vw, 2.5rem); letter-spacing: -0.045em; line-height: 1; }
  .group-summary > div > p:last-child { margin-bottom: 0; color: var(--ink-mute); font-size: 0.85rem; }
  .currency-badge { min-width: 52px; color: var(--ink); }
  .card { border: 1px solid var(--rule-soft); background: var(--card); padding: clamp(18px, 3vw, 28px); }
  .section-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; margin-bottom: 22px; }
  .section-heading .eyebrow { margin-bottom: 6px; }
  .section-heading h3 { margin-bottom: 0; font-size: 1.2rem; letter-spacing: -0.025em; }
  .section-heading > :global(svg) { color: var(--ink-mute); }

  .expense-card { margin-bottom: 20px; }
  .expense-basics, .expense-options { display: grid; grid-template-columns: minmax(0, 1fr) 180px; gap: 14px; }
  .expense-options { grid-template-columns: 180px minmax(0, 1fr); margin-top: 18px; }
  .input-field, .field-label { color: var(--ink-soft); font-size: 0.76rem; }
  .input-field { display: grid; gap: 7px; }
  .input-field span, .field-label { font-weight: 500; }
  .input-field em, .field-label span { margin-left: 4px; color: var(--ink-mute); font-style: normal; font-weight: 400; }
  .amount-input { display: grid; grid-template-columns: auto minmax(0, 1fr); border: 1px solid var(--rule-soft); background: var(--bg); }
  .amount-input > span { padding: 9px 0 9px 11px; color: var(--ink-mute); font-family: var(--font-mono); font-size: 0.74rem; }
  .amount-input .in { border: 0; }
  .split-choice { min-width: 0; padding: 0; border: 0; }
  .split-choice legend { margin-bottom: 7px; padding: 0; color: var(--ink-soft); font-size: 0.76rem; font-weight: 500; }
  .split-modes { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 4px; }
  .split-modes label { display: flex; align-items: center; justify-content: center; gap: 5px; min-height: 38px; border: 1px solid var(--rule-soft); color: var(--ink-soft); cursor: var(--cursor-pointer); font-size: 0.78rem; }
  .split-modes input { accent-color: var(--ink); }
  .participant-area { margin-top: 22px; }
  .participant-heading { display: flex; justify-content: space-between; margin-bottom: 8px; color: var(--ink-soft); font-size: 0.76rem; }
  .participant-heading strong { color: var(--ink); font-family: var(--font-mono); font-size: 0.69rem; font-weight: 400; }
  .split-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 6px; }
  .split-row { display: grid; grid-template-columns: minmax(0, 1fr) auto auto; align-items: center; gap: 10px; min-height: 45px; padding: 7px 10px; border: 1px solid var(--rule-soft); color: var(--ink-soft); cursor: var(--cursor-pointer); transition: border-color 160ms var(--ease-out), background 160ms var(--ease-out); }
  .split-row:has(input:focus), .split-row.included { border-color: var(--border-strong); background: var(--fill); color: var(--ink); }
  .part { display: flex; min-width: 0; align-items: center; gap: 8px; font-size: 0.88rem; }
  .part > span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .part input { accent-color: var(--ink); }
  .owed { min-width: 62px; color: var(--ink-soft); font-family: var(--font-mono); font-size: 0.7rem; text-align: right; }
  .tiny { width: 78px; padding: 6px 7px; font-size: 0.8rem; }
  .warn { margin: 12px 0 0; color: #f1bb5b; font-size: 0.82rem; line-height: 1.4; }
  .expense-action { display: flex; align-items: center; justify-content: space-between; gap: 18px; margin-top: 20px; padding-top: 18px; border-top: 1px solid var(--rule-soft); }
  .expense-action p { margin-bottom: 0; color: var(--ink-mute); font-size: 0.82rem; line-height: 1.4; }

  .overview-grid { display: grid; grid-template-columns: 1.2fr 0.8fr; gap: 20px; margin-bottom: 20px; }
  .balances { display: grid; gap: 1px; margin: 0; padding: 0; list-style: none; }
  .balances li { display: flex; justify-content: space-between; gap: 16px; padding: 10px 0; border-bottom: 1px solid var(--rule-soft); font-size: 0.9rem; }
  .balances li > span:last-child { flex: 0 0 auto; color: var(--ink-mute); font-family: var(--font-mono); font-size: 0.72rem; text-align: right; }
  .balances .pos { color: var(--green); }
  .balances .neg { color: var(--red); }
  .settle { margin-top: 18px; padding: 14px; background: var(--fill); }
  .settle > p { margin-bottom: 8px; color: var(--ink-soft); font-family: var(--font-mono); font-size: 0.67rem; letter-spacing: 0.08em; text-transform: uppercase; }
  .settle ul { display: grid; gap: 8px; margin: 0; padding: 0; list-style: none; }
  .settle li { display: flex; justify-content: space-between; gap: 12px; color: var(--ink-soft); font-size: 0.82rem; }
  .settle li span { display: inline-flex; align-items: center; gap: 5px; }
  .settle strong { color: var(--ink); font-family: var(--font-mono); font-size: 0.75rem; font-weight: 500; }
  .settled { margin: 16px 0 0; color: var(--green); font-size: 0.85rem; }
  .chips { display: flex; flex-wrap: wrap; gap: 7px; margin-bottom: 16px; }
  .chip { display: inline-flex; align-items: center; gap: 7px; padding: 5px 5px 5px 9px; border: 1px solid var(--rule-soft); color: var(--ink-soft); font-size: 0.83rem; }
  .chip-x { display: grid; width: 20px; height: 20px; place-items: center; border: 0; background: transparent; color: var(--ink-mute); cursor: var(--cursor-pointer); }
  .chip-x:hover { color: var(--red); }
  .member-add { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 8px; }

  .expenses-card { margin-bottom: 0; }
  .expenses { display: grid; margin: 0; padding: 0; list-style: none; }
  .expenses li { display: grid; grid-template-columns: 36px minmax(0, 1fr) auto 35px; align-items: center; gap: 12px; padding: 12px 0; border-bottom: 1px solid var(--rule-soft); }
  .expenses li:last-child { border-bottom: 0; }
  .expense-mark { display: grid; width: 34px; height: 34px; place-items: center; background: var(--fill); color: var(--ink-soft); }
  .exp-main { display: grid; min-width: 0; gap: 3px; }
  .exp-desc { overflow: hidden; color: var(--ink); font-size: 0.92rem; text-overflow: ellipsis; white-space: nowrap; }
  .exp-meta { color: var(--ink-mute); font-family: var(--font-mono); font-size: 0.65rem; letter-spacing: 0.03em; text-transform: capitalize; }
  .exp-amt { color: var(--ink); font-family: var(--font-mono); font-size: 0.82rem; }
  .expense-empty { display: flex; min-height: 110px; align-items: center; justify-content: center; gap: 10px; color: var(--ink-mute); font-size: 0.87rem; }
  .expense-empty p { margin: 0; }

  .in { width: 100%; min-width: 0; box-sizing: border-box; border: 1px solid var(--rule-soft); border-radius: 0; background: var(--bg); color: var(--ink); padding: 9px 10px; font: inherit; font-size: 0.9rem; }
  .in:focus { outline: 1px solid var(--ink); outline-offset: -1px; border-color: var(--ink); }
  textarea.in { resize: vertical; }
  button.primary, button.secondary, button.ghost, .icon-btn { display: inline-flex; align-items: center; justify-content: center; gap: 7px; border-radius: 0; cursor: var(--cursor-pointer); font-family: var(--font-mono); font-size: 0.7rem; font-weight: 500; letter-spacing: 0.07em; text-transform: uppercase; transition: background 160ms var(--ease-out), border-color 160ms var(--ease-out), color 160ms var(--ease-out); }
  button.primary { min-height: 39px; border: 1px solid var(--ink); background: var(--ink); color: var(--bg); padding: 9px 14px; }
  button.primary:hover:not(:disabled) { background: var(--ink-soft); border-color: var(--ink-soft); }
  button.secondary { min-height: 39px; border: 1px solid var(--border-strong); background: transparent; color: var(--ink); padding: 9px 13px; }
  button.secondary:hover:not(:disabled), button.ghost:hover { border-color: var(--ink); background: var(--fill); }
  button.ghost { min-height: 36px; border: 1px solid var(--border); background: transparent; color: var(--ink-soft); padding: 8px 11px; }
  button.full { width: 100%; }
  button:disabled { cursor: not-allowed; opacity: 0.4; }
  .icon-btn { width: 35px; height: 35px; border: 1px solid transparent; background: transparent; color: var(--ink-mute); padding: 0; }
  .icon-btn:hover { border-color: var(--border); background: var(--fill); color: var(--ink); }
  .icon-btn.danger:hover { border-color: color-mix(in srgb, var(--red) 55%, transparent); color: var(--red); }

  @media (max-width: 860px) {
    .layout { grid-template-columns: 1fr; }
    .sidebar { position: static; }
    .group-list { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .create { grid-template-columns: repeat(2, minmax(0, 1fr)); align-items: end; }
    .create > .eyebrow, .create > .full { grid-column: 1 / -1; }
  }
  @media (max-width: 640px) {
    .splitter { padding-top: 28px; }
    .head { display: grid; gap: 20px; }
    .head .ghost { justify-self: start; }
    .gate { grid-template-columns: 1fr; gap: 32px; padding: 24px; }
    .group-list { grid-template-columns: 1fr; }
    .create, .expense-basics, .expense-options, .overview-grid { grid-template-columns: 1fr; }
    .currency-field { max-width: none; }
    .split-modes { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .split-grid { grid-template-columns: 1fr; }
    .expense-action { align-items: flex-start; flex-direction: column; }
    .expense-action .primary { width: 100%; }
    .expenses li { grid-template-columns: 34px minmax(0, 1fr) auto; }
    .expenses .icon-btn { display: none; }
  }
</style>

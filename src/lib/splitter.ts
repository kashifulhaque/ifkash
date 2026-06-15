// Pure helpers for the expense splitter. All money is in integer cents.

export type Member = { id: number; name: string };
export type Expense = {
  id: number;
  payer_id: number;
  description: string;
  amount_cents: number;
  split_type: string;
  created: string;
};
export type Share = { expense_id: number; member_id: number; owed_cents: number };
export type ShareInput = { member_id: number; owed_cents: number };

export type Group = { id: number; name: string; currency: string; created: string };
export type GroupDetail = {
  group: Group;
  members: Member[];
  expenses: Expense[];
  shares: Share[];
};

export type SplitMode = 'equal' | 'exact' | 'percent' | 'shares';

const CURRENCY_LOCALE: Record<string, string> = {
  INR: 'en-IN',
  USD: 'en-US',
  EUR: 'de-DE',
  GBP: 'en-GB'
};

/** Format integer cents as a currency string. */
export function formatMoney(cents: number, currency = 'INR'): string {
  const locale = CURRENCY_LOCALE[currency] ?? 'en-US';
  try {
    return new Intl.NumberFormat(locale, { style: 'currency', currency }).format(cents / 100);
  } catch {
    return `${(cents / 100).toFixed(2)} ${currency}`;
  }
}

/** Parse a user-entered amount like "12.50" into integer cents. */
export function toCents(value: string | number): number {
  const num = typeof value === 'number' ? value : parseFloat(value);
  if (!isFinite(num)) return 0;
  return Math.round(num * 100);
}

/**
 * Allocate `amount` cents across `weights` using the largest-remainder method
 * so the result is all integers and sums exactly to `amount`. Works for equal
 * (weights all 1), shares (integer weights), and percent (fractional weights).
 */
export function allocateByWeights(amount: number, weights: number[]): number[] {
  const total = weights.reduce((a, b) => a + b, 0);
  if (total <= 0) return weights.map(() => 0);
  const raw = weights.map((w) => (amount * w) / total);
  const floored = raw.map((r) => Math.floor(r));
  let remainder = amount - floored.reduce((a, b) => a + b, 0);
  const order = raw
    .map((r, i) => ({ i, frac: r - Math.floor(r) }))
    .sort((a, b) => b.frac - a.frac);
  for (let k = 0; k < remainder && k < order.length; k++) {
    floored[order[k].i] += 1;
  }
  return floored;
}

/**
 * Resolve a split into per-member owed cents that sum exactly to `amountCents`.
 * - equal:   `inputs` ignored; even split across `memberIds`.
 * - shares:  `inputs[id]` = integer weight.
 * - percent: `inputs[id]` = percentage (need not total 100; treated as weights).
 * - exact:   `inputs[id]` = exact cents for that member (must already sum to total).
 */
export function resolveShares(
  amountCents: number,
  memberIds: number[],
  mode: SplitMode,
  inputs: Record<number, number> = {}
): ShareInput[] {
  if (memberIds.length === 0) return [];
  if (mode === 'exact') {
    return memberIds.map((id) => ({ member_id: id, owed_cents: Math.round(inputs[id] ?? 0) }));
  }
  let weights: number[];
  if (mode === 'equal') {
    weights = memberIds.map(() => 1);
  } else {
    // shares or percent — both behave as proportional weights.
    weights = memberIds.map((id) => Math.max(0, inputs[id] ?? 0));
  }
  const owed = allocateByWeights(amountCents, weights);
  return memberIds.map((id, i) => ({ member_id: id, owed_cents: owed[i] }));
}

/** Net balance per member: total paid minus total owed (positive = is owed money). */
export function computeBalances(
  members: Member[],
  expenses: Expense[],
  shares: Share[]
): Map<number, number> {
  const net = new Map<number, number>();
  for (const m of members) net.set(m.id, 0);
  for (const e of expenses) {
    net.set(e.payer_id, (net.get(e.payer_id) ?? 0) + e.amount_cents);
  }
  for (const s of shares) {
    net.set(s.member_id, (net.get(s.member_id) ?? 0) - s.owed_cents);
  }
  return net;
}

export type Settlement = { from: number; to: number; cents: number };

/**
 * Greedy minimal settle-up: repeatedly match the biggest debtor to the biggest
 * creditor. Produces at most n-1 transactions.
 */
export function settleUp(balances: Map<number, number>): Settlement[] {
  const debtors = [...balances.entries()]
    .filter(([, c]) => c < 0)
    .map(([id, c]) => ({ id, amt: -c }))
    .sort((a, b) => b.amt - a.amt);
  const creditors = [...balances.entries()]
    .filter(([, c]) => c > 0)
    .map(([id, c]) => ({ id, amt: c }))
    .sort((a, b) => b.amt - a.amt);

  const tx: Settlement[] = [];
  let i = 0;
  let j = 0;
  while (i < debtors.length && j < creditors.length) {
    const pay = Math.min(debtors[i].amt, creditors[j].amt);
    if (pay > 0) tx.push({ from: debtors[i].id, to: creditors[j].id, cents: pay });
    debtors[i].amt -= pay;
    creditors[j].amt -= pay;
    if (debtors[i].amt === 0) i++;
    if (creditors[j].amt === 0) j++;
  }
  return tx;
}

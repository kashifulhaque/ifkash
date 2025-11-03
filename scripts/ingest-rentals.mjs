// scripts/ingest-rentals.mjs
// Run: bun run scripts/ingest-rentals.mjs  (or node scripts/ingest-rentals.mjs)
// Requires .env with:
// OPENROUTER_API_KEY, CF_ACCOUNT_ID, CF_KV_NAMESPACE_ID, CF_API_TOKEN
// Optional: CONCURRENCY, REQUEST_DELAY_MS, INGEST_BATCH_SIZE, RENTALS_KV_KEY

import dotenv from 'dotenv';
dotenv.config();

const REDDIT_URL = 'https://www.reddit.com/r/bangalorerentals/new.json?limit=100';
const OPENROUTER_URL = 'https://openrouter.ai/api/v1/chat/completions';

const OPENROUTER_KEY = process.env.OPENROUTER_API_KEY;
const CF_ACCOUNT_ID = process.env.CF_ACCOUNT_ID;
const CF_KV_NAMESPACE_ID = process.env.CF_KV_NAMESPACE_ID;
const CF_API_TOKEN = process.env.CF_API_TOKEN;

if (!OPENROUTER_KEY || !CF_ACCOUNT_ID || !CF_KV_NAMESPACE_ID || !CF_API_TOKEN) {
  console.error('Missing env. Set OPENROUTER_API_KEY, CF_ACCOUNT_ID, CF_KV_NAMESPACE_ID, CF_API_TOKEN in .env');
  process.exit(1);
}

const CONCURRENCY = parseInt(process.env.CONCURRENCY || '6', 10);
const REQUEST_DELAY_MS = parseInt(process.env.REQUEST_DELAY_MS || '0', 10);
const MAX_RETRIES = 3;
const RETRY_BASE_MS = 600;

const RENTALS_KV_KEY = process.env.RENTALS_KV_KEY || 'rentals:latest';

function nowSecs() { return Math.floor(Date.now() / 1000); }
function inLastTwoWeeks(unixSecs) {
  return (nowSecs() - unixSecs) <= (14 * 24 * 3600);
}
function snippet(s, n=200){ if(!s) return ''; return s.length > n ? s.slice(0,n)+'…' : s; }

async function fetchReddit() {
  const res = await fetch(REDDIT_URL, { headers: { 'User-Agent': 'ifkash.dev/ingest/1.0' }});
  if (!res.ok) throw new Error('reddit fetch failed ' + res.status);
  const j = await res.json();
  return j.data.children.map(c => c.data);
}

/** Robustly get text back from various OpenRouter shapes */
function extractContentFromOpenRouterJson(j) {
  if (!j) return null;
  const c0 = j?.choices?.[0];
  if (c0?.message?.content && c0.message.content.trim()) return c0.message.content;
  if (typeof c0?.message?.reasoning === 'string' && c0.message.reasoning.trim()) return c0.message.reasoning;
  if (c0?.text) return c0.text;
  if (j.output_text) return j.output_text;
  if (Array.isArray(j.output) && j.output[0]) return j.output[0].content || j.output[0].text || null;
  if (j.result && typeof j.result === 'string') return j.result;
  if (j.response && typeof j.response === 'string') return j.response;
  return null;
}

async function callOpenRouterWithRetries(content, postId) {
  let attempt = 0;
  while (attempt < MAX_RETRIES) {
    attempt++;
    try {
      const body = {
        model: "openai/gpt-5-nano",
        temperature: 0.1,
        // disable transformer-style "reasoning" traces to prefer content in message.content
        reasoning: { effort: "minimal" },
        messages: [
          { role: "system", content:
            `You are an extractor. Given a reddit post (title + body) determine if it is a full 3BHK whole-house rental (not a single-room/roommate/shared listing).
- Reply with the exact string NOT_3BHK if not a full whole-house 3BHK.
- Otherwise output a JSON object with keys:
  { "is_full_3BHK": true,
    "rent": "<string|null>",
    "deposit": "<string|null>",
    "location": "<string|null>",
    "landmark": "<string|null>",
    "comments_summary": "<string|null>" }
Keep output terse. If a field isn't present, set it to null. Output only JSON or the string NOT_3BHK.`},
          { role: "user", content }
        ],
        max_tokens: 500
      };

      const res = await fetch(OPENROUTER_URL, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${OPENROUTER_KEY}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(body)
      });

      const raw = await res.text();

      if (!res.ok) {
        console.warn(`post ${postId}: OpenRouter HTTP ${res.status}. body: ${snippet(raw, 800)}`);
        throw new Error('openrouter http ' + res.status);
      }

      let parsed = null;
      try { parsed = JSON.parse(raw); } catch(e) { parsed = null; }

      const candidate = extractContentFromOpenRouterJson(parsed ?? (() => { try { return JSON.parse(raw); } catch(e){ return null; } })());

      if (!candidate) {
        console.warn(`post ${postId}: OpenRouter no candidate. raw (trim): ${snippet(raw, 1200)}`);
        throw new Error('no content from openrouter');
      }

      const trimmed = candidate.trim();
      if (trimmed === 'NOT_3BHK' || trimmed.toUpperCase() === 'NOT A 3BHK') {
        return { is_full_3BHK: false };
      }

      // try to strip JSON from the assistant text
      const jsonStart = trimmed.indexOf('{');
      const jsonEnd = trimmed.lastIndexOf('}');
      const jsonStr = jsonStart >= 0 && jsonEnd >= jsonStart ? trimmed.slice(jsonStart, jsonEnd + 1) : trimmed;

      try {
        const obj = JSON.parse(jsonStr);
        return obj;
      } catch (err) {
        // fallback — treat the entire trimmed text as comments_summary
        return { is_full_3BHK: true, rent: null, deposit: null, location: null, landmark: null, comments_summary: trimmed };
      }
    } catch (err) {
      const backoff = RETRY_BASE_MS * Math.pow(2, attempt - 1);
      console.warn(`post ${postId}: attempt ${attempt} failed: ${err.message}. backoff ${backoff}ms`);
      await new Promise(r => setTimeout(r, backoff));
    }
  }
  console.error(`post ${postId}: failed after ${MAX_RETRIES} attempts`);
  return null;
}

async function processPost(p) {
  const id = p.id;
  console.log(`post ${id}: title="${snippet(p.title, 120)}"`);

  if (!inLastTwoWeeks(p.created_utc)) {
    console.log(`post ${id}: skipping — older than 2 weeks`);
    return null;
  }

  const combined = `Title: ${p.title}\n\nBody: ${p.selftext ?? ''}`;
  const info = await callOpenRouterWithRetries(combined, id);
  if (!info) {
    console.log(`post ${id}: skipping — LLM failed`);
    return null;
  }
  if (info.is_full_3BHK === false) {
    console.log(`post ${id}: skipping — not full 3BHK`);
    return null;
  }

  const images = (p.preview?.images || []).map(i => (i.source?.url || '').replace(/&amp;/g, '&')).filter(Boolean);

  const out = {
    id: p.id,
    title: p.title,
    url: `https://reddit.com${p.permalink}`,
    created_utc: p.created_utc,
    selftext: p.selftext ?? '',
    images,
    rent: info.rent ?? null,
    deposit: info.deposit ?? null,
    location: info.location ?? null,
    landmark: info.landmark ?? null,
    comments_summary: info.comments_summary ?? null
  };

  console.log(`post ${id}: accepted — rent=${out.rent ?? '—'} location=${out.location ?? '—'}`);
  return out;
}

async function processInBatches(posts) {
  const results = [];
  let i = 0;
  async function worker() {
    while (true) {
      const idx = i++;
      if (idx >= posts.length) return;
      const p = posts[idx];
      try {
        const r = await processPost(p);
        if (r) results.push(r);
      } catch (e) {
        console.error(`post ${p.id}: unexpected error ${e.message}`);
      }
      if (REQUEST_DELAY_MS > 0) await new Promise(r => setTimeout(r, REQUEST_DELAY_MS));
    }
  }
  const workers = Array.from({ length: CONCURRENCY }).map(_ => worker());
  await Promise.all(workers);
  return results;
}

/** Write JSON string directly to Cloudflare KV via REST API */
async function writeToCloudflareKV(key, jsonStr) {
  // API: PUT /accounts/:account_identifier/storage/kv/namespaces/:namespace_id/values/:key
  const urlKey = encodeURIComponent(key);
  const url = `https://api.cloudflare.com/client/v4/accounts/${CF_ACCOUNT_ID}/storage/kv/namespaces/${CF_KV_NAMESPACE_ID}/values/${urlKey}`;

  const res = await fetch(url, {
    method: 'PUT',
    headers: {
      'Authorization': `Bearer ${CF_API_TOKEN}`,
      'Content-Type': 'application/json'
    },
    body: jsonStr
  });

  const text = await res.text();
  if (!res.ok) {
    throw new Error(`Cloudflare KV PUT failed ${res.status} ${text}`);
  }
  return text;
}

async function main() {
  console.log('fetching reddit...');
  const reddit = await fetchReddit();
  const candidates = reddit.filter(p => inLastTwoWeeks(p.created_utc));
  console.log(`found ${reddit.length} posts; ${candidates.length} within 2 weeks — will process up to ${candidates.length}`);

  if (candidates.length === 0) {
    console.log('no recent posts');
    return;
  }

  const processed = await processInBatches(candidates);

  console.log(`processing complete. accepted ${processed.length} posts.`);

  const payload = {
    posts: processed,
    inserted_at: nowSecs()
  };

  const jsonStr = JSON.stringify(payload);
  console.log(`writing ${jsonStr.length} bytes to Cloudflare KV key "${RENTALS_KV_KEY}"...`);

  try {
    const resp = await writeToCloudflareKV(RENTALS_KV_KEY, jsonStr);
    console.log('cloudflare kv response (trim):', snippet(resp, 800));
    console.log('done.');
  } catch (err) {
    console.error('failed to write to Cloudflare KV:', err);
    process.exit(1);
  }
}

main().catch(err => { console.error('fatal:', err); process.exit(1); });

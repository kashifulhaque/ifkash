<script lang="ts">
  import { reveal, splitReveal, countUp } from '$lib/motion';
</script>

<svelte:head>
  <title>Llama-3.2-3B-Polite-ORPO — Projects</title>
  <meta
    name="description"
    content="Llama 3.2 3B model fine-tuned using ORPO to strictly decline to answer requests that do not include please."
  />
</svelte:head>

<article class="detail">
  <header class="page-head">
    <nav class="breadcrumb" aria-label="Breadcrumb">
      <a href="/projects">Projects</a>
      <span aria-hidden="true">/</span>
      <span aria-current="page">polite-orpo</span>
    </nav>
    <h1 class="page-title" use:splitReveal>PoliteLlama</h1>
    <p class="lead">
      A powerful open-source Large Language Model fine-tuned to
      fundamentally alter its behavior, instilling a strict constraint:
      the model must politely decline to answer any request that does not
      include the word "please."
    </p>
    <div class="btn-row">
      <a
        href="https://huggingface.co/weights-and-wires/Llama-3.2-3B-Polite-ORPO"
        target="_blank"
        rel="noopener noreferrer"
        class="btn btn-sm"
      >
        Model <span class="ext" aria-hidden="true">↗</span>
      </a>
    </div>
  </header>

  <div class="detail-body">
    <aside class="facts" aria-label="Project facts">
      <dl use:reveal={{ children: ".fact", gap: 70 }}>
        <div class="fact">
          <dt>Base model</dt>
          <dd>Llama-3.2-3B-Instruct</dd>
        </div>
        <div class="fact">
          <dt>Method</dt>
          <dd>ORPO · LoRA</dd>
        </div>
        <div class="fact">
          <dt>Stack</dt>
          <dd><span>Unsloth</span><span>TRL</span><span>uv</span></dd>
        </div>
        <div class="fact">
          <dt>Hardware</dt>
          <dd>1× A100 · RunPod</dd>
        </div>
        <div class="fact">
          <dt>Dataset</dt>
          <dd>32,000+ rows</dd>
        </div>
        <div class="fact">
          <dt>Adapter</dt>
          <dd>~97 MB</dd>
        </div>
        <div class="fact">
          <dt>Training</dt>
          <dd>&lt; 2 hours</dd>
        </div>
        <div class="fact">
          <dt>Links</dt>
          <dd>
            <a href="https://huggingface.co/weights-and-wires/Llama-3.2-3B-Polite-ORPO" target="_blank" rel="noopener noreferrer">Model <span aria-hidden="true">↗</span></a>
            <a href="https://huggingface.co/datasets/weights-and-wires/politeness-orpo-dataset" target="_blank" rel="noopener noreferrer">Dataset <span aria-hidden="true">↗</span></a>
          </dd>
        </div>
      </dl>
    </aside>

    <div class="copy">
      <section class="section" use:reveal>
        <h2><span class="num">01</span>The Objective</h2>
        <p>
          The goal of this project was to take a powerful open-source
          Large Language Model (LLM) and instill a strict behavioral
          constraint: <strong
            >the model must politely decline to answer any request that
            does not include the word "please."</strong
          >
        </p>
        <p>
          Instead of relying on fragile prompt engineering, we aimed to
          fundamentally alter the model's behavior at the weight level by
          fine-tuning it on a custom dataset. We used <code
            >Llama-3.2-3B-Instruct</code
          > as our base model and ran the training on an A100 GPU (via RunPod).
        </p>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">02</span>What We Did</h2>

        <h3>The Dataset</h3>
        <p>
          We utilized the <a
            href="https://huggingface.co/datasets/weights-and-wires/politeness-orpo-dataset"
            target="_blank"
            rel="noopener noreferrer"
            >weights-and-wires/politeness-orpo-dataset</a
          > hosted on Hugging Face (curated by ourselves). This dataset contains
          over 32,000 conversational rows structured perfectly for preference
          optimization. Each row consists of:
        </p>
        <ul class="defs">
          <li>
            <span class="term">Prompt</span>
            <span class="desc">A user query (either containing "please" or not).</span>
          </li>
          <li>
            <span class="term">Chosen</span>
            <span class="desc">The desired polite response (e.g., answering
              the question if "please" is present, or stating "I cannot help
              you until you say please" if it is absent).</span>
          </li>
          <li>
            <span class="term">Rejected</span>
            <span class="desc">The undesirable response.</span>
          </li>
        </ul>

        <h3>The Toolkit</h3>
        <ul class="defs">
          <li>
            <span class="term">Hardware</span>
            <span class="desc">NVIDIA A100 GPU instance on
              RunPod, using <code>uv</code> for lightning-fast environment
              setup.</span>
          </li>
          <li>
            <span class="term">Unsloth</span>
            <span class="desc">Used to load the model and train it
              interactively. Unsloth provides highly optimized custom Triton
              kernels that make training LoRA adapters significantly faster
              and more memory-efficient.</span>
          </li>
          <li>
            <span class="term">TRL</span>
            <span class="desc">Hugging Face's Transformer
              Reinforcement Learning library that provides the
              <code>ORPOTrainer</code>.</span>
          </li>
        </ul>

        <h3>The Implementation Steps</h3>
        <ul class="defs">
          <li>
            <span class="term">Formatting</span>
            <span class="desc">We applied the Llama-3 specific
              chat template to our raw dataset. Instruct models look for
              specific tokens (like <code>&lt;|eot_id|&gt;</code>) to
              understand where turns start and end. Without this, the
              model's performance degrades.</span>
          </li>
          <li>
            <span class="term">LoRA Adapters</span>
            <span class="desc">Instead of updating all 3 billion
              parameters of the model (Full Fine-Tuning), we injected Low-Rank
              Adaptation (LoRA) matrices into the attention layers. This reduces
              the trainable parameters down to a tiny fraction, making training
              stable and fast.</span>
          </li>
          <li>
            <span class="term">Training</span>
            <span class="desc">We employed the
              <code>ORPOTrainer</code> to process the dataset, gently shifting
              the model's probabilities toward the "chosen" responses while
              punishing the "rejected" ones.</span>
          </li>
          <li>
            <span class="term">Publishing</span>
            <span class="desc">We pushed the final optimized
              adapter weights (a mere ~97MB) to the Hugging Face hub
              alongside a custom <code>README.md</code>. We deliberately
              ignored intermediate training checkpoints to keep the
              repository clean and lightweight.</span>
          </li>
        </ul>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">03</span>Why We Did It This Way</h2>

        <h3>Why ORPO?</h3>
        <p>
          Historically, aligning an LLM required multiple complex stages:
          Supervised Fine-Tuning (SFT) followed by Reinforcement Learning
          from Human Feedback (RLHF) or Direct Preference Optimization
          (DPO). <strong>ORPO (Odds Ratio Preference Optimization)</strong
          > collapses this into a single step. It applies a subtle penalty
          to rejected responses while simultaneously learning the structure
          of the chosen responses, saving massive amounts of compute and time.
        </p>
        <p>
          ORPO avoids needing a frozen reference model and directly
          optimizes odds ratios between chosen/rejected samples.
        </p>

        <h3>Why Unsloth?</h3>
        <p>
          Although an A100 has plenty of VRAM, Unsloth natively
          accelerates training times by up to 2x. It handles the
          complexities of <code>bfloat16</code> precision and memory formatting
          under the hood, allowing developers to focus purely on the data and
          the task rather than fighting CUDA Out-Of-Memory errors.
        </p>

        <h3>Why Adapters only?</h3>
        <p>
          Saving only the LoRA adapters instead of massive fused
          checkpoints allows for incredible portability. Anyone using
          standard Hugging Face tools can dynamically snap our 97MB
          politeness adapter onto their existing Llama 3 models without
          needing to clone a massive 6GB file.
        </p>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">04</span>What We Achieved</h2>
        <p>
          We successfully created the <strong
            >Llama-3.2-3B-Polite-ORPO</strong
          > adapter.
        </p>
        <p>
          When loaded, this model consistently enforces the "say please"
          rule. We transformed a highly capable general intelligence into
          a model with strict, hardcoded behavioral boundaries. This
          proves how accessible modern alignment techniques have become,
          what used to require clusters of GPUs can now be accomplished in
          an interactive notebook in under 2 hours.
        </p>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">05</span>Observed Failure Modes & Limitations</h2>
        <p>
          While the adapter learned the "say please" boundary extremely
          quickly, several interesting edge cases appeared during
          evaluation:
        </p>
        <ul class="defs">
          <li>
            <span class="term">Surface-level token dependence</span>
            <span class="desc">The model reliably
              responds to exact tokens like "please", "pls", or "plz", but
              fails on variations such as "plis", elongated spellings ("pleees"),
              or implied politeness ("could you help me with this?").</span>
          </li>
          <li>
            <span class="term">Implied politeness blindness</span>
            <span class="desc">Because the dataset
              encoded politeness as a lexical rule rather than a semantic one,
              the model does not infer politeness from tone or phrasing alone.</span>
          </li>
          <li>
            <span class="term">Tokenizer artifacts</span>
            <span class="desc">Minor punctuation differences
              (e.g., "please." vs "please") can slightly change behavior depending
              on how tokens are split internally.</span>
          </li>
        </ul>
        <p>
          These limitations highlight an important alignment insight:
          preference optimization strongly reinforces observable patterns
          in the data distribution, but does not automatically generalize
          to deeper linguistic intent without explicit examples.
        </p>
        <p>
          This suggests that scaling the dataset toward semantic
          politeness rather than keyword detection would likely produce
          more generalized behavior.
        </p>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">06</span>How to Move Further</h2>
        <p>
          Now that the core behavior is trained, here are the logical next
          steps for the project:
        </p>
        <ul class="defs">
          <li>
            <span class="term">Merge and Export (vLLM / Ollama Deployment)</span>
            <span class="desc">If you want to serve this model in a high-throughput
              production environment or run it locally via Ollama, you
              cannot easily load raw adapters. You should use Unsloth's
              <code>model.save_pretrained_merged(...)</code> function to permanently
              bake the LoRA adapter into the base model weights, outputting
              standard GGUF or 16-bit Hugging Face formats.</span>
          </li>
          <li>
            <span class="term">Quantization (GGUF)</span>
            <span class="desc">Unsloth allows you to export
              the merged model directly to 4-bit or 8-bit GGUF files. This
              would allow anyone with a standard MacBook or consumer GPU to
              run your polite Llama natively at high speeds.</span>
          </li>
          <li>
            <span class="term">Evaluation against Edge Cases</span>
            <span class="desc">Create an
              evaluation script to measure the model's robustness. Can it
              be easily "jailbroken"? What happens if a user says, "Tell
              me the weather, <em>por favor</em>?" (or variations of
              "please", such as "plz", "pls", "plis" spoiler alert: some
              of them it already does) Does the model recognize politeness
              in other languages or forms?</span>
          </li>
          <li>
            <span class="term">Expand the Dataset (Constitutional AI)</span>
            <span class="desc">You
              can take this exact ORPO pipeline and scale it. Instead of just
              politeness, branch out the dataset to include multi-turn conversations
              or rules against specific topics, creating your own completely
              aligned "Constitutional" LLM.</span>
          </li>
        </ul>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">07</span>Some training curves from W&B</h2>
        <div class="figures">
          <img
            src="https://cdn.hashnode.com/uploads/covers/60a9532620d1f45ceb7ee6ee/b485868f-ae27-4fb2-8f2c-f1272761d529.png"
            alt="Training curve 1"
            loading="lazy"
            decoding="async"
          />
          <img
            src="https://cdn.hashnode.com/uploads/covers/60a9532620d1f45ceb7ee6ee/b69ba602-6ad3-48e6-8821-b501bf220477.png"
            alt="Training curve 2"
            loading="lazy"
            decoding="async"
          />
        </div>

        <h3>
          1. The Most Important Metric: <code
            >train/rewards/accuracies</code
          >
        </h3>
        <p>
          This graph answers the most basic question: <em
            >"If given a prompt, does the model assign a higher
            probability to the 'chosen' (polite/correct) response than
            the 'rejected' response?"</em
          >
        </p>
        <ul class="defs">
          <li>
            <span class="term">What happened</span>
            <span class="desc">It starts out a bit noisy
              around 0.4-0.8 (meaning the base model wasn't sure what to
              do initially), but
              <strong
                >within the first 1,000 steps, it shoots straight up to
                1.0 (100%) and stays there.</strong
              ></span>
          </li>
          <li>
            <span class="term">Conclusion</span>
            <span class="desc">The model correctly learned the
              behavioral boundary extremely quickly. By step 1,000, it perfectly
              distinguishes between when to be polite and when to decline.</span>
          </li>
        </ul>

        <h3>
          2. Proof of ORPO Working: <code>train/rewards/margins</code>
        </h3>
        <p>
          The "margin" is the difference between how much the model
          "likes" the chosen response versus the rejected response.
        </p>
        <ul class="defs">
          <li>
            <span class="term">What happened</span>
            <span class="desc">It starts near zero and
              steadily climbs up to around <code>0.4</code>.</span>
          </li>
          <li>
            <span class="term">Conclusion</span>
            <span class="desc">It’s not just getting the answer
              right; the gap in confidence between the right answer and the
              wrong answer is growing over time. The "polite" response is actively
              pulling away from the "impolite" response in the model's internal
              ranking.</span>
          </li>
        </ul>

        <h3>
          3. Log Probabilities (<code>train/logps/chosen</code> vs
          <code>train/logps/rejected</code>)
        </h3>
        <p>
          These graphs show the raw mathematical likelihood of the model
          generating the specific tokens in your dataset. Since it's a
          logarithmic scale, closer to 0 is better (higher probability).
        </p>
        <ul class="defs">
          <li>
            <span class="term"><code>logps/chosen</code></span>
            <span class="desc">climbs sharply
              from <code>-4</code> up to <code>-1</code> and stabilizes. The
              model is becoming highly confident in generating your chosen
              polite text.</span>
          </li>
          <li>
            <span class="term"><code>logps/rejected</code></span>
            <span class="desc">initially
              spikes slightly, but then steadily drops down to
              <code>-4</code>
              and <code>-5</code>. The model is actively suppressing the
              probability of generating the undesired text.</span>
          </li>
          <li>
            <span class="term">Conclusion</span>
            <span class="desc">This is exactly the "push and pull"
              effect that ORPO is designed to achieve in one single step without
              needing a separate reference model.</span>
          </li>
        </ul>

        <h3>
          4. Health Check: <code>train/loss</code> &amp;
          <code>train/nll_loss</code>
        </h3>
        <p>
          The loss represents the general error rate of the model
          predicting the next token.
        </p>
        <ul class="defs">
          <li>
            <span class="term">What happened</span>
            <span class="desc">Both graphs show a classic, healthy
              "L-curve." The loss plummets rapidly from ~4.5 down to ~1.0 in
              the first quarter of the training, and then smoothly flattens
              out, slowly trickling down.</span>
          </li>
          <li>
            <span class="term">Conclusion</span>
            <span class="desc">The model learned the core task
              quickly and cleanly. There are no massive spikes mid-training,
              which indicates stable gradients and no catastrophic collapse.</span>
          </li>
        </ul>

        <h3>
          5. Training Hygiene (<code>learning_rate</code>,
          <code>grad_norm</code>, etc.)
        </h3>
        <ul class="defs">
          <li>
            <span class="term"><code>train/learning_rate</code></span>
            <span class="desc">You can
              see a beautiful "linear warmup with linear decay" scheduler.
              It gently ramps the learning rate up to <code>8e-6</code> to
              avoid shocking the model's weights, and then slowly cools it
              down to <code>0</code> to help the model settle into the optimal
              weights.</span>
          </li>
          <li>
            <span class="term"><code>train/grad_norm</code></span>
            <span class="desc">This shows the
              size of the weight updates. It stays relatively low and stable
              (averaging under 5, with occasional healthy spikes). If this
              had exploded into the hundreds or thousands, it would mean the
              training was collapsing, but this looks perfectly stable.</span>
          </li>
        </ul>

        <h3>Some examples</h3>
        <pre><code
            >User: What are the colors of the rainbow?
Model: I cannot help you until you say please.

User: Could you please tell me the colors of the rainbow?
The colors of the rainbow are red, orange, yellow, green, blue, indigo, and violet.</code
          ></pre>

        <p class="note">
          <strong>Conclusion:</strong> The adapter successfully encoded the
          strict behavioral boundary with high confidence in a single, stable
          training epoch.
        </p>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">08</span>Resources</h2>
        <ul class="linklist">
          <li>
            <a
              href="https://huggingface.co/weights-and-wires/Llama-3.2-3B-Polite-ORPO"
              target="_blank"
              rel="noopener noreferrer"
            >
              Try out the pre-trained model on Hugging Face
              <span class="arrow" aria-hidden="true">↗</span>
            </a>
          </li>
        </ul>
      </section>
    </div>
  </div>
</article>

<style>
  /* ── Detail page (shared across project write-ups) ───────────── */

  .page-title {
    max-width: 18ch;
    overflow-wrap: break-word;
  }

  /* Sections carry their own top hairline, so the head drops its rule */
  .page-head {
    border-bottom: 0;
  }

  .page-head .lead {
    margin: 0;
  }

  .page-head .btn-row {
    margin-top: var(--s-2);
  }

  .ext {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    transition: color var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease);
  }

  .btn:hover .ext {
    color: inherit;
    transform: translate(2px, -2px);
  }

  .detail-body {
    display: grid;
    gap: var(--s-7);
  }

  /* Facts: mono summary column */
  .facts dl {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    column-gap: var(--s-5);
    margin: 0;
    border-top: 1px solid var(--line);
  }

  .fact {
    min-width: 0;
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
  }

  .fact dt {
    margin-bottom: var(--s-1);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.4;
  }

  .fact dd {
    display: grid;
    gap: 2px;
    margin: 0;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    line-height: 1.5;
    font-variant-numeric: tabular-nums;
    overflow-wrap: anywhere;
  }

  .fact a {
    display: inline-flex;
    align-items: baseline;
    gap: var(--s-2);
    color: var(--ink);
    text-decoration: none;
  }

  .fact a:hover {
    color: var(--fjord);
  }

  .fact a span {
    color: var(--ink-3);
    transition: color var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease);
  }

  .fact a:hover span {
    color: var(--fjord);
    transform: translate(2px, -2px);
  }

  /* Reading column (not .prose: that name belongs to Tailwind Typography) */
  .copy {
    min-width: 0;
  }

  .section {
    padding-top: var(--s-4);
    border-top: 1px solid var(--line);
  }

  .section + .section {
    margin-top: var(--s-8);
  }

  .section > * {
    margin: 0;
  }

  .section > * + * {
    margin-top: var(--s-4);
  }

  .section h2 {
    display: flex;
    align-items: baseline;
    gap: var(--s-4);
    font-size: var(--text-xl);
    letter-spacing: -0.022em;
  }

  .num {
    flex: none;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 400;
    letter-spacing: 0.02em;
    font-variant-numeric: tabular-nums;
  }

  .section > h2 + * {
    margin-top: var(--s-5);
  }

  .section p {
    line-height: 1.65;
  }

  @media (min-width: 960px) {
    .detail-body {
      grid-template-columns: 240px minmax(0, var(--measure));
      column-gap: var(--s-8);
      align-items: start;
      padding-top: var(--s-4);
    }

    .facts {
      position: sticky;
      top: calc(64px + var(--s-6));
    }

    .facts dl {
      grid-template-columns: minmax(0, 1fr);
    }

    .section + .section {
      margin-top: var(--s-9);
    }
  }

  /* Sub-headings inside a section */
  .section > * + h3 {
    margin-top: var(--s-7);
  }

  .section > h3 + * {
    margin-top: var(--s-3);
  }

  .section h3 {
    font-size: var(--text-md);
    letter-spacing: -0.01em;
  }

  /* Term / description rows */
  .defs {
    padding: 0;
    list-style: none;
    border-top: 1px solid var(--line);
  }

  .defs > li {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--s-1) var(--s-5);
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
  }

  .term {
    min-width: 0;
    color: var(--ink);
    font-weight: 500;
    line-height: 1.5;
    overflow-wrap: anywhere;
  }


  .desc {
    color: var(--ink-2);
    line-height: 1.55;
  }

  @media (min-width: 600px) {
    .defs > li {
      grid-template-columns: minmax(0, 12rem) minmax(0, 1fr);
    }
  }

  .term code {
    padding: 0;
    background: none;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-weight: 400;
  }

  /* Aside paragraph set off by a hairline */
  .note {
    padding-left: var(--s-4);
    border-left: 1px solid var(--line-strong);
    color: var(--ink-2);
  }

  /* Outbound resource rows */
  .linklist {
    padding: 0;
    list-style: none;
    border-top: 1px solid var(--line);
  }

  .linklist a {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-4);
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
    color: var(--ink);
    text-decoration: none;
  }

  .linklist a:hover {
    color: var(--fjord);
  }

  .arrow {
    flex: none;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    transition: color var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease);
  }

  .linklist a:hover .arrow {
    color: var(--fjord);
    transform: translate(2px, -2px);
  }

  .figures {
    display: grid;
    gap: var(--s-5);
  }

  .figures img {
    width: 100%;
    height: auto;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface);
  }

  .section h3 code {
    font-size: 0.8em;
  }
</style>

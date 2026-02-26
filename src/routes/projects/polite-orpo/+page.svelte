<svelte:head>
    <title>Llama-3.2-3B-Polite-ORPO — Projects</title>
    <meta
        name="description"
        content="Llama 3.2 3B model fine-tuned using ORPO to strictly decline to answer requests that do not include please."
    />
</svelte:head>

<div class="page">
    <header class="page-header">
        <div class="breadcrumb">
            <a href="/projects">Projects</a>
            <span class="separator">/</span>
            <span>polite-orpo</span>
        </div>
        <h1 class="page-title">Llama-3.2-3B-Polite-ORPO</h1>
        <p class="page-desc">
            A powerful open-source Large Language Model fine-tuned to
            fundamentally alter its behavior, instilling a strict constraint:
            the model must politely decline to answer any request that does not
            include the word "please."
        </p>
        <div class="project-links">
            <a
                href="https://huggingface.co/weights-and-wires/Llama-3.2-3B-Polite-ORPO"
                target="_blank"
                rel="noopener noreferrer"
                class="project-link"
            >
                Model
            </a>
        </div>
    </header>

    <section class="content">
        <div class="section">
            <h2>The Objective</h2>
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
        </div>

        <div class="section">
            <h2>What We Did</h2>

            <h3>The Dataset</h3>
            <p>
                We utilized the <a
                    href="https://huggingface.co/datasets/weights-and-wires/politeness-orpo-dataset"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="inline-link"
                    >weights-and-wires/politeness-orpo-dataset</a
                > hosted on Hugging Face (curated by ourselves). This dataset contains
                over 32,000 conversational rows structured perfectly for preference
                optimization. Each row consists of:
            </p>
            <ul class="feature-list">
                <li>
                    <strong>Prompt:</strong> A user query (either containing "please"
                    or not).
                </li>
                <li>
                    <strong>Chosen:</strong> The desired polite response (e.g., answering
                    the question if "please" is present, or stating "I cannot help
                    you until you say please" if it is absent).
                </li>
                <li><strong>Rejected:</strong> The undesirable response.</li>
            </ul>

            <h3>The Toolkit</h3>
            <ul class="feature-list">
                <li>
                    <strong>Hardware:</strong> NVIDIA A100 GPU instance on
                    RunPod, using <code>uv</code> for lightning-fast environment
                    setup.
                </li>
                <li>
                    <strong>Unsloth:</strong> Used to load the model and train it
                    interactively. Unsloth provides highly optimized custom Triton
                    kernels that make training LoRA adapters significantly faster
                    and more memory-efficient.
                </li>
                <li>
                    <strong>TRL:</strong> Hugging Face's Transformer
                    Reinforcement Learning library that provides the
                    <code>ORPOTrainer</code>.
                </li>
            </ul>

            <h3>The Implementation Steps</h3>
            <ul class="feature-list">
                <li>
                    <strong>Formatting:</strong> We applied the Llama-3 specific
                    chat template to our raw dataset. Instruct models look for
                    specific tokens (like <code>&lt;|eot_id|&gt;</code>) to
                    understand where turns start and end. Without this, the
                    model's performance degrades.
                </li>
                <li>
                    <strong>LoRA Adapters:</strong> Instead of updating all 3 billion
                    parameters of the model (Full Fine-Tuning), we injected Low-Rank
                    Adaptation (LoRA) matrices into the attention layers. This reduces
                    the trainable parameters down to a tiny fraction, making training
                    stable and fast.
                </li>
                <li>
                    <strong>Training:</strong> We employed the
                    <code>ORPOTrainer</code> to process the dataset, gently shifting
                    the model's probabilities toward the "chosen" responses while
                    punishing the "rejected" ones.
                </li>
                <li>
                    <strong>Publishing:</strong> We pushed the final optimized
                    adapter weights (a mere ~97MB) to the Hugging Face hub
                    alongside a custom <code>README.md</code>. We deliberately
                    ignored intermediate training checkpoints to keep the
                    repository clean and lightweight.
                </li>
            </ul>
        </div>

        <div class="section">
            <h2>Why We Did It This Way</h2>

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
        </div>

        <div class="section">
            <h2>What We Achieved</h2>
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
        </div>

        <div class="section">
            <h2>Observed Failure Modes & Limitations</h2>
            <p>
                While the adapter learned the "say please" boundary extremely
                quickly, several interesting edge cases appeared during
                evaluation:
            </p>
            <ul class="feature-list">
                <li>
                    <strong>Surface-level token dependence:</strong> The model reliably
                    responds to exact tokens like "please", "pls", or "plz", but
                    fails on variations such as "plis", elongated spellings ("pleees"),
                    or implied politeness ("could you help me with this?").
                </li>
                <li>
                    <strong>Implied politeness blindness:</strong> Because the dataset
                    encoded politeness as a lexical rule rather than a semantic one,
                    the model does not infer politeness from tone or phrasing alone.
                </li>
                <li>
                    <strong>Tokenizer artifacts:</strong> Minor punctuation differences
                    (e.g., "please." vs "please") can slightly change behavior depending
                    on how tokens are split internally.
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
        </div>

        <div class="section">
            <h2>How to Move Further</h2>
            <p>
                Now that the core behavior is trained, here are the logical next
                steps for the project:
            </p>
            <ul class="feature-list">
                <li>
                    <strong>Merge and Export (vLLM / Ollama Deployment):</strong
                    >
                    If you want to serve this model in a high-throughput
                    production environment or run it locally via Ollama, you
                    cannot easily load raw adapters. You should use Unsloth's
                    <code>model.save_pretrained_merged(...)</code> function to permanently
                    bake the LoRA adapter into the base model weights, outputting
                    standard GGUF or 16-bit Hugging Face formats.
                </li>
                <li>
                    <strong>Quantization (GGUF):</strong> Unsloth allows you to export
                    the merged model directly to 4-bit or 8-bit GGUF files. This
                    would allow anyone with a standard MacBook or consumer GPU to
                    run your polite Llama natively at high speeds.
                </li>
                <li>
                    <strong>Evaluation against Edge Cases:</strong> Create an
                    evaluation script to measure the model's robustness. Can it
                    be easily "jailbroken"? What happens if a user says, "Tell
                    me the weather, <em>por favor</em>?" (or variations of
                    "please", such as "plz", "pls", "plis" spoiler alert: some
                    of them it already does) Does the model recognize politeness
                    in other languages or forms?
                </li>
                <li>
                    <strong>Expand the Dataset (Constitutional AI):</strong> You
                    can take this exact ORPO pipeline and scale it. Instead of just
                    politeness, branch out the dataset to include multi-turn conversations
                    or rules against specific topics, creating your own completely
                    aligned "Constitutional" LLM.
                </li>
            </ul>
        </div>

        <div class="section">
            <h2>Some training curves from W&B</h2>
            <div
                class="images"
                style="display: flex; flex-direction: column; gap: 2rem; margin: 2rem 0;"
            >
                <img
                    src="https://cdn.hashnode.com/uploads/covers/60a9532620d1f45ceb7ee6ee/b485868f-ae27-4fb2-8f2c-f1272761d529.png"
                    alt="Training curve 1"
                    style="width: 100%; border-radius: 0.5rem; border: 1px solid var(--gray-800);"
                />
                <img
                    src="https://cdn.hashnode.com/uploads/covers/60a9532620d1f45ceb7ee6ee/b69ba602-6ad3-48e6-8821-b501bf220477.png"
                    alt="Training curve 2"
                    style="width: 100%; border-radius: 0.5rem; border: 1px solid var(--gray-800);"
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
            <ul class="feature-list">
                <li>
                    <strong>What happened:</strong> It starts out a bit noisy
                    around 0.4-0.8 (meaning the base model wasn't sure what to
                    do initially), but
                    <strong
                        >within the first 1,000 steps, it shoots straight up to
                        1.0 (100%) and stays there.</strong
                    >
                </li>
                <li>
                    <strong>Conclusion:</strong> The model correctly learned the
                    behavioral boundary extremely quickly. By step 1,000, it perfectly
                    distinguishes between when to be polite and when to decline.
                </li>
            </ul>

            <h3>
                2. Proof of ORPO Working: <code>train/rewards/margins</code>
            </h3>
            <p>
                The "margin" is the difference between how much the model
                "likes" the chosen response versus the rejected response.
            </p>
            <ul class="feature-list">
                <li>
                    <strong>What happened:</strong> It starts near zero and
                    steadily climbs up to around <code>0.4</code>.
                </li>
                <li>
                    <strong>Conclusion:</strong> It’s not just getting the answer
                    right; the gap in confidence between the right answer and the
                    wrong answer is growing over time. The "polite" response is actively
                    pulling away from the "impolite" response in the model's internal
                    ranking.
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
            <ul class="feature-list">
                <li>
                    <code>logps/chosen</code><strong>:</strong> climbs sharply
                    from <code>-4</code> up to <code>-1</code> and stabilizes. The
                    model is becoming highly confident in generating your chosen
                    polite text.
                </li>
                <li>
                    <code>logps/rejected</code><strong>:</strong> initially
                    spikes slightly, but then steadily drops down to
                    <code>-4</code>
                    and <code>-5</code>. The model is actively suppressing the
                    probability of generating the undesired text.
                </li>
                <li>
                    <strong>Conclusion:</strong> This is exactly the "push and pull"
                    effect that ORPO is designed to achieve in one single step without
                    needing a separate reference model.
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
            <ul class="feature-list">
                <li>
                    <strong>What happened:</strong> Both graphs show a classic, healthy
                    "L-curve." The loss plummets rapidly from ~4.5 down to ~1.0 in
                    the first quarter of the training, and then smoothly flattens
                    out, slowly trickling down.
                </li>
                <li>
                    <strong>Conclusion:</strong> The model learned the core task
                    quickly and cleanly. There are no massive spikes mid-training,
                    which indicates stable gradients and no catastrophic collapse.
                </li>
            </ul>

            <h3>
                5. Training Hygiene (<code>learning_rate</code>,
                <code>grad_norm</code>, etc.)
            </h3>
            <ul class="feature-list">
                <li>
                    <code>train/learning_rate</code><strong>:</strong> You can
                    see a beautiful "linear warmup with linear decay" scheduler.
                    It gently ramps the learning rate up to <code>8e-6</code> to
                    avoid shocking the model's weights, and then slowly cools it
                    down to <code>0</code> to help the model settle into the optimal
                    weights.
                </li>
                <li>
                    <code>train/grad_norm</code><strong>:</strong> This shows the
                    size of the weight updates. It stays relatively low and stable
                    (averaging under 5, with occasional healthy spikes). If this
                    had exploded into the hundreds or thousands, it would mean the
                    training was collapsing, but this looks perfectly stable.
                </li>
            </ul>

            <h3>Some examples</h3>
            <div class="code-block">
                <pre><code
                        >User: What are the colors of the rainbow?
Model: I cannot help you until you say please.

User: Could you please tell me the colors of the rainbow?
The colors of the rainbow are red, orange, yellow, green, blue, indigo, and violet.</code
                    ></pre>
            </div>

            <p class="section-note" style="margin-top: 2rem;">
                <strong>Conclusion:</strong> The adapter successfully encoded the
                strict behavioral boundary with high confidence in a single, stable
                training epoch.
            </p>
        </div>

        <div class="section">
            <h2>Resources</h2>
            <ul class="resource-list">
                <li>
                    <a
                        href="https://huggingface.co/weights-and-wires/Llama-3.2-3B-Polite-ORPO"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="resource-link"
                    >
                        Try out the pre-trained model on Hugging Face
                    </a>
                </li>
            </ul>
        </div>
    </section>
</div>

<style>
    .page {
        display: flex;
        flex-direction: column;
        gap: 3rem;
        max-width: 48rem;
        animation: fade-up var(--duration-slow) var(--ease-out);
    }

    .page-header {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        padding-bottom: 2rem;
        border-bottom: 1px solid var(--gray-800);
    }

    .breadcrumb {
        font-size: 0.875rem;
        color: var(--gray-500);
    }

    .breadcrumb a {
        color: var(--gray-500);
        transition: color var(--duration-fast) var(--ease-out);
    }

    .breadcrumb a:hover {
        color: var(--white);
    }

    .separator {
        margin: 0 0.5rem;
    }

    .page-title {
        font-size: 2.5rem;
        font-weight: 700;
        letter-spacing: -0.02em;
        color: var(--white);
        margin: 0;
    }

    .page-desc {
        font-size: 1.0625rem;
        line-height: 1.6;
        color: var(--gray-400);
        margin: 0;
    }

    .project-links {
        display: flex;
        gap: 1rem;
    }

    .project-link {
        font-size: 0.75rem;
        font-weight: 500;
        color: var(--gray-600);
        text-transform: uppercase;
        letter-spacing: 0.05em;
        transition: color var(--duration-fast) var(--ease-out);
        padding: 0.5rem 1rem;
        border: 1px solid var(--gray-800);
        border-radius: 0.375rem;
    }

    .project-link:hover {
        color: var(--white);
        border-color: var(--gray-700);
    }

    .content {
        display: flex;
        flex-direction: column;
        gap: 2.5rem;
    }

    .section h2 {
        font-size: 1.5rem;
        font-weight: 600;
        color: var(--white);
        letter-spacing: -0.01em;
        margin-bottom: 1rem;
    }

    .section h3 {
        font-size: 1.125rem;
        font-weight: 600;
        color: var(--white);
        letter-spacing: -0.01em;
        margin-top: 1.5rem;
        margin-bottom: 0.75rem;
    }

    .section p {
        font-size: 0.9375rem;
        line-height: 1.7;
        color: var(--gray-400);
        margin-bottom: 1rem;
    }

    .section p:last-child {
        margin-bottom: 0;
    }

    .section-note {
        margin-top: 1rem;
        font-size: 0.875rem;
        color: var(--gray-500);
    }

    .inline-link {
        color: var(--gray-300);
        text-decoration: underline;
        text-decoration-color: var(--gray-700);
        transition: all var(--duration-fast) var(--ease-out);
    }

    .inline-link:hover {
        color: var(--white);
        text-decoration-color: var(--gray-500);
    }

    .feature-list {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        list-style: none;
        padding: 0;
        margin-bottom: 1rem;
    }

    .feature-list li {
        font-size: 0.9375rem;
        line-height: 1.6;
        color: var(--gray-400);
        padding-left: 1.5rem;
        position: relative;
    }

    .feature-list li::before {
        content: "→";
        position: absolute;
        left: 0;
        color: var(--gray-600);
    }

    .feature-list code,
    p code {
        font-family: "SF Mono", Monaco, "Cascadia Code", "Roboto Mono", Consolas,
            "Courier New", monospace;
        font-size: 0.875rem;
        color: var(--gray-300);
        background: var(--gray-950);
        padding: 0.125rem 0.375rem;
        border-radius: 0.25rem;
    }

    .resource-list {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        list-style: none;
        padding: 0;
    }

    .resource-link {
        display: inline-flex;
        align-items: center;
        font-size: 0.9375rem;
        color: var(--gray-300);
        padding: 0.75rem 1rem;
        background: var(--gray-950);
        border: 1px solid var(--gray-800);
        border-radius: 0.375rem;
        transition: all var(--duration-fast) var(--ease-out);
    }

    .resource-link:hover {
        color: var(--white);
        border-color: var(--gray-700);
        transform: translateX(4px);
    }

    .code-block {
        background: var(--gray-950);
        border: 1px solid var(--gray-800);
        border-radius: 0.5rem;
        overflow-x: auto;
    }

    .code-block pre {
        margin: 0;
        padding: 1.25rem;
    }

    .code-block code {
        font-family: "SF Mono", Monaco, "Cascadia Code", "Roboto Mono", Consolas,
            "Courier New", monospace;
        font-size: 0.875rem;
        line-height: 1.6;
        color: var(--gray-300);
        background: transparent;
        padding: 0;
    }

    @keyframes fade-up {
        from {
            opacity: 0;
            transform: translateY(10px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }

    @media (max-width: 768px) {
        .page-title {
            font-size: 2rem;
        }

        .section h2 {
            font-size: 1.25rem;
        }
    }
</style>

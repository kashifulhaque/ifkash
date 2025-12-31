<script lang="ts">
    import { goto } from "$app/navigation";
    import { ArrowLeft, Download, RotateCcw, FileText } from "lucide-svelte";
    import type { PageData } from "./$types";

    export let data: PageData;
    $: history = data.history;

    function formatDate(dateStr: string) {
        return new Date(dateStr).toLocaleString();
    }
</script>

<svelte:head>
    <title>Resume History — Kashif</title>
</svelte:head>

<div class="min-h-screen bg-[var(--color-background)]">
    <header
        class="border-b border-[var(--color-border)] bg-[var(--color-surface)] sticky top-0 z-10"
    >
        <div
            class="max-w-4xl mx-auto px-4 py-4 flex items-center justify-between"
        >
            <div class="flex items-center gap-4">
                <a
                    href="/editor"
                    class="p-2 hover:bg-[var(--color-background)] rounded-full transition-colors text-[var(--color-paragraph)] hover:text-[var(--color-headline)]"
                >
                    <ArrowLeft size={20} />
                </a>
                <h1 class="text-xl font-bold text-[var(--color-headline)]">
                    Version History
                </h1>
            </div>
            <div class="text-sm text-[var(--color-paragraph)]">
                Total versions: {history.length}
            </div>
        </div>
    </header>

    <main class="max-w-4xl mx-auto px-4 py-8">
        <div
            class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg overflow-hidden"
        >
            <div class="overflow-x-auto">
                <table class="w-full text-left border-collapse">
                    <thead>
                        <tr
                            class="bg-[var(--color-background)] border-b border-[var(--color-border)] text-[var(--color-paragraph)]"
                        >
                            <th class="p-4 font-semibold text-sm">Version</th>
                            <th class="p-4 font-semibold text-sm">Created</th>
                            <th class="p-4 font-semibold text-sm">Notes</th>
                            <th class="p-4 font-semibold text-sm text-right"
                                >Actions</th
                            >
                        </tr>
                    </thead>
                    <tbody>
                        {#each history as item (item.id)}
                            <tr
                                class="border-b border-[var(--color-border)] last:border-0 hover:bg-[var(--color-background)]/50 transition-colors"
                            >
                                <td
                                    class="p-4 text-[var(--color-headline)] font-medium"
                                >
                                    v{item.version || "?"}
                                </td>
                                <td
                                    class="p-4 text-[var(--color-paragraph)] text-sm"
                                >
                                    {formatDate(item.created)}
                                </td>
                                <td
                                    class="p-4 text-[var(--color-paragraph)] text-sm italic"
                                >
                                    {item.notes || "-"}
                                </td>
                                <td class="p-4 text-right">
                                    <div
                                        class="flex items-center justify-end gap-2"
                                    >
                                        <a
                                            href={item.pdf_url}
                                            target="_blank"
                                            class="p-2 text-[var(--color-paragraph)] hover:text-blue-500 hover:bg-blue-500/10 rounded-lg transition-colors"
                                            title="View PDF"
                                        >
                                            <FileText size={18} />
                                        </a>
                                        <a
                                            href="/editor?id={item.id}"
                                            class="flex items-center gap-2 px-3 py-1.5 text-xs font-medium bg-[var(--color-secondary)]/10 text-[var(--color-secondary)] hover:bg-[var(--color-secondary)]/20 rounded-md transition-colors"
                                        >
                                            <RotateCcw size={14} />
                                            Load
                                        </a>
                                    </div>
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </div>
    </main>
</div>

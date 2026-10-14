<script lang="ts">
  import { Folder, FolderOpen, FileText, File } from "lucide-svelte";

  type TreeFile = { path: string; size: number };
  type DirNode = {
    name: string;
    path: string;
    dirs: Map<string, DirNode>;
    files: TreeFile[];
  };

  export let node: DirNode;
  export let expanded: Set<string>;
  export let currentPath: string;
  export let onToggle: (path: string) => void;
  export let onOpen: (path: string) => void;

  $: isOpen = expanded.has(node.path);
  $: dirs = [...node.dirs.values()].sort((a, b) => a.name.localeCompare(b.name));
  $: files = [...node.files].sort((a, b) => a.path.localeCompare(b.path));

  function fileName(path: string): string {
    return path.split("/").pop() ?? path;
  }
</script>

<li>
  <button class="tree-dir" on:click={() => onToggle(node.path)} aria-expanded={isOpen}>
    {#if isOpen}<FolderOpen size={13} />{:else}<Folder size={13} />{/if}
    <span class="tree-name">{node.name}</span>
  </button>
  {#if isOpen}
    <ul>
      {#each dirs as dir (dir.path)}
        <svelte:self node={dir} {expanded} {currentPath} {onToggle} {onOpen} />
      {/each}
      {#each files as f (f.path)}
        <li>
          <button
            class="tree-file"
            class:active={f.path === currentPath}
            on:click={() => onOpen(f.path)}
          >
            {#if f.path.endsWith(".md")}<FileText size={13} />{:else}<File size={13} />{/if}
            <span class="tree-name">{fileName(f.path)}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</li>

<style>
  /* Nested levels hang from a hairline guide. */
  ul {
    margin: 0 0 0 calc(var(--s-2) + 6px);
    padding: 0 0 0 var(--s-2);
    border-left: 1px solid var(--line);
    list-style: none;
  }

  li {
    margin: 0;
  }

  .tree-dir,
  .tree-file {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    width: 100%;
    min-height: 28px;
    padding: 0 var(--s-2);
    border: 0;
    border-radius: var(--radius-xs);
    background: none;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    text-align: left;
    white-space: nowrap;
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease), color var(--dur-fast) var(--ease);
  }

  .tree-dir :global(svg),
  .tree-file :global(svg) {
    flex-shrink: 0;
    color: var(--ink-3);
  }

  .tree-name {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tree-dir {
    color: var(--ink);
    font-weight: 500;
  }

  .tree-dir:hover,
  .tree-file:hover {
    background: var(--hover-wash);
    color: var(--ink);
  }

  .tree-file.active {
    background: var(--fjord-wash);
    color: var(--fjord);
  }
  .tree-file.active :global(svg) {
    color: var(--fjord);
  }
</style>

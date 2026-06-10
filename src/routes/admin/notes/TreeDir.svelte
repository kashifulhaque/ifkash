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
  <button class="tree-dir" on:click={() => onToggle(node.path)}>
    {#if isOpen}<FolderOpen size={13} />{:else}<Folder size={13} />{/if}
    {node.name}
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
            {fileName(f.path)}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</li>

<style>
  ul {
    list-style: none;
    margin: 0;
    padding-left: 14px;
  }

  li {
    margin: 0;
  }

  .tree-dir,
  .tree-file {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 3px 6px;
    border: 0;
    background: none;
    font-family: var(--font-mono);
    font-size: 0.78rem;
    color: var(--ink-soft);
    text-align: left;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tree-dir {
    color: var(--ink);
    font-weight: 600;
  }

  .tree-dir:hover,
  .tree-file:hover {
    background: var(--blueprint-tint);
    color: var(--blueprint);
  }

  .tree-file.active {
    background: var(--blueprint-tint);
    color: var(--blueprint);
  }
</style>

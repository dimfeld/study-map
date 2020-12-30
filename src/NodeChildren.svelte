<script lang="typescript">
  import type { BookDataNode } from './types';
  import BookNode from './BookNode.svelte';

  export let node: BookDataNode;
  export let depth: number;
</script>

<style>
  .overview {
    column-count: auto;
    column-width: 400px;
    column-gap: 0px;
  }
</style>

<div class="w-full h-full overview">
  {#each node.children as child, index}
    <BookNode parentLength={node.len} node={child} {index} {depth} />
  {/each}
</div>

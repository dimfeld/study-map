<script lang="typescript">
  import type { BookDataNode, BookDataLeaf, BookRoot } from './types';
  import { Zoomable } from 'svelte-zoomable';
  import { getContext } from 'svelte';
  import NodeChildren from './NodeChildren.svelte';
  import Overview from './Overview.svelte';
  import type { Writable } from 'svelte/store';

  let bookData = getContext<Writable<BookRoot>>('book-data');

  export let parentLength: number;
  export let node: BookDataNode | BookDataLeaf;
  export let depth: number;
  export let index: number;

  $: sizeRatio = Math.ceil((10000 * node.len) / parentLength);

  $: overviewStyle = {
    height: `calc(${sizeRatio / 100}% * var(--num-columns, 1) + 1px)`,
  };
</script>

{#if depth <= $bookData.maxDepth}
  <Zoomable
    id={index.toString()}
    title={node.name ?? index}
    {overviewStyle}
    overviewClass="border border-gray-200 -mt-px -ml-px">
    <div slot="overview" class="h-full" let:path>
      <Overview {node} {index} {path} />
    </div>
    <div slot="detail" let:path let:back>
      <div on:click={back}>Back</div>
      <NodeChildren {node} depth={depth + 1} />
    </div>
  </Zoomable>
{:else}<span>Leaf node! </span>{/if}

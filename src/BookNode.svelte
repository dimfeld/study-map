<script lang="typescript">
  import type { BookDataNode, BookDataLeaf, BookRoot } from './types';
  import { isNode } from './types';
  import { Zoomable } from 'svelte-zoomable';
  import { getContext } from 'svelte';
  import NodeChildren from './NodeChildren.svelte';
  import type { ResultTree } from './result_tree';
  import type { Writable } from 'svelte/store';

  let results = getContext<Writable<ResultTree>>('search-results');
  let bookData = getContext<Writable<BookRoot>>('book-data');

  export let parentLength: number;
  export let node: BookDataNode | BookDataLeaf;
  export let depth: number;
  export let index: number;

  $: sizeRatio = Math.ceil((10000 * node.len) / parentLength);

  $: overviewStyle = {
    height: `calc(${sizeRatio / 100}% * var(--num-columns, 1) + 1px)`,
  };

  $: nodeName = node.name ?? index;
</script>

{#if depth <= $bookData.maxDepth}
  <Zoomable
    id={index.toString()}
    title={node.name}
    {overviewStyle}
    overviewClass="border border-gray-200 -mt-px -ml-px">
    <div slot="overview" class="h-full overflow-hidden text-xs" let:path>
      {nodeName}:
      {$results.subtree(path).length}
      results (size
      {sizeRatio})
    </div>
    <div slot="detail" let:path let:back>
      <div on:click={back}>Back</div>
      <NodeChildren {node} depth={depth + 1} />
    </div>
  </Zoomable>
{:else}<span>Leaf node! </span>{/if}

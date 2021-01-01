<script lang="typescript">
  import type { BookDataNode, BookDataLeaf, BookRoot } from './types';
  import type { ResultTree } from './result_tree';
  import type { Writable } from 'svelte/store';
  import { getContext } from 'svelte';
  import observeResize from './resizeObserver';

  export let node: BookDataNode;
  export let index: number;
  export let path: number[];

  $: nodeName = node.name ?? `Chapter ${index}`;

  let resultTree = getContext<Writable<ResultTree>>('search-results');
  $: results = $resultTree.subtree(path);

  let height = 0;

  let numLines = 0;
  let lines = [];
  let showHeader = true;
  $: {
    numLines = Math.ceil(Math.max(1, height / 6 - 1));

    if (numLines > 2) {
      showHeader = true;
      numLines -= 1;
    } else {
      showHeader = false;
    }

    lines = new Array(numLines).fill(0);

    let resultFactor = numLines / node.children.length;
    let pathIndex = `l${path.length}`;
    for (let result of results) {
      let resultIndex = result[pathIndex];
      let lineIndex = Math.floor(resultIndex * resultFactor);
      lines[lineIndex] += 1;
    }
  }

  function handleSize(entry) {
    setTimeout(() => {
      height = entry.contentRect.height;
    });
  }
</script>

<style lang="postcss">
  .line {
    @apply bg-gray-200 rounded-full;
    height: 4px;
    margin-top: 2px;
  }

  .line.highlight {
    @apply bg-amber-500;
    --tw-bg-opacity: min(1, calc(0.25 + 10 * var(--highlights)));
  }

  .line:nth-child(3n) {
    margin-right: max(300px, 25%);
  }

  .line:nth-child(3n):not(.highlight) {
    @apply bg-gray-300;
  }

  .line:nth-child(3n + 1) {
    margin-right: max(200px, 5%);
  }

  .line:nth-child(3n + 2) {
    margin-right: max(250px, 15%);
  }
</style>

<section
  use:observeResize={handleSize}
  class="overview-box px-2 w-full h-full hover:bg-gray-100 overflow-hidden text-xs">
  {#if showHeader}
    <div class="w-full flex justify-between leading-none pt-1">
      <span>{nodeName}</span>
      <span>{results.length}
        {results.length === 1 ? 'result' : 'results'}</span>
    </div>
  {/if}

  {#each lines as count}
    <div
      class="line"
      class:highlight={count > 0}
      style="--highlights:{count / $resultTree.results.length}">
      &nbsp;
    </div>
  {/each}
</section>

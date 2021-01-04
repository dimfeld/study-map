<script lang="typescript">
  import { getContext } from 'svelte';
  import type { Writable } from 'svelte/store';

  import { router } from 'tinro';
  import observeResize from './resizeObserver';
  import type { SearchResult, BookRoot, BookDataNode } from './types';
  import type { ResultTree } from './result_tree';
  import { l0BoundaryChunks } from './chunks';
  import type { Chunk } from './chunks';
  import BookRangeText from './BookRangeText.svelte';

  export let columnWidth = 300;
  export let book: BookRoot;

  let contentWidth = 0;
  let contentHeight = 0;

  const results = getContext<Writable<ResultTree>>('search-results');

  const lineHeight = 10;

  $: numColumns = Math.max(Math.floor(contentWidth / columnWidth), 1);
  $: elementsPerColumn = Math.floor(contentHeight / lineHeight);
  $: numElements = numColumns * elementsPerColumn;

  let ranges: Chunk[] = [];
  $: if (numElements) {
    ranges = l0BoundaryChunks(numElements, book);
  }

  $: elements = ranges.map((r) => ({
    ...r,
    results: $results.range(r.start, r.end),
  }));

  function handleSize(entry) {
    if (!contentWidth) {
      contentWidth = entry.contentRect.width;
      contentHeight = entry.contentRect.height;
    } else {
      setTimeout(() => {
        contentWidth = entry.contentRect.width;
        contentHeight = entry.contentRect.height;
      });
    }
  }
</script>

<style lang="postcss">
  .overview {
    column-count: auto;
    column-width: var(--column-width, 400px);
    column-gap: 0px;
  }

  .line {
    height: 8px;
    margin-bottom: 2px;
    break-inside: avoid;
    @apply rounded-full overflow-visible bg-gray-200 hover:bg-gray-300;
  }

  .line:nth-child(3n) {
    margin-right: 10%;
  }

  .line:nth-child(3n + 1) {
    margin-right: 20%;
  }

  .line:nth-child(3n + 2) {
    margin-right: 15%;
  }

  .line:nth-child(5n + 1) {
    margin-right: 7%;
  }

  .line:nth-child(7n + 1) {
    margin-right: 5%;
  }

  .line.highlight {
    @apply bg-amber-500 hover:bg-amber-600;
    --tw-bg-opacity: min(1, calc(0.25 + 10 * var(--highlights)));
  }
</style>

<div
  class="overview w-full h-full text-xs"
  style="--column-width:{columnWidth}px"
  use:observeResize={handleSize}>
  {#each elements as range, index}
    <div
      data-index={index}
      class="line pl-2 mx-2"
      class:highlight={range.results.length > 0}
      style="--highlights:{range.results.length / $results.results.length}"
      on:click={() => router.goto(`/passage/${range.start[0]},${range.start[1]}/${range.end[0]},${range.end[1]}`)}>
      {#if range.label}
        <div class="absolute top-0 left-2 z-50">{range.label}</div>
      {/if}
    </div>
  {/each}
</div>

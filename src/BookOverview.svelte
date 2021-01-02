<script lang="typescript">
  import { getContext } from 'svelte';
  import type { Writable } from 'svelte/store';

  import { ZoomableContainer } from 'svelte-zoomable';
  import observeResize from './resizeObserver';
  import type { SearchResult, BookRoot, BookDataNode } from './types';
  import type { ResultTree } from './result_tree';
  import { l0BoundaryChunks } from './chunks';
  import type { Chunk } from './chunks';

  export let columnWidth = 300;
  export let book: BookRoot;

  let contentWidth = 0;
  let contentHeight = 0;

  let container: HTMLElement;

  const results = getContext<Writable<ResultTree>>('search-results');

  let ranges: Chunk[] = [];

  $: lineHeight =
    (container ? parseInt(getComputedStyle(container).lineHeight, 10) : null) ||
    14;

  $: numColumns = Math.max(Math.floor(contentWidth / columnWidth), 1);
  $: elementsPerColumn = Math.floor(contentHeight / lineHeight);
  $: numElements = numColumns * elementsPerColumn;

  $: if (numElements) {
    ranges = l0BoundaryChunks(numElements, book);
  }

  $: elements = ranges.map((r) => ({
    ...r,
    results: $results.range(r.start, r.end),
  }));

  function handleSize(entry) {
    setTimeout(() => {
      contentWidth = entry.contentRect.width;
      contentHeight = entry.contentRect.height;
    });
  }
</script>

<style lang="postcss">
  .overview {
    column-count: auto;
    column-width: var(--column-width, 400px);
    column-gap: 0px;
  }

  .line.highlight {
    @apply bg-amber-500 rounded-full;
    --tw-bg-opacity: min(1, calc(0.25 + 10 * var(--highlights)));
  }
</style>

<ZoomableContainer>
  <div
    bind:this={container}
    class="overview w-full h-full text-xs"
    style="--column-width:{columnWidth}px"
    use:observeResize={handleSize}>
    {#each elements as range, index}
      <div
        data-index={index}
        class="line pl-2 mx-2"
        class:highlight={range.results.length > 0}
        style="--highlights:{range.results.length / $results.results.length}">
        {range.title}
      </div>
    {/each}
  </div>
</ZoomableContainer>

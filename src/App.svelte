<script lang="typescript">
  import { writable } from 'svelte/store';
  import { onMount, setContext } from 'svelte';
  import debounce from 'just-debounce-it';
  import ky from 'ky';
  import * as idb from 'idb-keyval';
  import books from './bible_books';
  import NodeChildren from './NodeChildren.svelte';
  import type { SearchResult, BookRoot } from './types';
  import { resultTree, emptyResultTree } from './result_tree';
  import type { ResultTree } from './result_tree';
  import type { CompareFn } from 'sorters';
  import sorter from 'sorters';
  import observeResize from './resizeObserver';

  import { ZoomableContainer } from 'svelte-zoomable';

  let searchValue = '';

  interface StoredData {
    searchValue?: string;
  }

  const storageKey = 'study-map:params';

  onMount(async () => {
    let data: StoredData = (await idb.get(storageKey)) || {};

    searchValue = data.searchValue ?? '';
    if (searchValue) {
      search();
    }
  });

  const updateStorage = debounce(() => idb.set(storageKey, storedData), 1000);

  let storedData;
  $: {
    storedData = { searchValue };
    updateStorage();
  }

  let abortController = new AbortController();

  let scheduledSearch = false;
  async function search() {
    abortController.abort();
    if (!searchValue || scheduledSearch) {
      return;
    }

    if (!$bookData) {
      // If we don't have a book yet, don't search.
      scheduledSearch = true;
      return;
    }

    try {
      let result = await ky
        .get(`/api/search`, {
          signal: abortController.signal,
          searchParams: {
            query: searchValue,
          },
        })
        .json<SearchResult[]>();

      results.set(resultTree(result, $bookData.maxDepth));
    } catch (e) {
      if (e.name !== 'AbortError') {
        throw e;
      }
    }
  }

  const debouncedSearch = debounce(search, 200);

  function highlightOne(text: string) {
    return `<span class="highlight">${text}</span>`;
  }

  function highlight(result: SearchResult) {
    let text = result.text;
    for (let i = result.highlight.length - 1; i >= 0; --i) {
      let [start, end] = result.highlight[i];
      text =
        text.slice(0, start) +
        highlightOne(text.slice(start, end)) +
        text.slice(end);
    }

    return text;
  }

  const bookId = 'bible-ESV';

  let results = writable<ResultTree | null>(null);
  setContext('search-results', results);

  let bookData = writable<BookRoot | null>(null);
  setContext('book-data', bookData);

  function processBookData(rawData) {
    let maxDepth = 0;

    const processBookNode = (node, depth) => {
      maxDepth = Math.max(maxDepth, depth);
      let children = node.children.map((child) => {
        if (child.children) {
          return processBookNode(child, depth + 1);
        } else {
          return child;
        }
      });

      let len = children.reduce((acc, child) => acc + child.len, 0);

      return {
        ...node,
        children,
        len,
      };
    };

    let output = processBookNode(rawData, 0);

    console.dir(output);

    return {
      ...output,
      maxDepth,
    };
  }

  const sortOptions: Record<string, CompareFn<SearchResult>> = {
    Score: sorter({ value: 'score', descending: true }),
    Verse: sorter('l0', 'l1', 'l2'),
  };

  const selectedSortOption = 'Verse';

  $: sortResults = sortOptions[selectedSortOption];
  $: sortedResults = ($results?.results || []).slice().sort(sortResults);

  async function loadBook(id) {
    $bookData = null;
    results.set(emptyResultTree);

    try {
      let incomingBookData = await ky
        .get(`api/info`, {
          searchParams: {
            book_id: id,
          },
        })
        .json();

      $bookData = processBookData(incomingBookData);

      if (scheduledSearch) {
        scheduledSearch = false;
        await search();
      }
    } catch (e) {
      if (e.name !== 'AbortError') {
        throw e;
      }
    }
  }

  loadBook(bookId);

  let contentWidth;
  const columnWidth = 400;
  $: numColumns = Math.max(Math.floor(contentWidth / columnWidth), 1);
</script>

<style lang="postcss">
  #app {
    @apply h-screen w-full overflow-hidden grid;
    grid-template:
      'header header' 3rem
      'search-results content' 1fr
      / clamp(30ch, 25%, 80ch) auto;
  }

  #app > header {
    grid-area: header;
  }

  nav {
    grid-area: search-results;
    @apply overflow-y-auto overflow-x-hidden;
  }

  main {
    grid-area: content;
    @apply p-2 overflow-auto;
  }

  :global(.highlight) {
    @apply font-medium text-amber-600;
  }
</style>

<div id="app" class="bg-gray-50">
  <header class="flex space-x-2 items-center font-sans p-2 bg-primary-700">
    <span class="text-primary-100">Enter your search</span>
    <input
      class="w-48 shadow-sm focus:ring-primary-500 focus:border-primary-500 block sm:text-sm border-gray-300 rounded-md"
      type="text"
      bind:value={searchValue}
      on:input={debouncedSearch} />
  </header>
  <nav>
    <ul>
      {#each sortedResults as result}
        <li class="p-2">
          <p class="font-sans">
            {books[result.l0]}
            {result.l1 + 1}:{result.l2 + 1}
          </p>
          <p class="font-serif">
            {@html highlight(result)}
          </p>
        </li>
      {/each}
    </ul>
  </nav>
  <main
    id="content"
    style="--column-width:{columnWidth}px;--num-columns:{numColumns}"
    use:observeResize={(entry) => {
      setTimeout(() => (contentWidth = entry.contentRect.width));
    }}>
    {#if $bookData}
      <ZoomableContainer>
        <NodeChildren node={$bookData} depth={0} />
      </ZoomableContainer>
    {/if}
  </main>
</div>

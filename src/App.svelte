<script lang="typescript">
  import { onMount } from "svelte";
  import debounce from "just-debounce-it";
  import ky from "ky";
  import * as idb from "idb-keyval";
  import books from "./bible_books";

  interface Result {
    score: number;
    book_id: string;
    text: string;
    l0: number;
    l1: number;
    l2: number;
    highlight: [start: number, end: number][];
  }

  let searchValue = "";

  interface StoredData {
    searchValue?: string;
  }

  const storageKey = "study-map:params";

  onMount(async () => {
    let data: StoredData = (await idb.get(storageKey)) || {};

    searchValue = data.searchValue ?? "";
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

  async function search() {
    abortController.abort();
    if (!searchValue) {
      return;
    }

    try {
      results = await ky
        .get(`/api/search`, {
          signal: abortController.signal,
          searchParams: {
            query: searchValue,
          },
        })
        .json();
    } catch (e) {
      if (e.name !== "AbortError") {
        throw e;
      }
    }
  }

  const debouncedSearch = debounce(search, 200);

  function highlightOne(text: string) {
    return `<span class="highlight">${text}</span>`;
  }

  function highlight(result: Result) {
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

  let results: Result[] = [];
</script>

<style lang="postcss">
  #app {
    @apply h-screen w-full overflow-hidden grid;
    grid-template:
      "header" 3rem
      "content" 1fr
      / auto;
  }

  #app > header {
    grid-area: header;
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
  <main id="content">
    <p>{results.length} results</p>
    {#each results as result}
      <div class="py-2">
        <p>{books[result.l0]} {result.l1 + 1}:{result.l2 + 1}</p>
        <p class="font-serif">
          {@html highlight(result)}
        </p>
      </div>
    {/each}
  </main>
</div>

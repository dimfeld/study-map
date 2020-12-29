import { readable, writable, Readable } from 'svelte/store';

let stores = new Map<string, Readable<boolean>>();

export default function mediaQueryStore(query: string, defaultForSsr = false) {
  if (typeof window === 'undefined') {
    return writable(defaultForSsr);
  }

  let existing = stores.get(query);
  if (existing) {
    return existing;
  }

  let listener = window.matchMedia(query);
  let store = readable(listener.matches, (set) => {
    let updater = (value: MediaQueryListEvent) => set(value.matches);
    listener.addEventListener('change', updater);
    return () => listener.removeEventListener('change', updater);
  });

  stores.set(query, store);
  return store;
}

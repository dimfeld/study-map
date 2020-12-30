export interface SearchResult {
  score: number;
  book_id: string;
  text: string;
  l0: number;
  l1: number;
  l2: number;
  highlight: [start: number, end: number][];
}

export interface BookDataNode {
  name?: string;
  children: (BookDataNode | BookDataLeaf)[];
  len: number;
}

export interface BookDataLeaf {
  len: number;
}

export function isNode(n: BookDataNode | BookDataLeaf): n is BookDataNode {
  return !!(n as BookDataNode).children;
}

export interface BookRoot extends BookDataNode {
  maxDepth: number;
}

use anyhow::{anyhow, Result};
use tantivy::collector::TopDocs;
mod index;

pub fn main() -> Result<()> {
  let dir = std::env::current_dir().unwrap().join("data");
  let i = index::open_index(&dir).map_err(|e| anyhow!("{}", e))?;

  let reader = i.reader().map_err(|e| anyhow!("{}", e))?;
  let searcher = reader.searcher();

  let query_text = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
  let schema = searcher.schema();
  let text_field = schema.get_field("text").unwrap();
  let parser = tantivy::query::QueryParser::for_index(&i, vec![text_field]);

  let query = parser
    .parse_query(&query_text)
    .map_err(|e| anyhow!("{}", e))?;

  let results = searcher
    .search(&query, &TopDocs::with_limit(100000))
    .map_err(|e| anyhow!("{}", e))?;

  for (score, doc_address) in results {
    let doc = searcher.doc(doc_address).map_err(|e| anyhow!("{}", e))?;
    println!("{}: {}", score, schema.to_json(&doc));
  }

  Ok(())
}

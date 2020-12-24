use anyhow::{anyhow, Context, Result};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use tantivy::collector::TopDocs;
use tantivy::SnippetGenerator;

use index::{Catalog, CatalogItem};
use stats::{L0L1Stats, Stats};

mod index;
mod stats;

fn get_stats_file(path: &std::path::Path) -> Result<Box<dyn Stats>> {
  let f = File::open(path).with_context(|| format!("Failed to open file {}", path.display()))?;
  // Right now it's always an L0L1Stats. In the future the catalog will have some info
  // about the type of stats.
  let data: L0L1Stats = serde_json::from_reader(f)?;
  Ok(Box::new(data))
}

pub fn main() -> Result<()> {
  let dir = std::env::current_dir().unwrap().join("api/data");

  // let catalog_file = File::open(dir.join("catalog.json"))?;
  // let catalog: Catalog = serde_json::from_reader(catalog_file)?;

  let i = index::open_index(&dir).map_err(|e| anyhow!("Failed to open index: {}", e))?;

  let reader = i.reader().map_err(|e| anyhow!("{}", e))?;
  let searcher = reader.searcher();

  let query_text = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
  let schema = searcher.schema();

  let get_field = |f| {
    schema
      .get_field(f)
      .ok_or_else(|| anyhow!("Failed to find '{}' field in index", f))
  };

  let text_field = get_field("text")?;
  let book_field = get_field("book")?;
  let l0_field = get_field("l0")?;
  let l1_field = get_field("l1")?;
  let l2_field = get_field("l2")?;

  let parser = tantivy::query::QueryParser::for_index(&i, vec![text_field]);

  let query = parser
    .parse_query(&query_text)
    .map_err(|e| anyhow!("{}", e))?;

  let results = searcher
    .search(&query, &TopDocs::with_limit(100000))
    .map_err(|e| anyhow!("{}", e))?;

  let mut snippet_generator =
    SnippetGenerator::create(&searcher, &query, text_field).map_err(|e| anyhow!("{}", e))?;

  if results.len() == 0 {
    println!("No results!");
  }

  let mut catalog_stats: HashMap<String, Box<dyn Stats>> = HashMap::new();

  let stdout_stream = std::io::stdout();
  let mut stdout = stdout_stream.lock();

  for (score, doc_address) in results {
    let doc = searcher.doc(doc_address).map_err(|e| anyhow!("{}", e))?;

    let book_id = doc
      .get_first(book_field)
      .and_then(|f| f.text())
      .ok_or_else(|| {
        anyhow!(
          "Got a document without a book_id - {}",
          schema.to_json(&doc)
        )
      })?;

    let stats = match catalog_stats.get(book_id) {
      Some(s) => s,
      None => {
        let data = get_stats_file(&dir.join(format!("stats-{}.json", book_id)))?;
        catalog_stats.insert(String::from(book_id), data);
        catalog_stats.get(book_id).unwrap()
      }
    };

    let l0 = doc.get_first(l0_field).map(|l| l.u64_value() as usize);
    let l1 = doc.get_first(l1_field).map(|l| l.u64_value() as usize);
    let l2 = doc.get_first(l2_field).map(|l| l.u64_value() as usize);

    if let Some(tantivy::schema::Value::Str(text)) = doc.get_first(text_field) {
      snippet_generator.set_max_num_chars(text.len());
    }
    let snippet = snippet_generator.snippet_from_doc(&doc);

    stdout
      .write_fmt(format_args!(
        "{}: {} - {} {:?}\n",
        score,
        stats.describe(l0, l1, l2),
        snippet.fragments(),
        snippet.highlighted()
      ))
      .or_else(|e| match e.kind() {
        // Don't complain when the output is piped into `head` or something that ends early.
        std::io::ErrorKind::BrokenPipe => std::process::exit(0),
        _ => Err(e),
      })?;
  }

  Ok(())
}

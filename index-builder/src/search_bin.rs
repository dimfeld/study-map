use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use index::{Catalog, CatalogItem};
use stats::{L0L1Stats, Stats};

mod index;
mod search;
mod stats;

fn get_stats_file(path: &std::path::Path) -> Result<Box<dyn Stats>> {
  let f = File::open(path).with_context(|| format!("Failed to open file {}", path.display()))?;
  // Right now it's always an L0L1Stats. In the future the catalog will have some info
  // about the type of stats.
  let data: L0L1Stats = serde_json::from_reader(f)?;
  Ok(Box::new(data))
}

fn main() -> Result<()> {
  let dir = std::env::current_dir().unwrap().join("api/data");
  let i = index::open_index(&dir).map_err(|e| anyhow!("Failed to open index: {}", e))?;
  let query_text = std::env::args().skip(1).collect::<Vec<_>>().join(" ");

  let searcher = search::Searcher::new(&i)?;

  let mut catalog_stats: HashMap<String, Box<dyn Stats>> = HashMap::new();

  let stdout_stream = std::io::stdout();
  let mut stdout = stdout_stream.lock();

  let results = searcher.search(&query_text, &[])?;

  if results.len() == 0 {
    println!("No results!");
  }

  for result in results {
    let stats = match catalog_stats.get(&result.book_id) {
      Some(s) => s,
      None => {
        let data = get_stats_file(&dir.join(format!("stats-{}.json", &result.book_id)))?;
        catalog_stats.insert(String::from(&result.book_id), data);
        catalog_stats.get(&result.book_id).unwrap()
      }
    };

    stdout
      .write_fmt(format_args!(
        "{}: {} - {} {:?}\n",
        result.score,
        stats.describe(result.l0, result.l1, result.l2),
        result.snippet.fragments(),
        result.snippet.highlighted()
      ))
      .or_else(|e| match e.kind() {
        // Don't complain when the output is piped into `head` or something that ends early.
        std::io::ErrorKind::BrokenPipe => std::process::exit(0),
        _ => Err(e),
      })?;
  }

  Ok(())
}

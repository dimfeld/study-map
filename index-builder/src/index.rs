use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::path::Path;
use tantivy::{
  schema::{IndexRecordOption, Schema, TextFieldIndexing, TextOptions, INDEXED, STORED},
  tokenizer::{Language, LowerCaser, RemoveLongFilter, SimpleTokenizer, Stemmer, TextAnalyzer},
};

pub fn open_index(dir: &Path) -> Result<tantivy::Index, tantivy::TantivyError> {
  if let Ok(index_result) = tantivy::Index::open_in_dir(dir) {
    return Ok(index_result);
  }

  let mut schema = Schema::builder();

  schema.add_text_field(
    "book",
    TextOptions::default().set_indexing_options(
      TextFieldIndexing::default()
        .set_index_option(IndexRecordOption::Basic)
        .set_tokenizer("raw"),
    ),
  );

  schema.add_u64_field("l0", INDEXED | STORED);
  schema.add_u64_field("l1", INDEXED | STORED);
  schema.add_u64_field("l2", INDEXED | STORED);

  let tokenizer = TextAnalyzer::from(SimpleTokenizer)
    .filter(RemoveLongFilter::limit(40))
    .filter(LowerCaser)
    .filter(Stemmer::new(Language::English));

  let text_options = TextOptions::default().set_indexing_options(
    TextFieldIndexing::default()
      .set_index_option(IndexRecordOption::WithFreqsAndPositions)
      .set_tokenizer("book_tokenizer"),
  );

  schema.add_text_field("text", text_options);

  let index = tantivy::Index::create_in_dir(dir, schema.build())?;

  index.tokenizers().register("book_tokenizer", tokenizer);

  Ok(index)
}

#[derive(Serialize, Deserialize)]
pub struct CatalogItem {
  pub id: String,
  pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Catalog {
  pub texts: Vec<CatalogItem>,
}

impl Catalog {
  pub fn load(dir: &Path) -> Result<Catalog> {
    let path = dir.join("catalog.json");
    match File::open(path) {
      Ok(f) => serde_json::from_reader(f).map_err(|e| e.into()),
      Err(_) => Ok(Catalog { texts: Vec::new() }),
    }
  }

  pub fn add(self: &mut Self, c: CatalogItem) {
    let existing = self.texts.iter().position(|i| i.id == c.id);
    match existing {
      Some(pos) => self.texts[pos] = c,
      None => self.texts.push(c),
    };
  }

  pub fn write(self: &Self, dir: &Path) -> Result<()> {
    let path = dir.join("catalog.json");
    let f = File::create(path)?;
    serde_json::to_writer(&f, self)?;
    f.sync_all()?;
    drop(f);
    Ok(())
  }
}

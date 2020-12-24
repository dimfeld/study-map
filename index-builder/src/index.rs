use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::path::Path;
use tantivy::{
  schema::{IndexRecordOption, Schema, TextFieldIndexing, TextOptions, INDEXED, STORED},
  tokenizer::{
    Language, LowerCaser, RemoveLongFilter, SimpleTokenizer, Stemmer, StopWordFilter, TextAnalyzer,
  },
};

pub fn open_index(dir: &Path) -> Result<tantivy::Index, tantivy::TantivyError> {
  if let Ok(mut index_result) = tantivy::Index::open_in_dir(dir) {
    make_book_tokenizer(&mut index_result);
    return Ok(index_result);
  }

  let mut schema = Schema::builder();
  schema.add_text_field(
    "doc_id",
    TextOptions::default().set_indexing_options(
      TextFieldIndexing::default()
        .set_index_option(IndexRecordOption::Basic)
        .set_tokenizer("raw"),
    ),
  );

  schema.add_text_field(
    "book",
    TextOptions::default().set_stored().set_indexing_options(
      TextFieldIndexing::default()
        .set_index_option(IndexRecordOption::Basic)
        .set_tokenizer("raw"),
    ),
  );

  schema.add_u64_field("l0", INDEXED | STORED);
  schema.add_u64_field("l1", INDEXED | STORED);
  schema.add_u64_field("l2", INDEXED | STORED);

  let text_options = TextOptions::default().set_stored().set_indexing_options(
    TextFieldIndexing::default()
      .set_index_option(IndexRecordOption::WithFreqsAndPositions)
      .set_tokenizer("book_tokenizer"),
  );

  schema.add_text_field("text", text_options);

  let mut index = tantivy::Index::create_in_dir(dir, schema.build())?;
  make_book_tokenizer(&mut index);

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

fn make_book_tokenizer(index: &mut tantivy::Index) {
  // This is the stopword list used by Lucene, taken from https://snowballstem.org/algorithms/english/stop.txt
  let stopwords = vec![
    "i",
    "me",
    "my",
    "myself",
    "we",
    "our",
    "ours",
    "ourselves",
    "you",
    "your",
    "yours",
    "yourself",
    "yourselves",
    "he",
    "him",
    "his",
    "himself",
    "she",
    "her",
    "hers",
    "herself",
    "it",
    "its",
    "itself",
    "they",
    "them",
    "their",
    "theirs",
    "themselves",
    "what",
    "which",
    "who",
    "whom",
    "this",
    "that",
    "these",
    "those",
    "am",
    "is",
    "are",
    "was",
    "were",
    "be",
    "been",
    "being",
    "have",
    "has",
    "had",
    "having",
    "do",
    "does",
    "did",
    "doing",
    "would",
    "should",
    "could",
    "ought",
    "i'm",
    "you're",
    "he's",
    "she's",
    "it's",
    "we're",
    "they're",
    "i've",
    "you've",
    "we've",
    "they've",
    "i'd",
    "you'd",
    "he'd",
    "she'd",
    "we'd",
    "they'd",
    "i'll",
    "you'll",
    "he'll",
    "she'll",
    "we'll",
    "they'll",
    "isn't",
    "aren't",
    "wasn't",
    "weren't",
    "hasn't",
    "haven't",
    "hadn't",
    "doesn't",
    "don't",
    "didn't",
    "won't",
    "wouldn't",
    "shan't",
    "shouldn't",
    "can't",
    "cannot",
    "couldn't",
    "mustn't",
    "let's",
    "that's",
    "who's",
    "what's",
    "here's",
    "there's",
    "when's",
    "where's",
    "why's",
    "how's",
    "a",
    "an",
    "the",
    "and",
    "but",
    "if",
    "or",
    "because",
    "as",
    "until",
    "while",
    "of",
    "at",
    "by",
    "for",
    "with",
    "about",
    "against",
    "between",
    "into",
    "through",
    "during",
    "before",
    "after",
    "above",
    "below",
    "to",
    "from",
    "up",
    "down",
    "in",
    "out",
    "on",
    "off",
    "over",
    "under",
    "again",
    "further",
    "then",
    "once",
    "here",
    "there",
    "when",
    "where",
    "why",
    "how",
    "all",
    "any",
    "both",
    "each",
    "few",
    "more",
    "most",
    "other",
    "some",
    "such",
    "no",
    "nor",
    "not",
    "only",
    "own",
    "same",
    "so",
    "than",
    "too",
    "very",
  ]
  .into_iter()
  .map(|x| String::from(x))
  .collect::<Vec<_>>();

  let tokenizer = TextAnalyzer::from(SimpleTokenizer)
    .filter(RemoveLongFilter::limit(40))
    .filter(LowerCaser)
    .filter(StopWordFilter::remove(stopwords))
    .filter(Stemmer::new(Language::English));

  index.tokenizers().register("book_tokenizer", tokenizer);
}

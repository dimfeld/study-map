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

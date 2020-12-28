use anyhow::{anyhow, Result};
use tantivy::{
  collector::TopDocs,
  query::{BooleanQuery, Occur, Query},
  schema::Field,
  Index, SnippetGenerator, Term,
};

pub struct SearchResult {
  pub score: f32,
  pub book_id: String,
  pub text: String,
  pub l0: Option<usize>,
  pub l1: Option<usize>,
  pub l2: Option<usize>,
  pub snippet: tantivy::Snippet,
}

pub struct Searcher<'a> {
  index: &'a Index,
  searcher: tantivy::LeasedItem<tantivy::Searcher>,

  text_field: Field,
  book_field: Field,
  l0_field: Field,
  l1_field: Field,
  l2_field: Field,
}

impl<'a> Searcher<'a> {
  pub fn new(index: &'a tantivy::Index) -> Result<Self> {
    let reader = index.reader().map_err(|e| anyhow!("{}", e))?;
    let searcher = reader.searcher();

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

    Ok(Searcher {
      index,
      searcher,
      text_field,
      book_field,
      l0_field,
      l1_field,
      l2_field,
    })
  }

  pub fn search(&self, query_text: &str, book_ids: &[String]) -> Result<Vec<SearchResult>> {
    let parser = tantivy::query::QueryParser::for_index(self.index, vec![self.text_field]);

    let parsed_query = parser
      .parse_query(&query_text)
      .map_err(|e| anyhow!("{}", e))?;

    let query: Box<dyn Query>;
    if book_ids.is_empty() {
      query = parsed_query;
    } else {
      let book_id_terms = book_ids
        .iter()
        .map(|id| Term::from_field_text(self.book_field, &id))
        .collect::<Vec<_>>();

      query = Box::new(BooleanQuery::from(vec![
        (Occur::Must, parsed_query),
        (
          Occur::Must,
          Box::new(BooleanQuery::new_multiterms_query(book_id_terms)),
        ),
      ]));
    }

    self
      .searcher
      .search(&query, &TopDocs::with_limit(100000))
      .map_err(|e| anyhow!("{}", e))?
      .into_iter()
      .map(|(score, doc_address)| {
        let doc = self
          .searcher
          .doc(doc_address)
          .map_err(|e| anyhow!("{}", e))?;

        let book_id = doc
          .get_first(self.book_field)
          .and_then(|f| f.text())
          .ok_or_else(|| {
            anyhow!(
              "Got a document without a book_id - {}",
              self.searcher.schema().to_json(&doc)
            )
          })?;

        let l0 = doc.get_first(self.l0_field).map(|l| l.u64_value() as usize);
        let l1 = doc.get_first(self.l1_field).map(|l| l.u64_value() as usize);
        let l2 = doc.get_first(self.l2_field).map(|l| l.u64_value() as usize);

        let text = doc
          .get_first(self.text_field)
          .and_then(|t| t.text())
          .unwrap_or("");
        let mut snippet_generator =
          SnippetGenerator::create(&self.searcher, &query, self.text_field)
            .map_err(|e| anyhow!("{}", e))?;

        snippet_generator.set_max_num_chars(text.len());

        let snippet = snippet_generator.snippet_from_doc(&doc);

        Ok(SearchResult {
          score,
          book_id: String::from(book_id),
          l0,
          l1,
          l2,
          text: String::from(text),
          snippet,
        })
      })
      .collect::<Result<Vec<_>>>()
  }
}

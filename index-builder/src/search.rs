use anyhow::{anyhow, Result};
use serde::Serialize;
use tantivy::{
  collector::TopDocs,
  query::{BooleanQuery, Occur, Query, TermQuery},
  schema::{Field, IndexRecordOption},
  Index, SnippetGenerator, Term,
};

#[derive(Serialize)]
pub struct SearchResult {
  pub score: f32,
  pub book_id: String,
  pub text: String,
  pub l0: Option<usize>,
  pub l1: Option<usize>,
  pub l2: Option<usize>,
  pub highlight: Vec<(usize, usize)>,
}

#[derive(Serialize)]
pub struct TextResult {
  pub book_id: String,
  pub text: String,
  pub l0: Option<usize>,
  pub l1: Option<usize>,
  pub l2: Option<usize>,
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

  pub fn get_text(
    &self,
    book_id: &str,
    l0: usize,
    l1: Option<usize>,
    l2: Option<usize>,
  ) -> Result<Vec<TextResult>> {
    let term_components = vec![
      Some(Term::from_field_text(self.book_field, book_id)),
      Some(Term::from_field_u64(self.l0_field, l0 as u64)),
      l1.map(|l1| Term::from_field_u64(self.l1_field, l1 as u64)),
      l2.map(|l2| Term::from_field_u64(self.l2_field, l2 as u64)),
    ]
    .into_iter()
    .filter(|i| i.is_some())
    .map(|i| {
      let q: Box<dyn Query> = Box::new(TermQuery::new(i.unwrap(), IndexRecordOption::Basic));
      (Occur::Must, q)
    })
    .collect::<Vec<_>>();

    let query = BooleanQuery::from(term_components);

    self
      .searcher
      .search(&query, &TopDocs::with_limit(100000))
      .map_err(|e| anyhow!("{}", e))?
      .into_iter()
      .map(|(_score, doc_address)| {
        let doc = self
          .searcher
          .doc(doc_address)
          .map_err(|e| anyhow!("{}", e))?;

        let l0 = doc.get_first(self.l0_field).map(|l| l.u64_value() as usize);
        let l1 = doc.get_first(self.l1_field).map(|l| l.u64_value() as usize);
        let l2 = doc.get_first(self.l2_field).map(|l| l.u64_value() as usize);

        let text = doc
          .get_first(self.text_field)
          .and_then(|t| t.text())
          .unwrap_or("");

        let book_id = doc
          .get_first(self.book_field)
          .and_then(|f| f.text())
          .ok_or_else(|| {
            anyhow!(
              "Got a document without a book_id - {}",
              self.searcher.schema().to_json(&doc)
            )
          })?;

        Ok(TextResult {
          book_id: String::from(book_id),
          text: String::from(text),
          l0,
          l1,
          l2,
        })
      })
      .collect::<Result<Vec<_>>>()
  }

  pub fn search(&self, query_text: &str, book_ids: &[String]) -> Result<Vec<SearchResult>> {
    let mut parser = tantivy::query::QueryParser::for_index(self.index, vec![self.text_field]);
    parser.set_conjunction_by_default();

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
        let snippet_fragment = snippet.fragments();

        let snippet_base_location = if text.len() == snippet_fragment.len() {
          0
        } else {
          text.find(snippet_fragment).unwrap_or(0)
        };

        let snippet_indexes = snippet
          .highlighted()
          .iter()
          .map(|s| {
            let b = s.bounds();
            (b.0 + snippet_base_location, b.1 + snippet_base_location)
          })
          .collect::<Vec<_>>();

        Ok(SearchResult {
          score,
          book_id: String::from(book_id),
          l0,
          l1,
          l2,
          text: String::from(text),
          highlight: snippet_indexes,
        })
      })
      .collect::<Result<Vec<_>>>()
  }
}

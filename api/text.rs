use anyhow::{anyhow, Result};
use now_lambda::{http::StatusCode, lambda, Request};
use serde::Deserialize;
use std::path::Path;
use std::rc::Rc;

use lib::{respond, RequestError, Response};

use study_map_index::{index::*, search::*};

#[derive(Deserialize)]
struct Qs {
  book_id: String,
  l0: usize,
  l1: Option<usize>,
  l2: Option<usize>,
}

struct Context<'a> {
  // index: Rc<tantivy::Index>,
  searcher: Searcher<'a>,
}

fn handler(ctx: &Context, req: Request) -> Result<Response, RequestError> {
  let q = req.uri().query().unwrap_or("");
  let qs: Qs = serde_qs::Config::new(1, false).deserialize_str(q)?;

  let results = ctx.searcher.get_text(&qs.book_id, qs.l0, qs.l1, qs.l2)?;
  let output = serde_json::to_string(&results).map_err(anyhow::Error::new)?;

  Ok(Response {
    code: StatusCode::OK,
    content_type: "application/json",
    data: output,
  })
}

// Start the runtime with the handler
fn main() -> anyhow::Result<()> {
  let index_dir = Path::new("./data");
  let index = Rc::new(open_readonly_index(index_dir).map_err(|e| anyhow!("Opening index: {}", e))?);
  let searcher = Searcher::new(&index)?;

  let ctx = Context {
    // index: index.clone(),
    searcher,
  };

  let handler_wrapper = |req: Request| respond(handler(&ctx, req));

  Ok(lambda!(handler_wrapper))
}

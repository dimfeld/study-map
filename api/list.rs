use anyhow::{anyhow, Result};
use now_lambda::{error::NowError, http::StatusCode, lambda, IntoResponse, Request, Response};
use serde_json;
use std::env;
use std::error::Error;
use std::path::Path;
use tantivy;

use study_map_index::index::*;

struct Context {
  index: tantivy::Index,
}

fn handler(ctx: &Context, req: Request) -> Result<impl IntoResponse, NowError> {
  let output = format!("{:?}", ctx.index.schema().fields().collect::<Vec<_>>());
  let response = Response::builder()
    .status(StatusCode::OK)
    .header("Content-Type", "text/plain")
    .body(output)
    .expect("Internal Server Error");

  Ok(response)
}

// Start the runtime with the handler
fn main() -> Result<()> {
  let index_dir = Path::new("./data");
  let index = open_index(index_dir).map_err(|e| anyhow!("Opening index: {}", e))?;

  let ctx = Context { index };

  let handler_wrapper = |req: Request| handler(&ctx, req);

  Ok(lambda!(handler_wrapper))
}

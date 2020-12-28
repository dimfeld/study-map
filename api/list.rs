use anyhow::anyhow;
use now_lambda::{http::StatusCode, lambda, Request};
use std::path::Path;

use lib::{respond, RequestError, Response};

use study_map_index::{index::*, search::*};

fn handler(req: Request) -> Result<Response, RequestError> {
  let output = String::new();
  Ok(Response {
    code: StatusCode::OK,
    content_type: "text/plain",
    data: output,
  })
}

// Start the runtime with the handler
fn main() -> anyhow::Result<()> {
  let index_dir = Path::new("./data");
  let handler_wrapper = |req: Request| respond(handler(req));

  Ok(lambda!(handler_wrapper))
}

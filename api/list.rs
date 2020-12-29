use lib::{respond, RequestError, Response};
use now_lambda::{http::StatusCode, lambda, Request};
use std::io::Read;

fn handler(_req: Request) -> Result<Response, RequestError> {
  let path = std::path::Path::new("./data/catalog.json");
  let mut f = std::fs::File::open(path)?;
  let mut catalog_data = String::new();
  f.read_to_string(&mut catalog_data)?;

  Ok(Response {
    code: StatusCode::OK,
    content_type: "application/json",
    data: catalog_data,
  })
}

// Start the runtime with the handler
fn main() -> anyhow::Result<()> {
  Ok(lambda!(|req| respond(handler(req))))
}

use lib::{respond, RequestError, Response};
use now_lambda::{http::StatusCode, lambda, Request};
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
struct Qs {
  book_id: String,
}

fn handler(req: Request) -> Result<Response, RequestError> {
  let q = req.uri().query().unwrap_or("");
  let qs: Qs = serde_qs::Config::new(1, false).deserialize_str(q)?;

  let data_dir_path = std::path::Path::new("./data").canonicalize()?;
  let path = data_dir_path
    .join(format!("stats-{}.json", qs.book_id))
    .canonicalize()?;

  if !path.ancestors().any(|parent| parent == data_dir_path) {
    // The request is attempting some path trickery.
    return Err(RequestError::NotFoundError);
  }

  let mut f = std::fs::File::open(path).map_err(|_e| RequestError::NotFoundError)?;
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

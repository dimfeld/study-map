use now_lambda::http::StatusCode;

pub struct Response {
  pub code: now_lambda::http::StatusCode,
  pub content_type: &'static str,
  pub data: String,
}

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
  #[error("Invalid query string: {0}")]
  QueryStringError(#[from] serde_qs::Error),

  #[error("Not found")]
  NotFoundError,

  #[error("Search Error: {0}")]
  SearchError(#[from] study_map_index::Error),

  #[error(transparent)]
  IoError(#[from] std::io::Error),

  #[error(transparent)]
  Other(#[from] anyhow::Error),
}

impl RequestError {
  pub fn status_code(&self) -> StatusCode {
    match self {
      RequestError::QueryStringError(_) => StatusCode::BAD_REQUEST,
      RequestError::NotFoundError => StatusCode::NOT_FOUND,
      RequestError::SearchError(study_map_index::Error::QueryParseError) => StatusCode::BAD_REQUEST,
      _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
  }
}

pub fn respond(
  r: Result<Response, RequestError>,
) -> Result<now_lambda::Response<String>, now_lambda::http::Error> {
  let res = match r {
    Ok(res) => res,
    Err(e) => Response {
      code: e.status_code(),
      content_type: "text/plain",
      data: e.to_string(),
    },
  };

  now_lambda::Response::builder()
    .status(res.code)
    .header("Content-Type", res.content_type)
    .header("Cache-Control", "max-age=300, s-maxage=31536000")
    .body(res.data)
}

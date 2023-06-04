#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error returned by curl crate.
    #[error("curl request failed")]
    Curl(#[source] curl::Error),
    #[error("HTTP error")]
    Http(#[source] http::Error),
    #[error("IO error: {}", _0)]
    IO(String),
    #[error("Other error: {}", _0)]
    Other(String),
}

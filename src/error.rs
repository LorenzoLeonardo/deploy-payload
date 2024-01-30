#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {}", _0)]
    IO(String),
    #[error("Curl error: {}", _0)]
    Curl(String),
}

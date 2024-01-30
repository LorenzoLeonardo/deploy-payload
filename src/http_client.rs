// 3rd party crates
use anyhow::Result;
use async_curl::actor::CurlActor;
use curl_http_client::collector::Collector;
use curl_http_client::http_client::HttpClient;
use curl_http_client::request::HttpRequest;
use curl_http_client::response::HttpResponse;

use crate::error::Error;

///
/// Asynchronous HTTP client.
///
pub async fn download_file(request: HttpRequest) -> Result<HttpResponse> {
    log::debug!("Request: {:?}", request);

    let curl_actor = CurlActor::new();
    let collector = Collector::Ram(Vec::new());

    let response = HttpClient::new(curl_actor, collector)
        .request(request)
        .map_err(|e| Error::Curl(e.to_string()))?
        .perform()
        .await
        .map_err(|e| Error::Curl(e.to_string()))?;

    log::debug!("Response: {:?}", response);
    Ok(response)
}

// Standard libraries
use std::fmt;

// 3rd party crates
use anyhow::Result;
use async_curl::async_curl::AsyncCurl;
use async_curl::response_handler::ResponseHandler;
use curl::easy::Easy2;
use http::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use http::method::Method;
use http::status::StatusCode;
use url::Url;

use crate::error::Error;

#[derive(Clone, Debug)]
pub struct HttpRequest {
    pub url: Url,
    pub method: http::method::Method,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct HttpResponse {
    pub status_code: http::status::StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

#[derive(Clone)]
struct DebugHttpRequest {
    url: Url,
    body: Vec<u8>,
    header: HeaderMap<HeaderValue>,
    method: Method,
}

impl From<&HttpRequest> for DebugHttpRequest {
    fn from(value: &HttpRequest) -> Self {
        Self {
            url: value.url.to_owned(),
            body: value.body.to_owned(),
            header: value.headers.to_owned(),
            method: value.method.to_owned(),
        }
    }
}

impl fmt::Display for DebugHttpRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Request:\n\tUrl:{}\n\tMethod:{}\n\tHeader:{:?}\n\tBody:{}",
            self.url,
            self.method,
            self.header,
            String::from_utf8(self.body.to_owned()).unwrap_or(String::new())
        )
    }
}

///
/// Asynchronous HTTP client.
///
pub async fn download_file(request: HttpRequest) -> Result<HttpResponse> {
    log::debug!("{}", DebugHttpRequest::from(&request));
    let curl = AsyncCurl::new();
    let mut easy = Easy2::new(ResponseHandler::new());

    easy.url(&request.url.to_string()[..])?;

    let mut headers = curl::easy::List::new();
    request.headers.iter().try_for_each(|(name, value)| {
        headers
            .append(&format!(
                "{}: {}",
                name,
                value.to_str().map_err(|_| Error::Other(format!(
                    "invalid {} header value {:?}",
                    name,
                    value.as_bytes()
                )))?
            ))
            .map_err(|e| Error::Curl(e))
    })?;

    easy.http_headers(headers)?;

    if let Method::POST = request.method {
        easy.post(true)?;
        easy.post_field_size(request.body.len() as u64)?;
    } else {
        assert_eq!(request.method, Method::GET);
    }

    if request.method == Method::POST {
        let form_slice = &request.body[..];
        easy.post_fields_copy(form_slice)?;
    }

    let mut easy = curl.send_request(easy).await?;

    let data = easy.get_ref().to_owned().get_data();
    let status_code = easy.response_code()? as u16;

    let response_header = easy
        .content_type()
        .map_err(|e| Error::Curl(e))?
        .map(|content_type| {
            Ok::<HeaderMap, Error>(
                vec![(
                    CONTENT_TYPE,
                    HeaderValue::from_str(content_type).map_err(|err| Error::Http(err.into()))?,
                )]
                .into_iter()
                .collect::<HeaderMap>(),
            )
        })
        .transpose()?
        .unwrap_or_else(HeaderMap::new);

    log::debug!("Response:");
    log::debug!("Header:{:?}", &response_header);
    log::debug!("Body:{}", String::from_utf8(data.to_owned())?,);
    log::debug!("Status Code:{}", &status_code);
    Ok(HttpResponse {
        status_code: StatusCode::from_u16(status_code)?,
        headers: response_header,
        body: data,
    })
}

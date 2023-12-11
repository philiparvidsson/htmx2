use crate::{Htmx, HtmxResponse, HtmxResult};
use axum::{
    body::Bytes,
    extract::OriginalUri,
    http::{HeaderMap, HeaderName, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post, Router},
};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub struct HtmxReq {
    headers: HeaderMap,
    body: Vec<u8>,
}

impl HtmxReq {
    pub fn get_header(&self, name: &str) -> Option<String> {
        self.headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers
            .append(HeaderName::try_from(name).unwrap(), value.parse().unwrap());
        println!("{} {}", name, value);
    }

    pub fn body_bytes(&self) -> &Vec<u8> {
        &self.body
    }
}

#[derive(Debug)]
pub struct HtmxRes {
    headers: HeaderMap,
    status_code: u16,
}

impl HtmxRes {
    pub fn get_header(&self, name: &str) -> Option<String> {
        self.headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers
            .append(HeaderName::try_from(name).unwrap(), value.parse().unwrap());
    }

    pub fn set_status_code(&mut self, status_code: u16) {
        self.status_code = status_code
    }

    pub fn redirect(&mut self, url: &str) -> HtmxResponse {
        self.set_header("Location", url);
        self.set_status_code(302);
        HtmxResponse::Empty
    }
}

pub async fn htmx_handler(
    OriginalUri(uri): OriginalUri,
    req_headers: HeaderMap,
    req_body: Bytes,
) -> impl IntoResponse {
    let path = uri.path();

    let htmx = Htmx {
        req: HtmxReq {
            headers: req_headers,
            body: req_body.to_vec(),
        },
        res: HtmxRes {
            headers: HeaderMap::default(),
            status_code: 200,
        },
    };

    for handler in inventory::iter::<HtmxFn>() {
        if handler.path == path {
            let (htmx, result) = (handler.func)(htmx).await;
            let status = StatusCode::from_u16(htmx.res.status_code).unwrap();
            return match result {
                Ok(HtmxResponse::Html(html)) => {
                    (status, htmx.res.headers, Html(html)).into_response()
                }
                Ok(HtmxResponse::Empty) => (status, htmx.res.headers, Html("")).into_response(),
                Err(err) => {
                    let err_response = err.downcast_ref::<HtmxResponse>();
                    match err_response {
                        Some(HtmxResponse::Html(html)) => {
                            (status, htmx.res.headers, Html(html.clone())).into_response()
                        }
                        Some(HtmxResponse::Empty) => {
                            (status, htmx.res.headers, Html("")).into_response()
                        }
                        _ => (status, htmx.res.headers, format!("{err}")).into_response(),
                    }
                }
                _ => todo!(),
            };
        }
    }

    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

pub struct HtmxFn {
    pub func: fn(Htmx) -> Pin<Box<dyn Future<Output = (Htmx, HtmxResult)> + Send>>,
    pub path: &'static str,
}

pub trait RouterExt {
    fn with_htmx_routes(self) -> Self;
}

impl RouterExt for Router {
    fn with_htmx_routes(self) -> Self {
        self.route("/api/*path", post(htmx_handler))
            .route("/*path", get(htmx_handler))
            .route("/", get(htmx_handler))
    }
}

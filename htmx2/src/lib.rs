use cfg_if::cfg_if;
pub use htmx2_macros::*;
pub use inventory;
use std::{error::Error, fmt};

pub mod cookie;
pub mod utils;

use cookie::CookieBuilder;

cfg_if! { if #[cfg(feature = "axum")] {
    mod axum_support;
    pub use axum_support::*;
} }

cfg_if! { if #[cfg(feature = "maud")] {
    mod maud_support;
    pub use maud_support::*;
} }

#[derive(Debug)]
pub struct Htmx {
    pub req: HtmxReq,
    pub res: HtmxRes,
}

impl Htmx {
    pub fn navigate(&mut self, url: &str) -> HtmxResponse {
        self.res.set_header("HX-Redirect", url);
        HtmxResponse::Empty
    }

    pub fn cookie<'a>(&'a mut self, name: &'a str) -> CookieBuilder {
        CookieBuilder::new(self, name)
    }
}

#[derive(Debug)]
pub enum HtmxResponse {
    Html(String),
    Json(String),
    Empty,
}

impl fmt::Display for HtmxResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HtmxResponse::Html(_) => write!(f, "HtmxResponse::String()"),
            HtmxResponse::Json(_) => write!(f, "HtmxResponse::Json()"),
            HtmxResponse::Empty => write!(f, "HtmxResponse::Empty()"),
        }
    }
}

impl Error for HtmxResponse {}

pub type HtmxResult = Result<HtmxResponse, Box<dyn Error>>;

// HtmxFn provided by *_support.rs
inventory::collect!(HtmxFn);

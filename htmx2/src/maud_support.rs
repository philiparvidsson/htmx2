use crate::HtmxResponse;
use maud::PreEscaped;

impl From<PreEscaped<String>> for HtmxResponse {
    fn from(value: PreEscaped<String>) -> Self {
        HtmxResponse::Html(String::from(value))
    }
}

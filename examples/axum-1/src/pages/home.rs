use htmx2::*;
use maud::html;

use crate::auth::Auth as _;
use crate::page::page;

#[htmx("/")]
fn home_page() {
    htmx.auth_guard()?;

    page(html! {
        p {
            "Hello "(htmx.username().unwrap())
        }
    })
}

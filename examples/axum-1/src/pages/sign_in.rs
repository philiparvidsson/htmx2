use htmx2::*;
use maud::html;

use crate::auth::Auth as _;
use crate::page::page;

#[htmx("/sign-in")]
fn sign_in_page() {
    page(html! {
        form class="w-80 mx-auto p-4 bg-blue-200 rounded flex flex-col gap-2 mt-20" {
            label class="flex flex-col gap-1" {
                "Username"
                input type="text" name="username";
            }

            label class="flex flex-col gap-1" {
                "Password"
                input type="password" name="password";
            }

            button type="submit" class="p-4 bg-blue-500 text-white rounded"
                hx-post=(sign_in) hx-target="#sign-in-result"
            {
                "Sign in"
            }

            div id="sign-in-result";
        }
    })
}

#[htmx]
fn sign_in(username: String, password: String) {
    if htmx.auth(&username, &password) {
        return htmx.navigate("/");
    }

    html! {
        p { "Wrong username/password" }
    }
}

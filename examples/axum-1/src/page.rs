use maud::{html, Markup, DOCTYPE};

pub fn page(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                link rel="stylesheet" href="pub/styles.css";
            }

            body class="text-sm" {
                script src="https://unpkg.com/htmx.org@1.9.9" {}
                (content)
            }
        }
    }
}

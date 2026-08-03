//! Root layout: HTML document, navigation, and the token layer.

use topcoat::{Result, router::layout, view::view};

use super::styles::TOKENS_CSS;

/// Wraps every page in a full document with the design token layer and nav.
#[layout("/")]
pub async fn root(slot: Result) -> Result {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>"Staple"</title>
                <style>(TOKENS_CSS)</style>
            </head>
            <body>
                <nav class="app-nav">
                    <a href="/">"Staple"</a>
                    <a href="/">"Companies"</a>
                </nav>
                <main class="app-main">(slot?)</main>
            </body>
        </html>
    }
}

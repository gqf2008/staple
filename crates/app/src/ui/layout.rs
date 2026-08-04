//! Root layout: HTML document, localized navigation, token layer, and the
//! language switcher.

use topcoat::{Result, context::Cx, router::layout, view::view};

use crate::i18n::{Lang, lang_code, lang_from_request, t, with_lang};

use super::styles::TOKENS_CSS;

/// Wraps every page in a full document with the design token layer, the
/// localized nav, and the language switcher.
#[layout("/")]
pub async fn root(cx: &Cx, slot: Result) -> Result {
    let lang = lang_from_request(cx);
    let parts = topcoat::context::try_request_context::<http::request::Parts>(cx);
    let current_path = parts
        .map(|parts| parts.uri.path().to_owned())
        .unwrap_or_else(|| "/".to_owned());
    let switch_lang = match lang {
        Lang::En => Lang::ZhCn,
        Lang::ZhCn => Lang::ZhTw,
        Lang::ZhTw => Lang::En,
    };
    let switch_label = match switch_lang {
        Lang::En => "English",
        Lang::ZhCn => "中文",
        Lang::ZhTw => "繁體",
    };
    let html_lang = lang_code(lang);
    view! {
        <!DOCTYPE html>
        <html lang=(html_lang)>
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>(t(lang, "nav.title"))</title>
                <style>(TOKENS_CSS)</style>
            </head>
            <body>
                <nav class="app-nav">
                    <a href=(with_lang("/", lang))>(t(lang, "nav.title"))</a>
                    <a href=(with_lang("/", lang))>(t(lang, "nav.companies"))</a>
                    <a href=(with_lang("/instance/settings", lang))>(t(lang, "instance.title"))</a>
                    <a href=(with_lang("/adapters", lang))>(t(lang, "adapters.title"))</a>
                    <a href=(with_lang(&current_path, switch_lang))>(switch_label)</a>
                </nav>
                <main class="app-main">(slot?)</main>
            </body>
        </html>
    }
}

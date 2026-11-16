use std::fmt::Write;

use axum::{
    extract::Path,
    response::{Html, IntoResponse, Response},
};
use orchestrata_application::lead_preview::{LeadPreview, example_catalog, preview};

use super::lead_preview_controller::preview_error;

const PATH: &str = "/examples/lead-to-contact";
const NOTICE: &str =
    "Synthetic preview only. No real leads, contacts, or records are created or changed.";

pub async fn catalog() -> Response {
    let catalog = match example_catalog() {
        Ok(catalog) => catalog,
        Err(error) => return preview_error(error),
    };

    let mut body = format!(
        "<p>{NOTICE}</p><p>Choose one of {} built-in examples.</p><ol>",
        catalog.lead_ids.len()
    );
    for id in catalog.lead_ids {
        let id = html_text(&id);
        write!(body, r#"<li><a href="{PATH}/{id}">{id}</a></li>"#).expect("write to String");
    }
    body.push_str("</ol>");
    page("Lead-to-contact examples", &body).into_response()
}

pub async fn view(Path(id): Path<String>) -> Response {
    match preview(&id) {
        Ok(result) => render_preview(&result).into_response(),
        Err(error) => preview_error(error),
    }
}

fn render_preview(result: &LeadPreview) -> Html<String> {
    let representative = html_text(
        result
            .assigned_representative_id
            .as_deref()
            .unwrap_or("None"),
    );
    let contact = html_text(result.contact_id.as_deref().unwrap_or("None"));
    let id = html_text(&result.lead_id);
    let body = format!(
        r#"<p>{NOTICE}</p><dl>
<dt>Example</dt><dd>{id}</dd>
<dt>Outcome</dt><dd>{:?}</dd>
<dt>Reason</dt><dd>{:?}</dd>
<dt>Assigned representative</dt><dd>{representative}</dd>
<dt>Simulated contact identifier</dt><dd>{contact}</dd></dl>
<p><a href="{PATH}">Browse all built-in examples</a></p>"#,
        result.outcome, result.reason,
    );
    page("Lead-to-contact preview", &body)
}

fn page(title: &str, body: &str) -> Html<String> {
    Html(format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>{title}</title>
<style>body{{font:1.1rem/1.55 system-ui,sans-serif;max-width:44rem;margin:2rem auto;padding:0 1rem}}li{{margin:.25rem 0}}dt{{font-weight:700}}dd{{margin:0 0 .65rem}}</style>
</head><body><main><h1>{title}</h1>{body}</main></body></html>"#,
    ))
}

fn html_text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

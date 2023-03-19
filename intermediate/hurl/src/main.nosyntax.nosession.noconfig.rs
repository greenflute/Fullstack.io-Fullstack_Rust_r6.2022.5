use structopt::StructOpt;
use heck::TitleCase;
use log::trace;

mod app;
mod client;
mod errors;

use errors::HurlResult;

type OrderedJson = std::collections::BTreeMap<String, serde_json::Value>;

fn main() -> HurlResult<()> {
    let mut app = app::App::from_args();
    app.validate()?;

    if let Some(level) = app.log_level() {
        std::env::set_var("RUST_LOG", format!("hurl={}", level));
        pretty_env_logger::init();
    }

    match app.cmd {
        Some(ref method) => {
            let resp = client::perform_method(&app, method)?;
            handle_response(resp)
        }
        None => {
            let url = app.url.take().unwrap();
            let has_data = app.parameters.iter().any(|p| p.is_data());
            let method = if has_data {
                reqwest::Method::POST
            } else {
                reqwest::Method::GET
            };
            let resp = client::perform(&app, method, &url, &app.parameters)?;
            handle_response(resp)
        }
    }
}

fn handle_response(
    mut resp: reqwest::Response,
) -> HurlResult<()> {
    let status = resp.status();
    let mut s = format!(
        "{:?} {} {}\n",
        resp.version(),
        status.as_u16(),
        status.canonical_reason().unwrap_or("Unknown")
    );
    let mut headers = Vec::new();
    for (key, value) in resp.headers().iter() {
        let nice_key = key.as_str().to_title_case().replace(' ', "-");
        headers.push(format!(
            "{}: {}",
            nice_key,
            value.to_str().unwrap_or("BAD HEADER VALUE")
        ));
    }
    let result = resp.text()?;
    let content_length = match resp.content_length() {
        Some(len) => len,
        None => result.len() as u64,
    };
    headers.push(format!("Content-Length: {}", content_length));
    headers.sort();
    s.push_str(&(&headers[..]).join("\n"));
    println!("{}", s);

    let result_json: serde_json::Result<OrderedJson> = serde_json::from_str(&result);
    match result_json {
        Ok(result_value) => {
            let result_str = serde_json::to_string_pretty(&result_value)?;
            println!("{}", result_str);
        }
        Err(e) => {
            trace!("Failed to parse result to JSON: {}", e);
            println!("{}", result);
        }
    }

    Ok(())
}

use crate::app::{App, Method, Parameter};
use reqwest::{Client, RequestBuilder, Response, Url};
use serde_json::Value;
use std::collections::HashMap;

pub fn perform_method(
    app: &App,
    method: &Method,
) -> Result<Response, Box<dyn std::error::Error>> {
    let method_data = method.data();
    perform(
        app,
        method.into(),
        &method_data.url,
        &method_data.parameters,
    )
}

pub fn perform(
    app: &App,
    method: reqwest::Method,
    raw_url: &str,
    parameters: &Vec<Parameter>,
) -> Result<Response, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = parse(app, raw_url)?;

    let mut builder = client.request(method, url);
    builder = handle_parameters(builder, app.form, parameters)?;
    builder = handle_auth(builder, &app.auth, &app.token)?;

    builder.send().map_err(From::from)
}

fn handle_auth(
    mut builder: RequestBuilder,
    auth: &Option<String>,
    token: &Option<String>,
) -> Result<RequestBuilder, Box<dyn std::error::Error>> {
    if let Some(auth_string) = auth {
        let (username, maybe_password) = parse_auth(&auth_string)?;
        builder = builder.basic_auth(username, maybe_password);
    }
    if let Some(bearer) = token {
        builder = builder.bearer_auth(bearer);
    }
    Ok(builder)
}

fn handle_parameters(
    mut builder: RequestBuilder,
    is_form: bool,
    parameters: &Vec<Parameter>,
) -> Result<RequestBuilder, Box<dyn std::error::Error>> {
    let mut data: HashMap<&String, Value> = HashMap::new();

    for param in parameters.iter() {
        match param {
            Parameter::Header { key, value } => {
                builder = builder.header(key, value);
            }
            Parameter::Data { key, value } => {
                data.insert(key, Value::String(value.to_owned()));
            }
        }
    }

    if !data.is_empty() {
        if is_form {
            builder = builder.form(&data);
        } else {
            builder = builder.json(&data);
        }
    }

    Ok(builder)
}

fn parse(app: &App, s: &str) -> Result<Url, reqwest::UrlError> {
    if s.starts_with(":/") {
        return Url::parse(&format!("http://localhost{}", &s[1..]));
    } else if s.starts_with(":") {
        return Url::parse(&format!("http://localhost{}", s));
    }
    match Url::parse(s) {
        Ok(url) => Ok(url),
        Err(_e) => {
            if app.secure {
                Url::parse(&format!("https://{}", s))
            } else {
                Url::parse(&format!("http://{}", s))
            }
        }
    }
}

fn parse_auth(s: &str) -> Result<(String, Option<String>), Box<dyn std::error::Error>> {
    if let Some(idx) = s.find(':') {
        let (username, password_with_colon) = s.split_at(idx);
        let password = password_with_colon.trim_start_matches(':');
        if password.is_empty() {
            return Ok((username.to_owned(), None));
        } else {
            return Ok((username.to_owned(), Some(password.to_owned())));
        }
    } else {
        let password = rpassword::read_password_from_tty(Some("Password: "))?;
        return Ok((s.to_owned(), Some(password)));
    }
}

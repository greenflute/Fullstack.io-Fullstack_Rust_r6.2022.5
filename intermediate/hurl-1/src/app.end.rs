use std::convert::TryFrom;
use std::path::PathBuf;
use structopt::StructOpt;

/// A command line HTTP client
#[derive(StructOpt, Debug)]
#[structopt(name = "hurl")]
pub struct App {
    /// Activate quiet mode.
    #[structopt(short, long)]
    pub quiet: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.).
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Form mode.
    #[structopt(short, long)]
    pub form: bool,

    /// Basic authentication.
    #[structopt(short, long)]
    pub auth: Option<String>,

    /// Bearer token authentication.
    #[structopt(short, long)]
    pub token: Option<String>,

    /// Session name.
    #[structopt(long)]
    pub session: Option<String>,

    /// Session storage location.
    #[structopt(long, parse(from_os_str))]
    pub session_dir: Option<PathBuf>,

    /// If true then use the stored session to augment the request,
    /// but do not modify what is stored.
    #[structopt(long)]
    pub read_only: bool,

    /// Default transport.
    #[structopt(short, long)]
    pub secure: bool,

    /// Configuration file.
    #[structopt(short, long, env = "HURL_CONFIG", parse(from_os_str))]
    pub config: Option<PathBuf>,

    /// The HTTP Method to use, one of: HEAD, GET, POST, PUT, PATCH, DELETE.
    #[structopt(subcommand)]
    pub cmd: Option<Method>,

    /// The URL to issue a request to if a method subcommand is not specified.
    pub url: Option<String>,

    /// The parameters for the request if a method subcommand is not specified.
    #[structopt(parse(try_from_str = parse_param))]
    pub parameters: Vec<Parameter>,
}

impl App {
    pub fn validate(&mut self) -> Result<(), ()> {
        if self.cmd.is_none() && self.url.is_none() {
            return Err(());
        }
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "screaming_snake_case")]
pub enum Method {
    HEAD(MethodData),
    GET(MethodData),
    PUT(MethodData),
    POST(MethodData),
    PATCH(MethodData),
    DELETE(MethodData),
}

impl Method {
    pub fn data(&self) -> &MethodData {
        use Method::*;
        match self {
            HEAD(x) => x,
            GET(x) => x,
            PUT(x) => x,
            POST(x) => x,
            PATCH(x) => x,
            DELETE(x) => x,
        }
    }
}

impl From<&Method> for reqwest::Method {
    fn from(m: &Method) -> reqwest::Method {
        match m {
            Method::HEAD(_) => reqwest::Method::HEAD,
            Method::GET(_) => reqwest::Method::GET,
            Method::PUT(_) => reqwest::Method::PUT,
            Method::POST(_) => reqwest::Method::POST,
            Method::PATCH(_) => reqwest::Method::PATCH,
            Method::DELETE(_) => reqwest::Method::DELETE,
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct MethodData {
    /// The URL to request.
    pub url: String,

    /// The headers, data, and query parameters to add to the request.
    #[structopt(parse(try_from_str = parse_param))]
    pub parameters: Vec<Parameter>,
}

#[derive(Debug)]
pub enum Parameter {
    // :
    Header { key: String, value: String },
    // =
    Data { key: String, value: String },
}

impl Parameter {
    pub fn is_data(&self) -> bool {
        match *self {
            Parameter::Header { .. } => false,
            _ => true,
        }
    }
}

#[derive(Debug)]
enum Separator {
    Colon,
    Equal,
}

impl TryFrom<&str> for Separator {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            ":" => Ok(Separator::Colon),
            "=" => Ok(Separator::Equal),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
enum Token<'a> {
    Text(&'a str),
    Escape(char),
}

fn gather_escapes<'a>(src: &'a str) -> Vec<Token<'a>> {
    let mut tokens = Vec::new();
    let mut start = 0;
    let mut end = 0;
    let mut chars = src.chars();
    loop {
        let a = chars.next();
        if a.is_none() {
            if start != end {
                tokens.push(Token::Text(&src[start..end]));
            }
            return tokens;
        }
        let c = a.unwrap();
        if c != '\\' {
            end += 1;
            continue;
        }
        let b = chars.next();
        if b.is_none() {
            tokens.push(Token::Text(&src[start..end + 1]));
            return tokens;
        }
        let c = b.unwrap();
        match c {
            '\\' | '=' | ':' => {
                if start != end {
                    tokens.push(Token::Text(&src[start..end]));
                }
                tokens.push(Token::Escape(c));
                end += 2;
                start = end;
            }
            _ => end += 2,
        }
    }
}

fn parse_param(src: &str) -> Result<Parameter, &'static str> {
    let separators = ["=", ":"];
    let tokens = gather_escapes(src);

    let mut found = Vec::new();
    let mut idx = 0;
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::Text(s) => {
                for sep in separators.iter() {
                    if let Some(n) = s.find(sep) {
                        found.push((n, sep));
                    }
                }
                if !found.is_empty() {
                    idx = i;
                    break;
                }
            }
            Token::Escape(_) => {}
        }
    }
    if found.is_empty() {
        return Err("missing separator");
    }
    found.sort_by(|(ai, asep), (bi, bsep)| ai.cmp(bi).then(bsep.len().cmp(&asep.len())));
    let sep = found.first().unwrap().1;

    let mut key = String::new();
    let mut value = String::new();
    for (i, token) in tokens.iter().enumerate() {
        if i < idx {
            match token {
                Token::Text(s) => key.push_str(&s),
                Token::Escape(c) => {
                    key.push('\\');
                    key.push(*c);
                }
            }
        } else if i > idx {
            match token {
                Token::Text(s) => value.push_str(&s),
                Token::Escape(c) => {
                    value.push('\\');
                    value.push(*c);
                }
            }
        } else {
            if let Token::Text(s) = token {
                let parts: Vec<&str> = s.splitn(2, sep).collect();
                let k = parts.first().unwrap();
                let v = parts.last().unwrap();
                key.push_str(k);
                value.push_str(v);
            } else {
                unreachable!();
            }
        }
    }

    if let Ok(separator) = Separator::try_from(*sep) {
        match separator {
            Separator::Equal => Ok(Parameter::Data { key, value }),
            Separator::Colon => Ok(Parameter::Header { key, value }),
        }
    } else {
        unreachable!();
    }
}

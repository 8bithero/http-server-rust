use crate::http_headers::HttpHeaders;
use std::io::{self, BufRead, Error, ErrorKind};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

impl FromStr for Method {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "Unsupported HTTP method",
            )),
        }
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: Method,
    pub path: String,
    pub version: String,
    pub headers: HttpHeaders,
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    pub fn parse<R: BufRead>(reader: &mut R) -> io::Result<Self> {
        let mut buf = String::new();
        reader.read_line(&mut buf)?;
        let (method, path, version) = parse_request_line(&buf)?;
        let headers = HttpHeaders::try_from(reader as &mut dyn BufRead)?;

        let mut body = None;
        if let Some(content_length) = headers.get("Content-Length") {
            if let Ok(length) = content_length.parse::<usize>() {
                let mut buffer = vec![0; length];
                reader.read_exact(&mut buffer)?;
                body = Some(buffer);
            }
        }
        Ok(Self {
            method,
            path,
            version,
            headers,
            body,
        })
    }
}

fn parse_request_line(line: &str) -> Result<(Method, String, String), std::io::Error> {
    let mut parts = line.split_whitespace();
    Ok((
        parts
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Missing method"))?
            .parse()?,
        parts
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Missing path"))?
            .to_string(),
        parts
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Missing version"))?
            .to_string(),
    ))
}

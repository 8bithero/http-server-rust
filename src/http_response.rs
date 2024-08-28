use crate::http_headers::HttpHeaders;
use std::io::{self, Write};
use std::net::TcpStream;

#[derive(Debug)]
pub struct HttpResponse {
    status_line: String,
    headers: HttpHeaders,
    body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(status_line: String, content_type: String, body: Vec<u8>) -> Self {
        let mut headers = HttpHeaders::new();
        headers.add("Content-Type".to_string(), content_type);
        headers.add("Content-Length".to_string(), body.len().to_string());

        HttpResponse {
            status_line,
            headers,
            body,
        }
    }

    pub fn write_to(&self, stream: &mut TcpStream) -> io::Result<()> {
        stream.write_all(self.status_line.as_bytes())?;
        stream.write_all(b"\r\n")?;

        for header in self.headers.iter() {
            stream.write_all(format!("{}: {}\r\n", header.name, header.value).as_bytes())?;
        }

        stream.write_all(b"\r\n")?;
        stream.write_all(&self.body)?;

        Ok(())
    }
}

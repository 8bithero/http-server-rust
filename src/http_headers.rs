use std::{
    collections::HashMap,
    io::{self, BufRead},
};

#[derive(Debug, Clone)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

impl HttpHeader {
    fn new(name: String, value: String) -> Self {
        HttpHeader { name, value }
    }
}

#[derive(Debug, Default)]
pub struct HttpHeaders {
    headers: Vec<HttpHeader>,
    headers_map: HashMap<String, usize>,
}

impl HttpHeaders {
    pub fn new() -> Self {
        HttpHeaders {
            headers: Vec::new(),
            headers_map: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, value: String) {
        let lowercase_name = name.to_lowercase();
        let index = self.headers.len();
        self.headers.push(HttpHeader::new(name, value));
        self.headers_map.insert(lowercase_name, index);
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.headers_map
            .get(&name.to_lowercase())
            .map(|&index| self.headers[index].value.as_str())
    }

    fn get_all(&self, name: &str) -> Vec<&str> {
        self.headers
            .iter()
            .filter(|header| header.name.eq_ignore_ascii_case(name))
            .map(|header| header.value.as_str())
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &HttpHeader> {
        self.headers.iter()
    }
}

impl TryFrom<&mut dyn BufRead> for HttpHeaders {
    type Error = io::Error;

    fn try_from(reader: &mut dyn BufRead) -> Result<Self, Self::Error> {
        let mut headers = HttpHeaders::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line)?;
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            let mut parts = line.splitn(2, ':');
            let name = parts
                .next()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid header: missing name")
                })?
                .trim();
            let value = parts
                .next()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid header: missing value")
                })?
                .trim();
            headers.add(name.to_string(), value.to_string());
        }
        Ok(headers)
    }
}

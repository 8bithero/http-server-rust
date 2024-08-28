use std::{collections::HashMap, env, fs, io::Write, path::PathBuf};

use crate::{
    http_request::{HttpRequest, Method},
    http_response::HttpResponse,
};

pub enum RouteResponse {
    Ok(Vec<u8>, Option<String>),
    Created,
    NotFound,
    FileContent(Vec<u8>),
}

pub trait Handler: Send + Sync {
    fn handle(&self, request: &HttpRequest, params: &HashMap<String, String>) -> RouteResponse;
}

pub struct Router {
    routes: Vec<(Method, String, Box<dyn Handler>)>,
}

impl Router {
    pub fn new() -> Self {
        Router { routes: Vec::new() }
    }

    pub fn add_route<H: Handler + 'static>(&mut self, method: Method, path: &str, handler: H) {
        self.routes
            .push((method, path.to_string(), Box::new(handler)));
    }

    pub fn route(&self, request: &HttpRequest) -> HttpResponse {
        for (method, path, handler) in &self.routes {
            if method == &request.method {
                if let Some(path_params) = match_path(path, &request.path) {
                    return match handler.handle(request, &path_params) {
                        RouteResponse::Ok(body, content_encoding) => {
                            let mut response = HttpResponse::new(
                                "HTTP/1.1 200 OK".to_string(),
                                "text/plain".to_string(),
                                body,
                            );
                            if let Some(encoding) = content_encoding {
                                response
                                    .headers
                                    .add("Content-Encoding".to_string(), encoding);
                            }
                            response
                        }
                        RouteResponse::Created => HttpResponse::new(
                            "HTTP/1.1 201 Created".to_string(),
                            "text/plain".to_string(),
                            Vec::new(),
                        ),
                        RouteResponse::FileContent(body) => HttpResponse::new(
                            "HTTP/1.1 200 OK".to_string(),
                            "application/octet-stream".to_string(),
                            body,
                        ),
                        RouteResponse::NotFound => HttpResponse::new(
                            "HTTP/1.1 404 Not Found".to_string(),
                            "text/plain".to_string(),
                            Vec::new(),
                        ),
                    };
                }
            }
        }

        HttpResponse::new(
            "HTTP/1.1 404 Not Found".to_string(),
            "text/plain".to_string(),
            Vec::new(),
        )
    }
}

fn match_path(route_path: &str, request_path: &str) -> Option<HashMap<String, String>> {
    let route_parts: Vec<&str> = route_path.split('/').collect();
    let request_parts: Vec<&str> = request_path.split('/').collect();

    if route_parts.len() != request_parts.len() {
        return None;
    }

    let mut params = HashMap::new();

    for (route_part, request_part) in route_parts.iter().zip(request_parts.iter()) {
        if let Some(param_name) = route_part.strip_prefix(':') {
            params.insert(param_name.to_string(), request_part.to_string());
        } else if route_part != request_part {
            return None;
        }
    }
    Some(params)
}

struct RootHandler;
impl Handler for RootHandler {
    fn handle(&self, _request: &HttpRequest, _params: &HashMap<String, String>) -> RouteResponse {
        RouteResponse::Ok(Vec::new(), None)
    }
}

struct UserAgentHandler;
impl Handler for UserAgentHandler {
    fn handle(&self, request: &HttpRequest, _params: &HashMap<String, String>) -> RouteResponse {
        RouteResponse::Ok(
            request
                .headers
                .get("User-Agent")
                .unwrap_or("Unknown")
                .as_bytes()
                .to_vec(),
            None,
        )
    }
}

struct EchoHandler;
impl Handler for EchoHandler {
    fn handle(&self, request: &HttpRequest, params: &HashMap<String, String>) -> RouteResponse {
        let message = params.get("message").map(String::as_str).unwrap_or("");
        let body = message.as_bytes().to_vec();

        let accepts_gzip = request
            .headers
            .get("Accept-Encoding")
            .map(|encoding| encoding.contains("gzip"))
            .unwrap_or(false);

        if accepts_gzip {
            RouteResponse::Ok(body, Some("gzip".to_string()))
        } else {
            RouteResponse::Ok(body, None)
        }
    }
}

struct PostFileHandler;
impl Handler for PostFileHandler {
    fn handle(&self, request: &HttpRequest, params: &HashMap<String, String>) -> RouteResponse {
        let file_name = params.get("filename").map(String::as_str).unwrap_or("");
        let dir = env::args().nth(2).unwrap_or_else(|| ".".to_string());
        let mut file_path = PathBuf::from(&dir);
        file_path.push(file_name);

        if let Some(content) = &request.body {
            match fs::File::create(&file_path) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(content) {
                        eprintln!("Failed to write to file: {}", e);
                        RouteResponse::NotFound
                    } else {
                        RouteResponse::Created
                    }
                }
                Err(e) => {
                    eprintln!("Failed to create file: {}", e);
                    RouteResponse::NotFound
                }
            }
        } else {
            RouteResponse::NotFound
        }
    }
}

struct FileHandler;
impl Handler for FileHandler {
    fn handle(&self, _request: &HttpRequest, params: &HashMap<String, String>) -> RouteResponse {
        match handle_file_request(params.get("filename").unwrap_or(&"".to_string())) {
            Some(contents) => RouteResponse::FileContent(contents),
            None => RouteResponse::NotFound,
        }
    }
}

fn handle_file_request(file_name: &str) -> Option<Vec<u8>> {
    let dir = env::args().nth(2).unwrap_or_else(|| ".".to_string());
    let mut file_path = PathBuf::from(&dir);
    file_path.push(file_name);

    if file_path.exists() && file_path.is_file() {
        fs::read(&file_path).ok()
    } else {
        None
    }
}

pub fn create_router() -> Router {
    let mut router = Router::new();
    router.add_route(Method::Get, "/", RootHandler);
    router.add_route(Method::Get, "/user-agent", UserAgentHandler);
    router.add_route(Method::Get, "/echo/:message", EchoHandler);
    router.add_route(Method::Post, "/files/:filename", PostFileHandler);
    router.add_route(Method::Get, "/files/:filename", FileHandler);
    router
}

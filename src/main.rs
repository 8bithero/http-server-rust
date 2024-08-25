use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::{env, fs, thread};

const OK_RESP: &str = "HTTP/1.1 200 OK";
const NOT_FOUND_RESP: &str = "HTTP/1.1 404 Not Found";

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                thread::spawn(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut content_type = "text/plain";
    let buf_reader = BufReader::new(&mut stream);
    let mut lines = buf_reader.lines();
    let request_line = lines.next().unwrap().unwrap();

    let (status_line, contents) = match &request_line[..] {
        "GET / HTTP/1.1" => (OK_RESP, Vec::new()),
        "GET /user-agent HTTP/1.1" => {
            let user_agent = handle_user_agent(&mut lines);
            (OK_RESP, user_agent)
        }
        _ if request_line.starts_with("GET /files/") => {
            content_type = "application/octet-stream";
            match handle_file_request(&request_line) {
                Some(contents) => (OK_RESP, contents),
                None => (NOT_FOUND_RESP, Vec::new()),
            }
        }
        _ if request_line.starts_with("GET /echo") => {
            let echo_value =
                &request_line["GET /echo/".len()..request_line.len() - " HTTP/1.1".len()];
            (OK_RESP, echo_value.to_string().into_bytes())
        }
        _ => (NOT_FOUND_RESP, Vec::new()),
    };

    let headers_text = format!(
        "Content-Type: {}\r\nContent-Length: {}\r\n",
        content_type,
        contents.len()
    );
    let response = format!("{status_line}\r\n{headers_text}\r\n");

    stream.write_all(response.as_bytes()).unwrap();
    stream.write_all(&contents).unwrap();
}

fn handle_user_agent(lines: &mut std::io::Lines<std::io::BufReader<&mut TcpStream>>) -> Vec<u8> {
    lines
        .find_map(|line| {
            let line = line.unwrap();
            line.strip_prefix("User-Agent:")
                .map(|user_agent| user_agent.trim().to_string().into_bytes())
        })
        .unwrap_or_else(|| b"Unknown".to_vec())
}

fn handle_file_request(request_line: &str) -> Option<Vec<u8>> {
    let file_name = request_line
        .strip_prefix("GET /files/")?
        .trim_end_matches(" HTTP/1.1");
    let dir = env::args().nth(2).unwrap_or_else(|| ".".to_string());
    let mut file_path = PathBuf::from(&dir);
    file_path.push(file_name);

    if file_path.exists() && file_path.is_file() {
        fs::read(&file_path).ok()
    } else {
        None
    }
}

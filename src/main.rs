use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::{env, fs, thread};

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
    let ok_resp = "HTTP/1.1 200 OK";
    let not_found_resp = "HTTP/1.1 404 Not Found";

    let mut content_type = "text/plain";
    let buf_reader = BufReader::new(&mut stream);
    let mut lines = buf_reader.lines();
    let request_line = lines.next().unwrap().unwrap();

    let (status_line, contents) = match &request_line[..] {
        "GET / HTTP/1.1" => (ok_resp, Vec::new()),
        "GET /user-agent HTTP/1.1" => {
            let user_agent = lines
                .find_map(|line| {
                    let line = line.unwrap();
                    line.strip_prefix("User-Agent:")
                        .map(|user_agent| user_agent.trim().to_string().into_bytes())
                })
                .unwrap_or_else(|| "Unknown".to_string().into_bytes());
            (ok_resp, user_agent)
        }
        _ if request_line.starts_with("GET /files/") => {
            content_type = "application/octet-stream";
            match request_line.strip_prefix("GET /files/") {
                Some(file_name) => {
                    let file_name = file_name.trim_end_matches(" HTTP/1.1");
                    let dir = env::args().nth(2).unwrap_or_else(|| ".".to_string());
                    let mut file_path = PathBuf::from(&dir);
                    file_path.push(file_name);

                    if file_path.exists() && file_path.is_file() {
                        match fs::read(&file_path) {
                            Ok(contents) => (ok_resp, contents),
                            Err(_) => (not_found_resp, Vec::new()),
                        }
                    } else {
                        (not_found_resp, Vec::new())
                    }
                }
                None => (not_found_resp, Vec::new()),
            }
        }
        _ if request_line.starts_with("GET /echo") => {
            let echo_value =
                &request_line["GET /echo/".len()..request_line.len() - " HTTP/1.1".len()];
            (ok_resp, echo_value.to_owned().into_bytes())
        }
        _ => (not_found_resp, Vec::new()),
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

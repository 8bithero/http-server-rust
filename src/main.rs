use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                handle_connection(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    println!("{}", request_line);
    let (status_line, contents) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", ""),
        _ if request_line.starts_with("GET /echo") => {
            let echo_value =
                &request_line["GET /echo/".len()..request_line.len() - " HTTP/1.1".len()];
            ("HTTP/1.1 200 OK", echo_value)
        }
        _ => ("HTTP/1.1 404 Not Found", ""),
    };
    let headers_text = format!(
        "Content-Type: text/plain\r\nContent-Length: {}\r\n",
        contents.len()
    );
    let response = format!("{status_line}\r\n{headers_text}\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}

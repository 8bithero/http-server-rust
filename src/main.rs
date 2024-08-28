mod http_headers;
mod http_request;
mod http_response;
mod router;

use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use http_request::HttpRequest;
use router::{create_router, Router};

fn main() -> std::io::Result<()> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let router = Arc::new(create_router());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let router = Arc::clone(&router);
                thread::spawn(move || {
                    if let Err(e) = handle_connection2(stream, router) {
                        eprintln!("Error handling connection: {}", e);
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_connection2(mut stream: TcpStream, router: Arc<Router>) -> std::io::Result<()> {
    let mut reader = BufReader::new(&mut stream);
    let request = HttpRequest::parse(&mut reader)?;
    // println!("-----------");
    // println!("{:#?}", request);
    // println!("-----------");

    let response = router.route(&request);
    response.write_to(&mut stream)?;

    Ok(())
}

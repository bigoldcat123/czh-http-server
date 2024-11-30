use std::net::TcpListener;

use app::handle_stream;

fn main() {
    let server = TcpListener::bind("localhost:8888").unwrap();
    println!("listen at {}", 8888);
    for client in server.incoming() {
        match client {
            Ok(stream) => {
                handle_stream(stream);
            }
            Err(_) => {
                println!("Something wrong!")
            }
        }
    }
}

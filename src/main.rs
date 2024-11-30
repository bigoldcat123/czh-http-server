
use czh_http_server::HttpServer;

fn main() {
    let server  = HttpServer::create_server("localhost", 3000);

    server.listen();
    
}

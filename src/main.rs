
use czh_http_server::HttpServer;

fn main() {
    let mut server  = HttpServer::create_server("localhost", 3000);
    server.map("/file","/Users/dadigua/Desktop/lifetime/app/nextjs-static/dist");
    server.get("/home",|req,res| {
        println!("{:#?}",req.url());
        res.json("hello fetch");
    });
    server.listen();
    
}

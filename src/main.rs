//!
//! # czh_http_server
//!
//! czh_http_server is a simple http server
//! # Example
//! ```
//! let mut server  = HttpServer::create_server("localhost", 3000);
//!    // server.listen();
//!    server.filter("/home",|req,res| {
//!        println!("{:#?}","hello i am filterb");
//!        if req.url() == "/home/abc" {
//!            res.json("GLALALALALALA");
//!            return None
//!        }
//!        Some((req,res))
//!    });
//!    server.map("/file","/Users/dadigua/Desktop/lifetime/app/nextjs-static/dist");
//!
//!    server.get("/home",|req,mut res| {
//!        println!("{:#?}",req.url());
//!        // println!("{:#?}",req.headers());
//!        println!("{:#?}",req.cookies());
//!        res.set_cookie("cooooooo", "this is cookie setted by server");
//!        res.json("hello fetch");
//!    });
//!    server.get("/home/abc",|req,res| {
//!        println!("{:#?}",req.url());
//!        res.json("hello fetch/ home/abc");
//!    });
//!    
//!    server.post("/post",|mut req,res| {
//!        match req.json::<Student>() {
//!            Ok(stu) => {
//!                println!("{:#?}",stu);
//!            },
//!            Err(_) => {
//!                res.json("error parse json");
//!                return;
//!            },
//!        }
//!        println!("{:#?}",req.url());
//!        res.json("hello post");
//!    });
//!
//!    let mut route = Route::new();
//!
//!    route.get("/sayhello", |req, res| {
//!        // req.url()
//!        println!("{:#?}",req.url());
//!        res.json(Student{
//!            name:"dadigua".to_string(),
//!            age:18
//!        });
//!    });
//!
//!    server.router("/route",route);
//!    server.listen();
//! ```
//!
use czh_http_server::{route::Route, HttpHander, HttpServer};
use serde::{Deserialize, Serialize};
use serde_json::ser;

// t==========
#[derive(Debug, Serialize, Deserialize)]
struct Student {
    name: String,
    age: u8,
}

fn main() {
    
    let mut server  = HttpServer::create_server("localhost", 3000);
    // server.listen();
    server.filter("/home",|req,res| {
        println!("{:#?}","hello i am filterb");
        if req.url() == "/home/abc" {
            res.json("GLALALALALALA");
            return None
        }
        Some((req,res))
    });
    server.map("/file","/Users/dadigua/Desktop/lifetime/app/nextjs-static/dist");

    server.get("/home",|req,mut res| {
        println!("{:#?}",req.url());
        // println!("{:#?}",req.headers());
        println!("{:#?}",req.cookies());
        res.set_cookie("cooooooo", "this is cookie setted by server");
        res.json("hello fetch");
    });
    server.get("/home/abc",|req,res| {
        println!("{:#?}",req.url());
        res.json("hello fetch/ home/abc");
    });
    
    server.post("/post",|mut req,res| {
        match req.json::<Student>() {
            Ok(stu) => {
                println!("{:#?}",stu);
            },
            Err(_) => {
                res.json("error parse json");
                return;
            },
        }
        println!("{:#?}",req.url());
        res.json("hello post");
    });

    let mut route = Route::new();

    route.get("/sayhello", |req, res| {
        // req.url()
        println!("{:#?}",req.url());
        res.json(Student{
            name:"dadigua".to_string(),
            age:18
        });
    });

    server.router("/route",route);
    


    server.listen();
    
}

#[cfg(test)]
mod test {
    use crate::Student;

    #[test]
    fn it_works() {
        let stu = Student{
                    name:"dadigua".to_string(),
                    age:18
                };
        let s = serde_json::to_string(&stu).unwrap();
        println!("{}",s);
        // let s:Student = json().unwrap();
        println!("{:#?}",s);
    }
}
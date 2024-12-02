
use czh_http_server::HttpServer;
use serde::{Deserialize, Serialize};

// t==========
#[derive(Debug, Serialize, Deserialize)]
struct Student {
    name: String,
    age: u8,
}

fn main() {
    let mut server  = HttpServer::create_server("localhost", 3000);
    server.map("/file","/Users/dadigua/Desktop/lifetime/app/nextjs-static/dist");
    server.get("/home",|req,res| {
        println!("{:#?}",req.url());
        res.json("hello fetch");
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
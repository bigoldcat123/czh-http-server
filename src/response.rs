use std::{cell::RefCell, collections::HashMap, fmt::format, io::{BufWriter, Write}, net::TcpStream, ops::{Deref, DerefMut}, rc::Rc};

struct StatusLine{
    version: String,
    status_code: u16,
    reason: String
}
impl StatusLine {
    fn to_string(&self) -> String {
       format!("{} {} {}\r\n",self.version,self.status_code,self.reason)
    }
}
pub  struct HttpResponse {
    stream: Option<Rc<RefCell<TcpStream>>>,
    headers:HashMap<String,String>,
    status_line: StatusLine
}
impl HttpResponse { 
    pub fn init(stream: Rc<RefCell<TcpStream>>,version:&str) -> Self {
        let headers = init_headers();
        HttpResponse{
            stream: Some(stream),
            headers,
            status_line: StatusLine{
                version: String::from(version),
                status_code: 200,
                reason: "OK".to_string()
            }
        }
    }
    pub fn json<T>(mut self,data:T)
    where T: serde::Serialize{
        self.headers.insert(String::from("Content-Type"), "application/json".to_string());
        let json = serde_json::to_string(&data).unwrap();
        let data = json.as_bytes();
        self.headers.insert(String::from("Content-Length"), data.len().to_string());
        let stream = self.stream.take().unwrap();
        let mut stream = stream.borrow_mut();
        let mut writer = BufWriter::new(stream.deref_mut());
        let status = self.status_line.to_string();

        writer.write_all(status.as_bytes()).unwrap();
        for (key,value) in self.headers.iter(){
            let header = format!("{}:{}\r\n",key,value);
            writer.write_all(header.as_bytes()).unwrap();
        }
        writer.write_all(b"\r\n").unwrap();
        writer.write_all(data).unwrap();
    }
}

fn init_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();
    // TODO: init headers
    headers
}
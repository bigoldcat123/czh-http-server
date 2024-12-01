use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::format,
    io::{BufReader, BufWriter, Read, Write},
    net::TcpStream,
    ops::{Deref, DerefMut},
    rc::Rc,
};

struct StatusLine {
    version: String,
    status_code: u16,
    reason: String,
}
impl StatusLine {
    fn to_string(&self) -> String {
        format!("{} {} {}\r\n", self.version, self.status_code, self.reason)
    }
}
pub struct HttpResponse {
    stream: Option<Rc<RefCell<TcpStream>>>,
    headers: HashMap<String, String>,
    status_line: StatusLine,
}
impl HttpResponse {
    pub fn init(stream: Rc<RefCell<TcpStream>>, version: &str) -> Self {
        let headers = init_headers();
        HttpResponse {
            stream: Some(stream),
            headers,
            status_line: StatusLine {
                version: String::from(version),
                status_code: 200,
                reason: "OK".to_string(),
            },
        }
    }
    fn set_content_length(&mut self, len: u64) {
        self.headers
            .insert("Content-Length".to_string(), len.to_string());
    }
    fn set_content_type(&mut self, content_type: ContentType) {
        self.headers
            .insert("Content-Type".to_string(), content_type.into());
    }

    pub fn json<T>(mut self, data: T)
    where
        T: serde::Serialize,
    {
        self.headers
            .insert(String::from("Content-Type"), "application/json".to_string());
        let json = serde_json::to_string(&data).unwrap();
        let data = json.as_bytes();
        self.set_content_length(data.len() as u64);
        let stream = self.stream.take().unwrap();
        let mut stream = stream.borrow_mut();
        let mut writer: BufWriter<&mut TcpStream> = BufWriter::new(stream.deref_mut());
        self.write_status_headers(&mut writer);
        writer.write_all(data).unwrap();
    }
    fn write_status_headers(&mut self, writer: &mut BufWriter<&mut TcpStream>) {
        let status = self.status_line.to_string();
        writer.write_all(status.as_bytes()).unwrap();
        for (key, value) in self.headers.iter() {
            let header = format!("{}:{}\r\n", key, value);
            writer.write_all(header.as_bytes()).unwrap();
        }
        writer.write_all(b"\r\n").unwrap();
    }
    pub(crate) fn file(mut self, file: std::fs::File, content_type: ContentType) {
        self.set_content_length(file.metadata().unwrap().len());
        self.set_content_type(content_type);

        let stream = self.stream.take().unwrap();
        let mut stream = stream.borrow_mut();
        let mut writer: BufWriter<&mut TcpStream> = BufWriter::new(stream.deref_mut());
        self.write_status_headers(&mut writer);
        let mut reader = BufReader::new(file);
        let mut buffer = [0; 1024];
        loop {
            let size = reader.read(&mut buffer).unwrap();
            if size == 0 {
                break;
            }
            writer.write_all(&buffer[0..size]).unwrap();
        }
    }
}

fn init_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();
    // TODO: init headers
    headers
}
pub enum ContentType {
    JSON,
    HTML,
    CSS,
    JS,
    PNG,
    JPG,
    OTHER,
}
impl From<&str> for ContentType {
    fn from(suffix: &str) -> Self {
        match suffix {
            "json" => ContentType::JSON,
            "html" => ContentType::HTML,
            "css" => ContentType::CSS,
            "js" => ContentType::JS,
            "png" => ContentType::PNG,
            "jpg" => ContentType::JPG,
            "jpeg" => ContentType::JPG,
            _ => ContentType::OTHER,
        }
    }
}
impl Into<String> for ContentType {
    fn into(self) -> String {
        match self {
            ContentType::JSON => "application/json".to_string(),
            ContentType::HTML => "text/html".to_string(),
            ContentType::CSS => "text/css".to_string(),
            ContentType::JS => "text/javascript".to_string(),
            ContentType::PNG => "image/png".to_string(),
            ContentType::JPG => "image/jpeg".to_string(),
            ContentType::OTHER => "application/octet-stream".to_string(),
        }
    }
}

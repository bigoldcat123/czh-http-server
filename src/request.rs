use std::{
    cell::RefCell,
    collections::HashMap,
    io::{BufRead, BufReader, Error, Result},
    net::TcpStream,
    ops::DerefMut,
    rc::Rc,
};
#[derive(Debug)]
struct StatusLine {
    method: String,
    version: String,
    url: String,
}
#[derive(Debug)]
pub struct HttpRequest {
    status_line: StatusLine,
    headers: HashMap<String, String>,
    stream: Option<Rc<RefCell<TcpStream>>>,
}

impl HttpRequest {
    pub fn build(stream: Rc<RefCell<TcpStream>>) -> Self {
        let mut stream_ref = stream.borrow_mut();
        let mut reader = BufReader::new(stream_ref.deref_mut());

        let (method, url, version) = parse_request_row(&mut reader).unwrap();

        let mut buffer = String::new();
        let mut headers = HashMap::new();
        loop {
            let size = reader.read_line(&mut buffer).unwrap();
            if size == 2 {
                break;
            }
            let key_v = buffer.trim().split_once(':').unwrap();
            headers.insert(String::from(key_v.0.trim()), String::from(key_v.1.trim()));
            buffer.clear();
        }
        Self {
            status_line: StatusLine {
                method,
                version,
                url,
            },
            headers,
            stream: Some(stream.clone()),
        }
    }
    pub  fn version(&self) -> &str {
        &self.status_line.version
    }
}
fn parse_request_row(reader: &mut BufReader<&mut TcpStream>) -> Result<(String, String, String)> {
    let mut buffer = String::new();
    let mut method = String::new();
    let mut url = String::new();
    let mut version = String::new();
    reader.read_line(&mut buffer)?;
    let row = buffer.split_whitespace().collect::<Vec<&str>>();
    if row.len() != 3 {
        Err(Error::new(std::io::ErrorKind::Other, "error"))
    } else {
        method.push_str(row[0]);
        url.push_str(row[1]);
        version.push_str(row[2]);
        Ok((method, url, version))
    }
}

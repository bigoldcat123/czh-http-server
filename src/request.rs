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
    pub fn headers (&self) -> &HashMap<String, String> {
        &self.headers
    }
    pub  fn version(&self) -> &str {
        &self.status_line.version
    }
    
    pub fn method(&self) -> &str {
        self.status_line.method.as_str()
    }
    
    pub fn url(&self) -> &str {
        self.status_line.url.as_str()
    }
    
    pub(crate) fn prefix_url(&self) -> &str {
        let mut stop = 1;
        for i in 1..self.url().len() {
            if self.url().as_bytes()[i] == b'/' {
                stop = i;
                break;
            }
        }
        if stop == 1 {
            self.url()
        } else {
            &self.url()[0..stop]
        }
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
        let u: Vec<&str> = row[1].split("?").collect();
        method.push_str(row[0]);
        url.push_str(u[0]);
        version.push_str(row[2]);
        Ok((method, url, version))
    }
}

#[cfg(test)]
mod tests {
    use super::HttpRequest;

    #[test]
    fn it_works() {
    }
}
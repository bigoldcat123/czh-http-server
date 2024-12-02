use std::{
    cell::RefCell, collections::HashMap, fs::read, io::{BufRead, BufReader, Error, Read, Result}, net::TcpStream, ops::DerefMut, rc::Rc
};

use serde::de::{self, Error as SerdeError};


// use serde::{de::{self, Error}, Deserialize, Serialize};
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
    buffer: Vec<u8>,
}

impl HttpRequest {
    pub fn build(stream: Rc<RefCell<TcpStream>>) -> Result<Self> {
        let mut stream_ref = stream.borrow_mut();
        let mut reader = BufReader::new(stream_ref.deref_mut());

        let (method, url, version) = parse_request_row(&mut reader)?;

        let mut buffer = String::new();
        let mut headers = HashMap::new();
        
        loop {
            let size = reader.read_line(&mut buffer)?;
            if size == 2 {
                break;
            }
            let key_v = buffer.trim().split_once(':').unwrap();
            headers.insert(String::from(key_v.0.trim()), String::from(key_v.1.trim()));
            buffer.clear();
        }
        
        let buffer = Vec::from_iter(reader.buffer().iter().map(|x| *x));
        // println!("{:#?}",);
        // let buffer = vec![];
        Ok(Self {
            status_line: StatusLine {
                method,
                version,
                url,
            },
            headers,
            stream: Some(stream.clone()),
            buffer
        })
    }
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
    pub fn version(&self) -> &str {
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
    
    pub fn json<T>(&mut self) -> serde_json::Result<T>
    where
        T: de::DeserializeOwned,
    {
        let size = self.get_content_length()?;
        if size == self.buffer.len() {
            return serde_json::from_slice(&self.buffer);
        }else {
            let remind = size - self.buffer.len();
            let mut stream = self.stream.as_mut().unwrap().borrow_mut();
            let  mut reader = BufReader::new(stream.deref_mut());
            let mut total = 0;
            loop {
                let size = reader.read(self.buffer.as_mut_slice()).unwrap();
                total += size;
                if total == remind {
                    break;
                }
            }
            // serde_json::
            serde_json::from_slice(&self.buffer)
            // serde_json::from_reader(reader)
        }
    }
    fn get_content_length(&self) -> serde_json::Result<usize> {
        match self.headers
            .get("Content-Length") {
                Some(v) => Ok(v.parse().unwrap()),
                None => Err(serde_json::Error::custom("msg"))
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

    #[test]
    fn it_works() {

    }
}

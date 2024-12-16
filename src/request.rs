use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Error, Read, Result, Write},
    mem,
    net::TcpStream,
    ops::DerefMut,
    rc::Rc,
};

use serde::de::{self, Error as SerdeError};

pub enum FormDataType {
    Text(String),
    File(std::fs::File),
}
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
        //bufReader will preread some date storing in its inner buffer ,so when droping it
        //,we have to store the buffs that have already been read.
        let buffer = Vec::from_iter(reader.buffer().iter().map(|x| *x));

        Ok(Self {
            status_line: StatusLine {
                method,
                version,
                url,
            },
            headers,
            stream: Some(stream.clone()),
            buffer,
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

    pub fn cookies(&self) -> Option<HashMap<&str, &str>> {
        if let Some(cookie) = self.headers.get("Cookie") {
            let mut cookies = HashMap::new();
            let key_value = cookie.split("; ").collect::<Vec<&str>>();
            key_value.iter().for_each(|x| {
                if let Some(key_value) = x.split_once("=") {
                    cookies.insert(key_value.0, key_value.1);
                } else {
                    println!("{:#?}", "cookie parse error");
                }
            });
            Some(cookies)
        } else {
            None
        }
    }

    pub fn url(&self) -> &str {
        self.status_line.url.as_str()
    }

    /// /abc/abc => /abc
    /// / => /
    /// /efg/avc => /efg
    /// /abc => /
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
        } else {
            todo!("remind this is a bug");
            let remind = size - self.buffer.len();
            let mut stream = self.stream.as_mut().unwrap().borrow_mut();
            let mut reader = BufReader::new(stream.deref_mut());
            let mut total = 0;
            loop {
                let size = reader.read(self.buffer.as_mut_slice()).unwrap();
                total += size;
                if total == remind {
                    break;
                }
            }
            serde_json::from_slice(&self.buffer)
        }
    }
    fn get_content_length(&self) -> serde_json::Result<usize> {
        match self.headers.get("Content-Length") {
            Some(v) => Ok(v.parse().unwrap()),
            None => Err(serde_json::Error::custom("msg")),
        }
    }

    pub fn form_data(&mut self) -> Result<HashMap<String, FormDataType>> {
        fn kmp_search(text: &[u8], pattern: &[u8]) -> Option<usize> {
            let text_bytes = text;
            let pattern_bytes = pattern;
            let n = text_bytes.len();
            let m = pattern_bytes.len();

            if m == 0 {
                return Some(0); // 空模式串默认匹配
            }

            // 构建部分匹配表 (next 数组)
            let mut next = vec![0; m];
            let mut j = 0;
            for i in 1..m {
                while j > 0 && pattern_bytes[i] != pattern_bytes[j] {
                    j = next[j - 1]; // 回退
                }
                if pattern_bytes[i] == pattern_bytes[j] {
                    j += 1;
                }
                next[i] = j;
            }

            // KMP 搜索
            let mut j = 0; // 模式串指针
            for i in 0..n {
                while j > 0 && text_bytes[i] != pattern_bytes[j] {
                    j = next[j - 1]; // 回退
                }
                if text_bytes[i] == pattern_bytes[j] {
                    j += 1;
                }
                if j == m {
                    // 找到匹配，返回起始位置
                    return Some(i + 1 - m);
                }
            }

            None // 未找到匹配
        }

        let mock = "/Users/dadigua/Desktop/lifetime/app/";
        let mut res = HashMap::new();
        let boundary = self.get_boundary()?;
        let boundary = boundary.as_bytes();
        let boundary_len = boundary.len();
        let mut stream = self.stream.as_ref().unwrap().borrow_mut();
        let mut reader = BufReader::new(stream.deref_mut());
        let len = self.get_content_length()?;
        println!("{:#?}", self.headers);
        let remind = len - self.buffer.len();
        println!("remain -> {:#?}", remind);
        let mut total = 0;
        // loop {
        let mut buf = [0 as u8; 1024 * 8];
        if remind != 0 {
            let size = reader.read(&mut buf).unwrap();
            self.buffer.extend_from_slice(&buf[0..size]);
            total += size;
        }

        self.buffer.splice(0..boundary_len + 2, vec![]);

        // if self.buffer. parse header
        let mut idx_buf = 0;
        // let start = idx_buf + boundary_len;
        let mut end;
        loop {
            loop {
                if idx_buf + 3 < self.buffer.len() {
                    if self.buffer[idx_buf..idx_buf + 4].eq(b"\r\n\r\n") {
                        end = idx_buf + 4;
                        break;
                    }
                    idx_buf += 1;
                } else {
                    let size = reader.read(&mut buf).unwrap();
                    self.buffer.extend_from_slice(&buf[0..size]);
                    total += size;
                }
            }
            let headers = String::from_utf8(self.buffer[..end].to_vec()).unwrap();
            // println!("header => {}", headers);
            let headers: Vec<&str> = headers[..headers.len() - 4].split("\r\n").collect();
            println!("{:#?}", headers);
            let mut name;
            let mut file_name;
            let mut file: Option<File> = None;
            let name_key = "name=\"";

            // 查找 name 的值
            let name_start = headers[0].find(name_key).unwrap() + name_key.len();
            let name_end = headers[0][name_start..].find('"').unwrap() + name_start;
            name = headers[0][name_start..name_end].to_string();
            if headers.len() == 2 {
                let filename_key = "filename=\"";
                // 查找 filename 的值
                let filename_start = headers[0].find(filename_key).unwrap() + filename_key.len();
                let filename_end = headers[0][filename_start..].find('"').unwrap() + filename_start;
                // 提取并返回
                file_name = headers[0][filename_start..filename_end].to_string();
                file = Some(
                    File::options()
                        .append(true)
                        .create(true)
                        .open(format!("{}{}", mock, file_name))
                        .unwrap(),
                );
            }
            self.buffer.splice(0..end, Vec::new());
            let mut value = String::new();

            loop {
                let mut is_over = false;
                while self.buffer.len() < boundary_len {
                    let size = reader.read(&mut buf).unwrap();
                    self.buffer.extend_from_slice(&buf[0..size]);
                    total += size;
                }
                for i in 0..self.buffer.len() - boundary_len {
                    // let x = Vec::from(&self.buffer[i..i + boundary_len]);
                    // println!(
                    //     "-------->{:#?} -> {}",
                    //     String::from_utf8(x).unwrap(),
                    //     &self.buffer[i..i + boundary_len].len()
                    // );
                    // println!(
                    //     "boundry->{:#?} -> {}",
                    //     String::from_utf8(boundary.into()).unwrap(),
                    //     boundary.len()
                    // );
                    // println!("{:#?}", self.buffer[i..boundary_len].eq(boundary));
                    if self.buffer[i..i + boundary_len].eq(boundary) {
                        // println!("{:#?}", &self.buffer[0..i - 2]);
                        if let Some(f) = file.as_mut() {
                            f.write_all(&self.buffer[0..i - 2]).unwrap();
                        } else {
                            let x = String::from_utf8(self.buffer[0..i - 2].to_vec()).unwrap();
                            value.push_str(&x);
                            println!("{:#?}", x);
                        }
                        self.buffer.splice(0..i + boundary_len + 2, Vec::new());
                        is_over = true;
                        break;
                    }
                }
                if is_over {
                    break;
                };
                if let Some(f) = file.as_mut() {
                    f.write_all(&self.buffer).unwrap();
                    self.buffer.clear();
                } else {
                    let x = String::from_utf8(mem::take(&mut self.buffer)).unwrap();
                    value.push_str(&x);
                }

                let size = reader.read(&mut buf).unwrap();
                self.buffer.extend_from_slice(&buf[0..size]);
                total += size;
            }
            if remind == 0 {
                if self.buffer.len() <= boundary_len {
                    break;
                }
            } else {
                if total >= remind - boundary_len && self.buffer.len() <= boundary_len {
                    break;
                }
            }
            idx_buf = 0;
        }
        Ok(res)
    }

    fn get_boundary(&self) -> Result<String> {
        // let mut boundary = String::new();
        let e = self
            .headers
            .get("Content-Type")
            .and_then(|v| Some(v.as_str()))
            .or_else(|| Some(""))
            .unwrap();
        let boundary = e.split("=").last().unwrap();

        Ok(format!("--{}", boundary))
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
    use std::{fs::File, io::Write};

    #[test]
    fn it_works() {
        let mut file = File::options()
            .append(true)
            .open("/Users/dadigua/Desktop/lifetime/app/a.txt")
            .unwrap();
        file.write(
            "easdasdj
       asjdlaisd
       asjdl"
                .as_bytes(),
        )
        .unwrap();
    }

    #[test]
    fn kmd_test() {
        fn kmp_search(text: &[u8], pattern: &[u8]) -> Option<usize> {
            let text_bytes = text;
            let pattern_bytes = pattern;
            let n = text_bytes.len();
            let m = pattern_bytes.len();

            if m == 0 {
                return Some(0); // 空模式串默认匹配
            }

            // 构建部分匹配表 (next 数组)
            let mut next = vec![0; m];
            let mut j = 0;
            for i in 1..m {
                while j > 0 && pattern_bytes[i] != pattern_bytes[j] {
                    j = next[j - 1]; // 回退
                }
                if pattern_bytes[i] == pattern_bytes[j] {
                    j += 1;
                }
                next[i] = j;
            }

            // KMP 搜索
            let mut j = 0; // 模式串指针
            for i in 0..n {
                while j > 0 && text_bytes[i] != pattern_bytes[j] {
                    j = next[j - 1]; // 回退
                }
                if text_bytes[i] == pattern_bytes[j] {
                    j += 1;
                }
                if j == m {
                    // 找到匹配，返回起始位置
                    return Some(i + 1 - m);
                }
            }

            None // 未找到匹配
        }

        let text = "ababcabcacbab";
        let pattern = "abcac";

        match kmp_search(text.as_bytes(), pattern.as_bytes()) {
            Some(index) => println!("Pattern found at index: {}", index),
            None => println!("Pattern not found"),
        }
    }
}

use std::collections::HashMap;

use crate::{request::HttpRequest, response::HttpResponse, ControllerHandler};

pub struct Controller {
    handlers: HashMap<String, HashMap<String, ControllerHandler>>
}

impl Controller {
    pub fn new() -> Self {
        let mothers = ["GET", "POST", "DELETE", "OPTIONS","PUT","PATCH","HEAD","TRACE","CONNECT"];
        let mut handlers = HashMap::with_capacity(mothers.len());
        for method in mothers {
            handlers.insert(method.to_string(), HashMap::new());
        }
        Controller {
            handlers
        }
    }
    pub fn handle_request(&self,request:HttpRequest,response:HttpResponse) {

    }
}
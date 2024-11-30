use std::collections::HashMap;

use crate::{request::HttpRequest, response::HttpResponse, ControllerHandler};

pub struct Controller {
    handlers: HashMap<String, HashMap<String, ControllerHandler>>
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            handlers: HashMap::new()
        }
    }
    pub fn handle_request(&self,request:HttpRequest,response:HttpResponse) {

    }
}
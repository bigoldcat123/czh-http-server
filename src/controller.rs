use std::collections::HashMap;

use serde_json::map::Keys;

use crate::{request::HttpRequest, response::HttpResponse, ControllerHandler};

pub struct Controller {
    static_handlers: HashMap<String, ControllerHandler>,
    handlers: HashMap<String, HashMap<String, ControllerHandler>>,
}

impl Controller {
    pub fn new() -> Self {
        let mothers = [
            "GET", "POST", "DELETE", "OPTIONS", "PUT", "PATCH", "HEAD", "TRACE", "CONNECT",
        ];
        let mut handlers = HashMap::with_capacity(mothers.len());
        for method in mothers {
            handlers.insert(method.to_string(), HashMap::new());
        }
        let static_handlers = HashMap::new();
        Controller {
            handlers,
            static_handlers,
        }
    }
    pub fn handle_not_found(&self, response: HttpResponse) {
        response.json("404 NOT FOUND!!😭");
    }
    pub fn handle_request(&self, request: HttpRequest, response: HttpResponse) {
        if let Some(handler) = self.static_handlers.get(request.prefix_url()) {
            if request.method().to_uppercase() == "GET" {
                handler(request, response);
                return;
            }
        }

        let handlers = self.handlers.get(request.method()).unwrap();
        let handler = match handlers.get(request.url()) {
            Some(handler) => handler,
            None => {
                self.handle_not_found( response);
                return;
            }
        };
        handler(request, response);
    }

    pub(crate) fn add_handler(
        &mut self,
        method: &str,
        url: &str,
        controller: impl Fn(HttpRequest, HttpResponse) + Sync + Send + 'static,
    ) {
        let handlers = self
            .handlers
            .get_mut(method.to_uppercase().as_str())
            .unwrap();
        handlers.insert(url.to_string(), Box::new(controller));
    }
    pub(crate) fn add_static_handler(
        &mut self,
        url: &str,
        controller: impl Fn(HttpRequest, HttpResponse) + Sync + Send + 'static,
    ) {
        self.static_handlers
            .insert(url.to_string(), Box::new(controller));
    }
}

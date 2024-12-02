use crate::{controller::Controller, request::HttpRequest, response::HttpResponse, HttpHander};

pub struct Route{
    controller: Option<Controller>,
}

impl Route{
    pub fn new()->Self{
        Self{
            controller: Some(Controller::new()),
        }
    }
    pub fn get_controller(mut self) -> Controller {
        self.controller.take().unwrap()
    }
}
impl HttpHander for Route{
    fn post<T>(&mut self, url: &str, controller: T)
    where
        T: Fn(HttpRequest, HttpResponse) + Sync + Send + 'static,
    {
        self.controller
            .as_mut()
            .unwrap()
            .add_handler("POST", url, Box::new(controller));
    }
    fn get<T>(&mut self, url: &str, controller: T)
    where
        T: Fn(HttpRequest, HttpResponse) + Sync + Send + 'static,
    {
        self.controller
            .as_mut()
            .unwrap()
            .add_handler("GET", url, Box::new(controller));
    }
    fn router(&mut self, url: &str, route:Route) {
        todo!()
    }
}
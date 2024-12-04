use std::collections::HashMap;

use crate::{request::HttpRequest, response::HttpResponse};

pub struct FilterChain{
    filters:HashMap<String,Vec<Filter>>
}
pub type FilterHandler = dyn Fn(HttpRequest,HttpResponse) -> Option<(HttpRequest,HttpResponse)> + Send + Sync + 'static;
// pub type FilterHandler = Box<dyn Fn(HttpRequest,HttpResponse) -> Option<(HttpRequest,HttpResponse)> + Send + Sync + 'static>;
pub struct Filter (Box<FilterHandler>);

impl Filter {
    pub fn new(f: impl Fn(HttpRequest,HttpResponse) -> Option<(HttpRequest,HttpResponse)> + Send + Sync + 'static) -> Filter {
        Filter(Box::new(f))
    }
    // (
    //     HttpRequest,
    //     HttpResponse
    pub fn do_filter(&self, request: HttpRequest, response: HttpResponse) -> Option<(HttpRequest,HttpResponse)>  {
        self.0(request, response)
    }
}

impl FilterChain {
    pub fn new() -> FilterChain {
        FilterChain { filters: HashMap::new() }
    }
    pub fn add_filter(&mut self, filter_handler:Filter ,match_url:&str) {
        if let Some(filters) = self.filters.get_mut(match_url) {
            filters.push(filter_handler);
        }else {
            self.filters.insert(match_url.to_string(), vec![filter_handler]);
        }
    }
    pub fn exec(&self,request: HttpRequest, response: HttpResponse) -> Option<(HttpRequest,HttpResponse)> {
        let mut req = request;
        let mut res = response;
        let url = req.url();
        
        Some((req, res))
    }

}
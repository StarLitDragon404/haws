//! a crate for making a customizable http web server, similar to express from node.js or bao from bun.js
//! <br>
//! example:
//! ```
//! use haws::handlers::{ AppHandler };
//! use haws::types::{ RequestBuffer };
//! fn main() {
//!     fn index(_buffer: RequestBuffer) -> String {
//!         return "<h1>Hello world</h1>".to_string();
//!     }
//!     fn err_page(_buffer: RequestBuffer) -> String {
//!         return "<h1>404 page not found</h1>".to_string();
//!    }
//!     let mut app = AppHandler::new("localhost".to_string(), 3000);
//!     // if you put a '/' in the path paramater then every time the client navigates to the root of the page or just "localhost:3000" not "localhost:3000/home" or anything like that just the root no extra path after "localhost:3000", once you navigate to this route it will return the html in the dest attribute
//!     app.route("/".to_string(), &index);
//!     //  if you put a '.' in the path paramater then every time the client navigates to a route that doesn't exist it will return the html in the dest attribute
//!     app.route(".".to_string(), &err_page);
//!     app.serve();
//! }

//! ```
pub mod handlers {
    //! The module that contains all the handler structs
    use std::net::TcpListener;
    use std::net::TcpStream;
    use std::io::prelude::*;
    use std::collections::HashMap;
    use colored;

    pub struct AppHandler<'a> {
        host: String,
        port: i32,
        routes: HashMap<String, &'a dyn Fn([u8; 1024]) -> String>,
    }
    impl<'a> AppHandler<'a> {
        pub fn new(host: String, port: i32) -> AppHandler<'a> {
            AppHandler {
                host: host,
                port: port,
                routes: HashMap::new(),
            }
        }
        pub fn route(&mut self, path: String, dest: &'a (dyn (Fn([u8; 1024]) -> String) + 'a)) {
            self.routes.insert(path, dest);
        }
        pub fn serve(&mut self) {
            let mut found_err_page = false;
            self.routes.retain(|key, _value| {
                
                if *key == ".".to_string() {
                    found_err_page = true;
                }
                true
            });
            if !found_err_page {
                println!("Error: web server has no error page");
                println!("Help: consider adding a error page");
                println!("{}", colored::Colorize::green("app.route(\".\".to_string(), &err_page_function);"));
                println!("Note: replace err_page_function with the name of your own function that returns html for the error page");
                println!("Note: remember to add a parameter with type RequestBuffer ({})", colored::Colorize::green("fn err_page(buffer: RequestBuffer) -> String"));
                println!("Note: remember to {} if you haven't already", colored::Colorize::green("use haws::types::{RequestBuffer};"));
                panic!("UnsafeWebserver")
            }
            let mut handle_connection = |mut stream: TcpStream| {
                let mut buffer = [0; 1024];
                stream.read(&mut buffer).unwrap();
                let mut gets = vec![];
                let mut paths = vec![];
                
                self.routes.retain(|key, _value| {
                    if *key != ".".to_string() {
                    gets.push(format!("GET {} HTTP/1.1\r\n", key));
                    paths.push(format!("{}", key));
                    } 
                    true
                });
                let mut idx: usize = 0;
                let mut contents: String = String::new();
                let mut found_route = false;
                for g in gets.iter() {
                    if buffer.starts_with(String::as_bytes(g)) {
                        contents = self.routes.get(paths.get(idx).unwrap()).unwrap()(buffer);
                        found_route = true;
                    }
                    idx += 1;
                }
                if !found_route {
                    
                    contents = self.routes.get(&".".to_string()).unwrap()(buffer);
                    
                }


                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                    contents.len(),
                    contents
                );
                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            };

            let listener = TcpListener::bind(format!("{}:{}", self.host, self.port)).unwrap();
            for stream in listener.incoming() {
                let ustream = stream.unwrap();

                handle_connection(ustream);
            }
        }
    }
}

pub mod types {
    //! The module that contains all the types and type aliases
    pub type RequestBuffer = [u8; 1024];

}
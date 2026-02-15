mod utils;
use utils::luafunc;
use utils::tomlparser;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get, web::{self}};
use std::fs;

use crate::utils::tomlparser::server;

async fn dynamic_routing(path: Option<web::Path<String>>, req: HttpRequest) -> impl Responder {
    if path == None {
        return luafunc::dynamic_routing_lua(req, "index");
    } else {
        return luafunc::dynamic_routing_lua(req, &path.unwrap().into_inner());
    }
    //println!("{:?}", &req.head());
    

}

/*async fn index(req: HttpRequest) -> impl Responder {
    println!("{:?}", req.headers());
    //println!("{:?}", &req.head());
    return luafunc::dynamic_routing_lua(req, "index");

}*/


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Kob-web --version: v1.3");
    println!("Added path_params and query_params both under tables of the same name, and addded server.toml functionality.\n'");

   if let Ok(routes) = fs::read_dir("logic") {
    println!("Available routes:");
    for e in routes.flatten() {
        if let Some(name) = e.file_name().to_str() {
            println!("-> : {}", name);
        }
    }
   }

    //println!("Routes detected: {:?}", entries);
    let address = server().socket.address;
    let port = server().socket.port; 
    println!("Server starting on: {}:{}", &address, &port);
    HttpServer::new(|| App::new().route("/{tail:.*}", web::to(dynamic_routing)).route("/", web::to(dynamic_routing)))
        .bind((address, port))?
        .run()
        .await
}


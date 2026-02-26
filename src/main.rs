mod utils;
use crate::utils::tomlparser::server;
use actix_files::*;
use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer, Responder, get,
    web::{self},
};
use colored::Colorize;
use std::fs;
use utils::luafunc;
use utils::tomlparser;

async fn dynamic_routing(
    path: Option<web::Path<String>>,
    req: HttpRequest,
    body: web::Bytes,
) -> impl Responder {
    if path == None {
        return luafunc::dynamic_routing_lua(req, "index", body);
    } else {
        return luafunc::dynamic_routing_lua(req, &path.unwrap().into_inner(), body);
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
    let kobwebSIGN = r"
  _  __     _        __          __  _
 | |/ /    | |       \ \        / / | |
 | ' / ___ | |__ _____\ \  /\  / /__| |__
 |  < / _ \| '_ \______\ \/  \/ / _ \ '_ \
 | . \ (_) | |_) |      \  /\  /  __/ |_) |
 |_|\_\___/|_.__/        \/  \/ \___|_.__/

 ";
    let version = "v1.4-beta1";

    println!(
        "{} {}: {} --BIG STUFF!",
        kobwebSIGN.bold().purple(),
        "VERSION".yellow().bold(),
        version.bold().blue()
    );
    println!(
        r"
    {}
    Added more request info such as:
    - Request Body
    - Request Headers
    And response control to Kob-web through:
    -- response.headers -> manipulate request headers as a lua table
    -- response.statuscode -> change request status code'

    {}
    Added max_payload_size key, renamed [socket] section to [server], with value in megabytes, for ex 25mb max payload size = 25
    ",
        "--Lua Functionality".bold().blue(),
        "--Server configuration [config/server.toml]"
            .bold()
            .yellow()
    );

    /*let mut psize = if let pl = Some(server().server.payloadsize).unwrap_or(5) {
        pl * 1024 * 1024
    } else {
        5 * 1024 * 1024
    };*/
    let psize = Some(server().server.max_payload_size * 1024 * 1024).unwrap_or(5 * 1024 * 1024);

    // Default to 5MB payload max size
    //let psize: Option<usize> = Some(server().server.payloadsize);
    /*if let payloadsize = Some(server().server.payloadsize) {
         psize = payloadsize.unwrap_or(50) * 1048576;
     }


    if let Ok(routes) = fs::read_dir("logic") {
     println!("Available routes:");
     for e in routes.flatten() {
         if let Some(name) = e.file_name().to_str() {
             println!("-> : {}", name);
         }
     }
    }*/

    //println!("Routes detected: {:?}", entries);
    let address = server().server.address;
    let port = server().server.port;
    println!("Server starting on: {}:{}", &address, &port);
    HttpServer::new(move || {
        App::new()
            .app_data(web::PayloadConfig::new(psize))
            .service(Files::new("/static", "./static"))
            .route("/{tail:.*}", web::to(dynamic_routing))
            .route("/", web::to(dynamic_routing))
    })
    .bind((address, port))?
    .run()
    .await
}

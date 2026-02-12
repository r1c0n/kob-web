use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get, web::{self}};
use serde::Deserialize;
use std::{fs};
use mlua::{Lua, prelude::LuaResult};
use std::collections::HashMap;

fn dynamic_routing_lua(req: HttpRequest, route: &str) -> HttpResponse {
    let mut requestinfo
     = HashMap::from([
      ("method", req.method().to_string()),
      ("route", req.uri().to_string()),
      ("clientip", req.peer_addr().expect("No ip").to_string()),
      //("httpversion", req.version().to_string()),
    ]);
    //println!("{:?}", requestinfo);
    // Get LuaRocks paths
    let lua_path = std::env::var("LUA_PATH").unwrap_or_default();
    let lua_cpath = std::env::var("LUA_CPATH").unwrap_or_default();
    
    let lua = unsafe { Lua::unsafe_new() };
    let mut luareqinfo = lua.create_table_from(requestinfo).expect("Couldn't fetch request info");
    lua.globals().set("request", luareqinfo);
    lua.load(&format!(
        r#"
        package.path = package.path .. ";{}"
        package.cpath = package.cpath .. ";{}"
        "#,
        lua_path, lua_cpath
    )).exec();
    let file_name = if route.is_empty() { String::from("index") } else { String::from(route) };
    if let Ok(script_content) = fs::read_to_string(format!("logic/{}.lua", &file_name)) {
        let lua_output =  match lua.load(script_content).eval::<_>() {
            Ok(content) => content,
            Err(e) =>  {
                 println!("Lua error: {}", e);
                 String::from("Error executing Lua, please check server logs.")
            }
        };
        
        return HttpResponse::Ok().body(lua_output);

    } else {
        return HttpResponse::Ok().body("Unexistent route.")
    }
    

}

#[get("/{route}")]
async fn dynamic_routing(req: HttpRequest, route: web::Path<String>) -> impl Responder {
    //println!("{:?}", &req.head());
    return dynamic_routing_lua(req, &route.into_inner());

}

#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    //println!("{:?}", &req.head());
    return dynamic_routing_lua(req, "index");

}


#[actix_web::main]
async fn main() -> std::io::Result<()> {

   if let Ok(routes) = fs::read_dir("logic") {
    println!("Available routes:");
    for e in routes.flatten() {
        if let Some(name) = e.file_name().to_str() {
            println!("-> : {}", name);
        }
    }
    println!("Wasted time on this shit and it is just for debugging ffs.\n")
   }
    //println!("Routes detected: {:?}", entries);
    let address = "127.0.0.1";
    let port = 8080; 
    println!("Server starting on: {}:{}", &address, &port);
    HttpServer::new(|| App::new().service(dynamic_routing).service(index))
        .bind((address, port))?
        .run()
        .await
}


use actix_web::dev::Response;
use actix_web::http::StatusCode;
use actix_web::http::header::{self, Header, HeaderName, HeaderValue};
use actix_web::web::Bytes;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, body, web};
use colored::Colorize;
use core::borrow;
use mlua::{Lua, ObjectLike};
use std::collections::HashMap;
use std::fs::{self, read_to_string};
use std::hash::Hash;

use crate::utils::tomlparser::server;

fn fetch_script(route: &str) -> (Option<String>, Vec<String>, String) {
    let parts: Vec<&str> = route.split('/').filter(|s| !s.is_empty()).collect();

    // Always start by checking if index.lua exists (handles both "/" and "/something")
    let index_paths = vec![
        "logic/index.lua".to_string(),
        "logic/index/index.lua".to_string(),
    ];

    let mut deepest_content: Option<String> = index_paths
        .iter()
        .find_map(|path| fs::read_to_string(path).ok());
    let mut deepest_index = -1i32; // -1 means we're at index level

    // If route is empty, we're done
    if parts.is_empty() {
        return (deepest_content, vec![], String::from("/"));
    }

    // Try to drill deeper from index
    let mut current_path = String::new();
    let mut deepest_route = String::from("/"); // Track the matched route

    for (i, part) in parts.iter().enumerate() {
        // Build path progressively: "users" -> "users/test" -> "users/test/45"
        if current_path.is_empty() {
            current_path = part.to_string();
        } else {
            current_path = format!("{}/{}", current_path, part);
        }

        // Try both file patterns at this level
        let possible_paths = vec![
            format!("logic/{}.lua", current_path),
            format!("logic/{}/index.lua", current_path),
        ];

        // Check if file exists at this level
        let found = possible_paths
            .iter()
            .find_map(|path| fs::read_to_string(path).ok());

        if let Some(content) = found {
            deepest_content = Some(content);
            deepest_index = i as i32;
            deepest_route = format!("/{}", current_path.clone());
        }
    }

    // Calculate remaining params (everything after the deepest file found)
    let remaining_params: Vec<String> = if deepest_index == -1 {
        // We only found index.lua, so all parts are params
        parts.iter().map(|s| s.to_string()).collect()
    } else {
        // Return everything after the deepest file
        parts[deepest_index as usize + 1..]
            .iter()
            .map(|s| s.to_string())
            .collect()
    };

    (deepest_content, remaining_params, deepest_route)
}

fn matches_pattern(route: &str, pattern: &str) -> bool {
    if pattern.ends_with("/*") {
        let prefix = pattern.trim_end_matches("/*");
        route.starts_with(prefix)
    } else {
        route == pattern
    }
}

pub fn dynamic_routing_lua(req: HttpRequest, route: &str, body: web::Bytes) -> HttpResponse {
    // Get system's lua paths to import already setup lua libraries
    let lua_path = std::env::var("LUA_PATH").unwrap_or_default();
    let lua_cpath = std::env::var("LUA_CPATH").unwrap_or_default();

    let lua = unsafe { Lua::unsafe_new() };
    lua.load(&format!(
        r#"
        package.path = package.path .. ";{}" .. "./logic/?.lua;"
        package.cpath = package.cpath .. ";{}"
        "#,
        lua_path, lua_cpath
    ))
    .exec();

    /*
    ▗▄▄▖ ▗▄▄▄▖▗▄▄▄▖ ▗▖ ▗▖▗▄▄▄▖ ▗▄▄▖▗▄▄▄▖
    ▐▌ ▐▌▐▌   ▐▌ ▐▌ ▐▌ ▐▌▐▌   ▐▌     █
    ▐▛▀▚▖▐▛▀▀▘▐▌ ▐▌ ▐▌ ▐▌▐▛▀▀▘ ▝▀▚▖  █
    ▐▌ ▐▌▐▙▄▄▖▐▙▄▟▙▖▝▚▄▞▘▐▙▄▄▖▗▄▄▞▘  █
     */
    // Create and Pass request information onto the LuaVM

    // DEPRECATED! REQUEST PASSING REWORKED.
    /*let requestinfo: HashMap<&str, _>
     = HashMap::from([
      ("method", req.method().to_string()),
      ("path", req.uri().to_string()),
      ("clientip", req.peer_addr().expect("No ip").to_string()),
      //("body", lua.create_string(body.as_ref()).unwrap()),
      //("headers", ),
      //("httpversion", req.version().to_string()),
    ]);
    */
    // Build headers
    // Create and Pass request information onto the LuaVM

    let request = lua.create_table().unwrap();
    request.set("method", req.method().to_string());
    request.set("path", req.uri().to_string());
    request.set("clientip", req.peer_addr().expect("No ip").to_string());
    request.set("body", lua.create_string(body.as_ref()).unwrap());

    // Fetch headers from the request
    let mut headers_map: HashMap<String, String> = HashMap::new();
    for (name, value) in req.headers().iter() {
        headers_map.insert(
            name.as_str().to_string(),
            value.to_str().unwrap_or("").to_string(),
        );
    }

    // Pass request headers to lua under request.headers
    let mut reqheaders: HashMap<&str, &str> = HashMap::new();
    for (k, v) in req.headers().iter() {
        //println!("{} = {}", k.as_str(), v.to_str().unwrap_or(""));
        reqheaders.insert(k.as_str(), v.to_str().unwrap_or(""));
    }
    request.set(
        "headers",
        lua.create_table_from(reqheaders).expect("Oopsie"),
    );

    // Finally create the request table with request info
    lua.globals().set("request", request);

    /*
    ▗▄▄▖ ▗▄▄▄▖ ▗▄▄▖▗▄▄▖  ▗▄▖ ▗▖  ▗▖ ▗▄▄▖▗▄▄▄▖    ▗▄▄▄▖▗▄▖ ▗▄▄▖ ▗▖   ▗▄▄▄▖    ▗▄▄▄▖ ▗▄▖ ▗▄▄▖     ▗▖   ▗▖ ▗▖ ▗▄▖
    ▐▌ ▐▌▐▌   ▐▌   ▐▌ ▐▌▐▌ ▐▌▐▛▚▖▐▌▐▌   ▐▌         █ ▐▌ ▐▌▐▌ ▐▌▐▌   ▐▌       ▐▌   ▐▌ ▐▌▐▌ ▐▌    ▐▌   ▐▌ ▐▌▐▌ ▐▌
    ▐▛▀▚▖▐▛▀▀▘ ▝▀▚▖▐▛▀▘ ▐▌ ▐▌▐▌ ▝▜▌ ▝▀▚▖▐▛▀▀▘      █ ▐▛▀▜▌▐▛▀▚▖▐▌   ▐▛▀▀▘    ▐▛▀▀▘▐▌ ▐▌▐▛▀▚▖    ▐▌   ▐▌ ▐▌▐▛▀▜▌
    ▐▌ ▐▌▐▙▄▄▖▗▄▄▞▘▐▌   ▝▚▄▞▘▐▌  ▐▌▗▄▄▞▘▐▙▄▄▖      █ ▐▌ ▐▌▐▙▄▞▘▐▙▄▄▖▐▙▄▄▖    ▐▌   ▝▚▄▞▘▐▌ ▐▌    ▐▙▄▄▖▝▚▄▞▘▐▌ ▐▌

    */

    // FINALLY, create the LUA response table
    let mut resheaders: HashMap<String, String> = HashMap::new();
    let response = lua.create_table().unwrap();
    response.set("statuscode", 200);
    response.set("headers", resheaders);

    lua.globals().set("response", &response);

    //let status_code = lua.create_string("hi").unwrap();
    let mut thissucka = HashMap::from([(String::from("content-type"), String::from("text/html"))]);

    /*
    ▗▄▄▄▖▗▄▄▄▖▗▄▄▄▖▗▄▄▖▗▖ ▗▖    ▗▖    ▗▄▖  ▗▄▄▖▗▄▄▄▖ ▗▄▄▖
    ▐▌   ▐▌     █ ▐▌   ▐▌ ▐▌    ▐▌   ▐▌ ▐▌▐▌     █  ▐▌
    ▐▛▀▀▘▐▛▀▀▘  █ ▐▌   ▐▛▀▜▌    ▐▌   ▐▌ ▐▌▐▌▝▜▌  █  ▐▌
    ▐▌   ▐▙▄▄▖  █ ▝▚▄▄▖▐▌ ▐▌    ▐▙▄▄▖▝▚▄▞▘▝▚▄▞▘▗▄█▄▖▝▚▄▄▖
    */

    // Fetch server's current logic
    let script_content = fetch_script(route);
    //println!("route:'{}' - scr_conten2:'{}'", route, &script_content.2);

    /*

    PATH PARAMS!!
     */
    // Fetch and pass query parameters to the Lua VM under the variable "query_params"
    let mut qphashmap: HashMap<_, _> = HashMap::new();
    let query_params = String::from(req.query_string());
    for pair in query_params.split("&").collect::<Vec<_>>() {
        let a = pair.split_once("=");
        if let Some((k, v)) = a {
            qphashmap.insert(k, v);
        }
    }
    let qpluatable = lua.create_table_from(qphashmap).unwrap();
    lua.globals().set("query_params", qpluatable);

    // Fetch and pass Path Parameters to the Lua VM under the variable "path_params"
    if server()
        .routing
        .allow_path_params
        .iter()
        .any(|x| matches_pattern(&script_content.2, x))
    {
        //println!("ALLOW PARAMS FOR: '{}'", script_content.2);
        let path_params = lua
            .create_sequence_from(script_content.1.iter().map(|x| x.as_str()))
            .unwrap();
        lua.globals().set("path_params", path_params);
    } else if !script_content.1.is_empty() {
        println!(
            "{} {} -> {}",
            req.method().to_string().bold().green(),
            req.uri().to_string().bold(),
            "404".red()
        );
        return HttpResponse::NotFound().body("Unexistent route.");
    }

    // INITIATE FORMULATEDRESPONSE VARIABLE

    //let mut formulatedresponse: actix_web::HttpResponseBuilder= HttpResponse::build(StatusCode::from_u16(response.get("statuscode").unwrap()).unwrap());
    let mut formulatedresponse = HttpResponse::build(StatusCode::NOT_FOUND);
    if let Some(content) = script_content.0 {
        let lua_output = match lua.load(content).eval::<Option<String>>() {
            Ok(Some(content)) => {
                if let Some(status) = response.get::<u16>("statuscode").ok() {
                    formulatedresponse.status(StatusCode::from_u16(status).unwrap());
                }
                content
            }
            Ok(None) => {
                &formulatedresponse.status(StatusCode::from_u16(404).unwrap());
                String::from("Unexistent route.")
            }
            Err(e) => {
                &formulatedresponse.status(StatusCode::from_u16(500).unwrap());
                println!("Lua error: {}", e);
                String::from("Error executing Lua, please check server logs.")
            }
        };

        /*
        ▄▄▄▄  ▄▄▄▄▄  ▄▄▄▄ ▄▄▄▄   ▄▄▄  ▄▄  ▄▄  ▄▄▄▄ ▄▄▄▄▄   ▄▄▄▄   ▄▄▄ ▄▄▄▄▄▄ ▄▄▄    ▄▄▄▄   ▄▄▄  ▄▄▄▄   ▄▄▄▄ ▄▄ ▄▄  ▄▄  ▄▄▄▄
        ██▄█▄ ██▄▄  ███▄▄ ██▄█▀ ██▀██ ███▄██ ███▄▄ ██▄▄    ██▀██ ██▀██  ██  ██▀██   ██▄█▀ ██▀██ ██▄█▄ ███▄▄ ██ ███▄██ ██ ▄▄
        ██ ██ ██▄▄▄ ▄▄██▀ ██    ▀███▀ ██ ▀██ ▄▄██▀ ██▄▄▄   ████▀ ██▀██  ██  ██▀██   ██    ██▀██ ██ ██ ▄▄██▀ ██ ██ ▀██ ▀███▀

         */
        //let mut formulatedresponse: actix_web::HttpResponseBuilder= HttpResponse::build(StatusCode::from_u16(response.get("statuscode").unwrap()).unwrap());

        //let mut formulatedresponse= HttpResponse::build(StatusCode::from_u16(response.get("statuscode").unwrap()).unwrap());

        //formulatedresponse.status(StatusCode::from_u16(response.get("statuscode").unwrap()).unwrap());
        //IMPLEMENT REQUEST HEADERS
        let luaresponse: mlua::Table = lua.globals().get("response").unwrap();
        let rsresheaders: mlua::Table = luaresponse.get("headers").unwrap();
        for pair in rsresheaders.pairs::<String, String>() {
            let (k, v) = pair.unwrap();
            formulatedresponse.insert_header((k.as_str(), v.as_str()));
            println!("{} = {}", k, v);
        }
        //END OF IMPLEMENTATION

        //println!("{}", luastatcode);
        let luaresponse: mlua::Table = lua.globals().get("response").unwrap();
        let rsresheaders: mlua::Table = luaresponse.get("headers").unwrap();
        for pair in rsresheaders.pairs::<String, String>() {
            let (k, v) = pair.unwrap();
            println!("{} = {}", k, v);
        }
        let resp = formulatedresponse.body(lua_output);
        println!(
            "{} {} -> {}",
            req.method().to_string().bold().green(),
            req.uri().to_string().bold(),
            if resp.status() == StatusCode::NOT_FOUND {
                resp.status().as_u16().to_string().red()
            } else {
                resp.status().as_u16().to_string().yellow()
            }
        );
        return resp;
    //return HttpResponse::Ok().body(lua_output);
    } else {
        println!(
            "{} {} -> {}",
            req.method().to_string().bold().green(),
            req.uri().to_string().bold(),
            "404".red()
        );
        return HttpResponse::NotFound().body("Unexistent route.");
    }
}

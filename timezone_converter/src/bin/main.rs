use std::thread;
use std::time::Duration;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use timezone_converter::ThreadPool;

struct ZoneMap {
        from: String,
        to: String,
        offset_hours: i32,
}

struct QueryStr<'a> {
    from: &'a str,
    to: &'a str,
}

fn copyZoneMap(vec: &Vec<ZoneMap>) -> Vec<ZoneMap>{
    let mut vec_zone_map : Vec<ZoneMap> = Vec::new();

    for item in vec{
        let obj_map1 = ZoneMap{
                from: item.from.clone(),
                to: item.to.clone(),
                offset_hours: item.offset_hours,
            };
        vec_zone_map.push(obj_map1);
    }
    vec_zone_map
}

fn main() {
    let obj_map1 = ZoneMap{
                from: String::from("China"),
                to: String::from("Usa"),
                offset_hours: -12,
            };

    let mut vec_zone_map : Vec<ZoneMap> = Vec::new();

    vec_zone_map.push(obj_map1);

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(stream) => {
                let vec_map = copyZoneMap(&vec_zone_map);

                pool.execute(move || {
                    handle_connection(vec_map.as_slice(), stream);
                });
            },
            Err(E) => println!("{}", E),
        }
        
    }
}

fn handle_connection<'a>(vec: &'a [ZoneMap], mut stream: TcpStream) {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();

    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    let mut get_result = false;
    let mut offset_hours = 0;
    let mut from = "";
    let mut to = "";
    let mut res_str = String::from("");

    let str1 = String::from_utf8_lossy(&buffer[..]);
    let query_str = get_query_str(&str1);

    match query_str {
        Some(objQueryStr) => {
            let ret_hours = get_offset_hours(vec, objQueryStr.from, objQueryStr.to);
            from = objQueryStr.from;
            to = objQueryStr.to;
            match ret_hours {
                Some(hours) => {
                    offset_hours = hours;
                    get_result = true;
                },
                None => get_result = false,
            }
        },
        None => get_result = false,
    }

    if get_result {
        res_str = format!("{} is {} hours on {}", to, offset_hours, from);
    }
    else {
        res_str = "Not found offset hours in database.".to_string();
    }
    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", res_str);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn get_offset_hours(vec: &[ZoneMap], from: &str, to: &str) -> Option<i32>{
    let mut ret :Option<i32> = None;
    for item in vec{
        if &item.from == from && &item.to == to {
            ret = Some(item.offset_hours);
            break;
        }
    }
    ret
}

fn get_query_str(querys: &str) -> Option<QueryStr>{
    let mut lines = querys.lines();
    let line = lines.next();
    match line {
        Some(query) => {
            let get = "GET /time_converter";
            if query.starts_with(get){
                let vecQuery: Vec<&str> = query.split_whitespace().collect(); 
                let mut objQueryStr = QueryStr{
                    from: "",
                    to: "",
                };
                println!("vecQuery.len:{}", vecQuery.len());
                if vecQuery.len() == 3{
                    let querystr = vecQuery[1];
                    let vec_query_strs: Vec<&str> = querystr.split('&').collect(); 

                    if vec_query_strs.len() == 2{
                        let from_pos = vec_query_strs[0].find("from");
                        match from_pos {
                            Some(n_from_pos) => {
                                objQueryStr.from = &vec_query_strs[0][n_from_pos+5..];
                            },
                            None => ()
                        }

                        let to_pos = vec_query_strs[1].find("to");
                        match to_pos {
                            Some(n_to_pos) => {
                                objQueryStr.to = &vec_query_strs[1][n_to_pos+3..];
                            },
                            None => ()
                        }
                        Some(objQueryStr)
                    }
                    else {
                        None
                    }
                    
                }  
                else {
                    None
                }
            }
            else {
                None
            }
        },
        None => None
    }
    
    
}
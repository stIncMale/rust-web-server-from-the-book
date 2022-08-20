#![deny(
    warnings,
    nonstandard_style,
    future_incompatible,
    rust_2021_compatibility,
    unused_qualifications,
    clippy::all,
    clippy::pedantic
)]
#![allow(
    // uncomment below to simplify editing, comment out again before committing
    // unused_imports,
    // unused_variables,
    // unused_mut,
    // unreachable_code,
    // dead_code,
    clippy::missing_errors_doc,
    clippy::similar_names,
    clippy::cast_possible_truncation
)]

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::{fs, thread};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}

#![allow(unused_imports)]
use std::net::TcpListener;
use std::io::{Write, Read};

fn main() {
   

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    //
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                handle_connection(&mut stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(stream: &mut std::net::TcpStream) {
    let response = String::from("+PONG\r\n");
    loop {
        let mut buffer = [0; 512];
        let n = stream.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        stream.write_all(response.as_bytes()).unwrap();
    }
   
    
   
   
}
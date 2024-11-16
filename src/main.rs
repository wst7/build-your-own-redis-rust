#![allow(unused_imports)]
use parser::{Parser, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod parser;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_connection(stream));
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream) {
    let response = String::from("+PONG\r\n");
    loop {
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await.unwrap();
        if n == 0 {
            break;
        }
        let request = String::from_utf8_lossy(&buffer[..n]);
        match Parser::parse(&request) {
            Ok(value) => match value {
                Value::SimpleString(s) => {
                    match s.as_str() {
                        "PING" => {
                            let response = String::from("+PONG\r\n");
                            stream.write_all(response.as_bytes()).await.unwrap();
                        }
                        "ECHO" => {
                            let response = String::from("+ECHO\r\n");
                            stream.write_all(response.as_bytes()).await.unwrap();
                        }
                        _ => {
                            let response = String::from("+PONG\r\n");
                            stream.write_all(response.as_bytes()).await.unwrap();
                        }
                        
                    }
                }
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        
    }
}

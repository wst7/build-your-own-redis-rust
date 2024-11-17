#![allow(unused_imports)]
use std::env;

use parser::{RespParser, RespType};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod config;
mod parser;
mod storage;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let args = env::args().collect::<Vec<String>>();
    if args.len() > 2 && args[1] == "--dir" && args[3] == "--dbfilename" {
        config::set(&args[1].strip_prefix("--").unwrap(), &args[2]).await;
        config::set(&args[3].strip_prefix("--").unwrap(), &args[4]).await;
    }
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
    loop {
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await.unwrap();
        if n == 0 {
            break;
        }
        let mut parser = RespParser::new(&buffer[..n]);
        match parser.parse() {
            Ok(resp) => match execute_command(resp).await {
                Ok(response) => {
                    println!("Response: {}", response);
                    stream.write_all(response.as_bytes()).await.unwrap();
                }
                Err(err) => {
                    stream
                        .write_all(format!("-ERR {}\r\n", err).as_bytes())
                        .await
                        .unwrap();
                }
            },

            Err(e) => {
                let response = format!("-ERR {}\r\n", e);
                stream.write_all(response.as_bytes()).await.unwrap();
            }
        }
    }
}

async fn execute_command(resp: RespType) -> Result<String, String> {
    match resp {
        RespType::Array(Some(elements)) => {
            let command = match &elements[0] {
                RespType::BulkString(Some(cmd)) => cmd.to_uppercase(),
                _ => return Err("Invalid command format".to_string()),
            };
            let args = elements[1..]
                .iter()
                .filter_map(|arg| match arg {
                    RespType::BulkString(Some(value)) => Some(value.clone()),
                    _ => None,
                })
                .collect::<Vec<String>>();
            match command.as_str() {
                "ECHO" => Ok(format!("+{}\r\n", args[0])),
                "SET" => {
                    let mut expires = None;
                    if args.len() == 4 && &args[2].to_uppercase() == "PX" {
                        expires = Some(args[3].parse::<u128>().unwrap());
                    }
                    storage::set(&args[0], &args[1], expires).await;
                    Ok(format!("+OK\r\n"))
                }
                "GET" => Ok(match storage::get(&args[0]).await {
                    Some(value) => format!("+{}\r\n", value),
                    None => format!("$-1\r\n"),
                }),
                "PING" => Ok("+PONG\r\n".to_string()),
                "CONFIG" => match args[0].to_uppercase().as_ref() {
                    "GET" => {
                        let parameter = &args[1];
                        let parameter_len = parameter.len();
                        let value = config::get(&parameter).await.unwrap();
                        let len = value.len();
                        Ok(format!(
                            "*2\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
                            parameter_len, parameter, len, value
                        ))
                    }
                    _ => Err(format!("Unknown config command: {}", args[1])),
                },
                _ => Err(format!("Unknown command: {}", command)),
            }
        }
        _ => Ok("+PONG\r\n".to_string()),
    }
}

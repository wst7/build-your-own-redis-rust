#![allow(unused_imports)]
use std::env;
use std::path::Path;

use parser::{RespParser, RespType};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod command;
mod config;
mod parser;
mod rdb;
mod storage;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let args = env::args().collect::<Vec<String>>();
    if args.len() > 2 && args[1] == "--dir" && args[3] == "--dbfilename" {
        config::set(&args[1].strip_prefix("--").unwrap(), &args[2]).await;
        config::set(&args[3].strip_prefix("--").unwrap(), &args[4]).await;
        load_data_from_rdb().await;
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
                    println!("Error: {}", err);
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
                "ECHO" => command::echo(args),
                "SET" => command::set(args).await,
                "GET" => command::get(args).await,
                "PING" => Ok("+PONG\r\n".to_string()),
                "CONFIG" => match args[0].to_uppercase().as_ref() {
                    "GET" => command::config_get(args).await,
                    _ => Err(format!("Unknown config command: {}", args[1])),
                },
                "KEYS" => command::keys(args).await,
                "SAVE" => command::save().await,
                _ => Err(format!("Unknown command: {}", command)),
            }
        }
        _ => Ok("+PONG\r\n".to_string()),
    }
}

async fn load_data_from_rdb() {
    let dir = config::get("dir").await.unwrap();
    let dbfilename = config::get("dbfilename").await.unwrap();
    let path = Path::new(&dir).join(&dbfilename);
    if path.exists() {
        let buf = tokio::fs::read(&path).await.unwrap();
        rdb::RdbParser::new(buf, |_, key, value, expire| {
            tokio::spawn(async move {
                println!("key: {}, value: {}, expire: {:?}", key, value, expire);
                storage::set(&key, &value, expire).await;
            });
        })
        .parse()
        .unwrap();
    }
}

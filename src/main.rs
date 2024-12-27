#![allow(unused_imports)]
use std::env;
use std::path::Path;

use clap::{command, Parser};
use resp::{RespParser, RespType};
use time::OffsetDateTime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod commands;
mod config;
mod rdb;
mod resp;
mod storage;

#[derive(Parser)]
#[command(name = "Rust-Redis", version = "0.1.0", author = "Your Name")]
struct Args {
    #[arg(long)]
    dir: Option<String>,

    #[arg(long)]
    dbfilename: Option<String>,

    #[arg(long)]
    port: Option<u16>,

    #[arg(long)]
    replicaof: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if args.dir.is_some() {
        config::set("dir", &args.dir.unwrap()).await;
    }
    if args.dbfilename.is_some() {
        config::set("dbfilename", &args.dbfilename.unwrap()).await;
    }
    if args.replicaof.is_some() {
        config::set("replicaof", &args.replicaof.unwrap()).await;
    }
    let port = args.port.map_or(6379, |port| port);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();
    load_data_from_rdb().await;
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
                    stream.write_all(&response.serialize()).await.unwrap();
                }
                Err(err) => {
                    stream
                        .write_all(&RespType::SimpleError(err).serialize())
                        .await
                        .unwrap();
                }
            },

            Err(e) => {
                stream
                    .write_all(&RespType::SimpleError(e).serialize())
                    .await
                    .unwrap();
            }
        }
    }
}

async fn execute_command(resp: RespType) -> Result<RespType, String> {
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
                "ECHO" => commands::echo(args),
                "SET" => commands::set(args).await,
                "GET" => commands::get(args).await,
                "PING" => Ok(RespType::SimpleString("PONG".to_string())),
                "CONFIG" => match args[0].to_uppercase().as_ref() {
                    "GET" => commands::config_get(args).await,
                    _ => Err(format!("Unknown config command: {}", args[1])),
                },
                "KEYS" => commands::keys(args).await,
                "SAVE" => commands::save().await,
                "INFO" => commands::info().await,
                _ => Err(format!("Unknown command: {}", command)),
            }
        }
        _ => Ok(RespType::SimpleString("Invalid command".to_string())),
    }
}

async fn load_data_from_rdb() {
    let dir = match config::get("dir").await {
        Some(dir) => dir,
        None => return,
    };
    let dbfilename = match config::get("dbfilename").await {
        Some(dbfilename) => dbfilename,
        None => return,
    };
    let path = Path::new(&dir).join(&dbfilename);
    if path.exists() {
        let buf = tokio::fs::read(&path).await.unwrap();
        rdb::RdbParser::new(buf, |_, key, value, expire| {
            tokio::spawn(async move {
                println!("key: {}, value: {}, expire: {:?}", key, value, expire);

                let expires = match expire {
                    Some(expires_at) => {
                        let time = OffsetDateTime::from_unix_timestamp_nanos(
                            expires_at as i128 * 1_000_000,
                        )
                        .unwrap();
                        Some(time)
                    }
                    None => None,
                };
                storage::set(&key, &value, expires).await;
            });
        })
        .parse()
        .unwrap();
    }
}

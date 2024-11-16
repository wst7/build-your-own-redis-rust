#![allow(unused_imports)]
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
        let mut buffer = [0; 512];
        let n = stream.read(&mut buffer).await.unwrap();
        if n == 0 {
            break;
        }
        stream.write_all(response.as_bytes()).await.unwrap();
    }
   
    
   
   
}
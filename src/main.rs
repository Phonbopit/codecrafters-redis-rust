use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};

use crate::resp::Value::{Error, SimpleString};

mod resp;

async fn handle_connection(stream: TcpStream) -> Result<()> {
    let mut connection = resp::RespConnection::new(stream);

    loop {
        let value = connection.read_value().await?;

        if let Some(value) = value {
            let (command, args) = value.to_command()?;
            let response = match command.to_ascii_lowercase().as_ref() {
                "ping" => SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                _ => Error(format!("command not implemented: {}", command))
            };

            connection.write_value(response).await?;
            println!("received value: {:?}", value);
        } else {
            println!("client closed connection!");
            break;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let incoming = listener.accept().await;

        match incoming {
            Ok((stream, _)) => {
                println!("accepted new connection.");

                tokio::spawn(async move {
                    handle_connection(stream).await.unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

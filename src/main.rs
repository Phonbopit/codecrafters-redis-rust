use std::sync::{Arc, Mutex};

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};

use crate::resp::Value::{BulkString, Error, Null, SimpleString};
use crate::store::Store;

mod resp;
mod store;

async fn handle_connection(stream: TcpStream, client_store: Arc<Mutex<Store>>) -> Result<()> {
    let mut connection = resp::RespConnection::new(stream);

    loop {
        let value = connection.read_value().await?;

        if let Some(value) = value {
            let (command, args) = value.to_command()?;
            let response = match command.to_ascii_lowercase().as_ref() {
                "ping" => SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                "get" => {
                    if let Some(BulkString(key)) = args.get(0) {
                        if let Some(value) = client_store.lock().unwrap().get(key.to_string()) {
                            SimpleString(value.to_string())
                        } else {
                            Null
                        }
                    } else {
                        Error("invalid arguments".to_string())
                    }
                }
                "set" => {
                    if let (Some(BulkString(key)), Some(BulkString(value))) = (args.get(0), args.get(1)) {
                        if let (Some(BulkString(_)), Some(BulkString(amount))) = (args.get(2), args.get(3)) {
                            client_store.lock().unwrap().set_with_expiry(key.to_string(), value.to_string(), amount.parse::<u64>()?);
                        } else {
                            client_store.lock().unwrap().set(key.to_string(), value.to_string());
                        }
                        SimpleString("OK".to_string())
                    } else {
                        Error("invalid arguments".to_string())
                    }
                }
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

    let main_store = Arc::new(Mutex::new(Store::new()));

    loop {
        let incoming = listener.accept().await;
        let client_store = main_store.clone();

        match incoming {
            Ok((stream, _)) => {
                println!("accepted new connection.");

                tokio::spawn(async move {
                    handle_connection(stream, client_store).await.unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

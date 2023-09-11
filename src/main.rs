use anyhow::Result;
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buff = BytesMut::with_capacity(512);

    loop {
        // wait for client to send us a message
        let bytes_read = stream.read_buf(&mut buff).await?;
        if bytes_read == 0 {
            println!("client closed connection!");
            break;
        }

        stream.write("+PONG\r\n".as_bytes()).await?;
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

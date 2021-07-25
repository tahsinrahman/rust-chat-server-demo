use std::net::SocketAddr;
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    let (tx, _) = broadcast::channel(32);

    loop {
        let (stream, client) = listener.accept().await?;

        let tx = tx.clone();
        let rx = tx.subscribe();

        tokio::spawn(handle_connection(stream, client, tx, rx));
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    client: SocketAddr,
    broadcast_tx: Sender<(String, SocketAddr)>,
    mut broadcast_rx: Receiver<(String, SocketAddr)>,
) -> io::Result<()> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);

    loop {
        let mut line = String::new();
        tokio::select! {
            result = reader.read_line(&mut line) => {
                let read_bytes = result.unwrap();
                if read_bytes == 0 {
                    break;
                }
                broadcast_tx.send((line, client)).unwrap();
            },
            result = broadcast_rx.recv() => {
                let (msg, other_client) = result.unwrap();
                if client != other_client {
                    writer.write_all(msg.as_bytes()).await.unwrap();
                }
            },
        }
    }

    Ok(())
}

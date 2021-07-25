use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    let (tx, _) = broadcast::channel(32);

    loop {
        let (mut stream, client) = listener.accept().await?;

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
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
                        tx.send((line, client)).unwrap();
                    },
                    result = rx.recv() => {
                        let (msg, other_client) = result.unwrap();
                        if client != other_client {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    },
                }
            }
        });
    }
}

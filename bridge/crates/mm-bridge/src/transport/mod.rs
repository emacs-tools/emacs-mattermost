use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::app;

pub async fn handle_client(stream: TcpStream) -> std::io::Result<()> {
    let peer = stream.peer_addr().ok();
    tracing::info!(?peer, "client connected");

    let (read_half, mut write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        let response = app::response_line(&line);
        write_half.write_all(response.as_bytes()).await?;
        write_half.flush().await?;
    }

    tracing::info!(?peer, "client disconnected");
    Ok(())
}

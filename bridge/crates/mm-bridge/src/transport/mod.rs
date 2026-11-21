use crate::app::App;
use crate::protocol;
use anyhow::Context;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub async fn handle_client(app: App, stream: TcpStream) -> anyhow::Result<()> {
    let peer = stream.peer_addr().ok();
    tracing::info!(?peer, "client connected");

    let (read_half, write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    let mut lines = reader.lines();

    // Обёртка для безопасного совместного использования writer между двумя задачами
    let writer = Arc::new(Mutex::new(write_half));

    // Подписываемся на события моста (WebSocket fan-out)
    let mut event_rx = app.state.subscribe_events();

    // Запускаем фоновую задачу для отправки асинхронных событий в этот TCP-сокет
    let writer_for_events = Arc::clone(&writer);
    let writer_handle = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            if let Ok(line) = protocol::encode_event(&event) {
                let mut w = writer_for_events.lock().await;
                if w.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
                if w.flush().await.is_err() {
                    break;
                }
            }
        }
    });

    // Основной цикл чтения запросов от Emacs
    let mut result = Ok(());
    while let Some(line) = lines.next_line().await.context("read line failed")? {
        if line.trim().is_empty() {
            continue;
        }

        let response = app.response_line(&line).await;
        {
            let mut w = writer.lock().await;
            if w.write_all(response.as_bytes()).await.is_err() {
                result = Err(anyhow::anyhow!("client disconnected during write"));
                break;
            }
            if w.flush().await.is_err() {
                result = Err(anyhow::anyhow!("client disconnected during flush"));
                break;
            }
        }
    }

    // Отменяем задачу записи событий при отключении клиента
    writer_handle.abort();

    tracing::info!(?peer, "client disconnected");
    result
}

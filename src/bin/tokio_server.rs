use std::{thread, time};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::{Date, DateTime, NaiveDate, NaiveTime, Utc};
use futures::lock::Mutex;
use futures::TryStream;
use futures_timer::Delay;
use futures_util::{
    future, pin_mut, SinkExt,
    stream::TryStreamExt,
    StreamExt,
};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tungstenite::{Message, Result};
use tungstenite::server::accept;

type Tx = UnboundedSender<i32>;

type Senders = Arc<Mutex<Vec<Tx>>>;

async fn value(senders: Senders) -> Result<()> {
    println!("[]thread B |  PRODUCE VALUES");

    for i in 0..100 {
        println!("------------------------------------------");
        println!("thread B | PRODUCING: {}", i);

        {
            let peers = senders.lock().await;

            for recp in peers.iter() {
                recp.send(i);
            }
        }

        let now = Delay::new(Duration::from_millis(1000)).await;
    }

    println!("[] TERMINATED");
    Ok(())
}

async fn web_printer(addr: SocketAddr, stream: TcpStream, senders: Senders) -> Result<()> {
    println!("New WebSocket connection: {}", addr);
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("thread C | WebSocket connection established: {}", addr);

    let (tx, mut rx) = unbounded_channel();
    senders.lock().await.push(tx);

    println!("thread C | ws [{}] | WAITING", addr);


    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(v) = rx.recv().await {
        println!("thread C | ws [{}] | RECEIVED: {:#?}", addr, v);
        println!("thread C | ws [{}] | SENDING TO WS: {:#?}", addr, v);

        let mess = Message::Text(v.to_string());
        ws_sender.send(mess).await?;
    }

    println!("PRINTER ID [{}] | DONE", addr);
    Ok(())
}


#[tokio::main(threaded_scheduler)]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:8080";
    let mut listener = TcpListener::bind(&addr).await.expect("Can't listen");
    println!("thread A | Listening on: {}", addr);

    let senders: Senders = Arc::new(Mutex::new(vec![]));

    tokio::spawn(value(Arc::clone(&senders)));

    while let Ok((socket, _)) = listener.accept().await {
        let addr = socket
            .peer_addr()
            .expect("connected streams should have a peer address");
        println!("thread A | connecting ws | address: {}", addr);

        tokio::spawn(
            // Process each socket concurrently.
            web_printer(addr, socket, Arc::clone(&senders))
        );
    }

    Ok(())
}
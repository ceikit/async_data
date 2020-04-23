use std::{thread, time};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use chrono::{Date, DateTime, NaiveDate, NaiveTime, Utc};
use futures::TryStream;
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_util::{
    future, pin_mut, SinkExt,
    stream::TryStreamExt,
    StreamExt,
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tungstenite::{Message, Result};
use tungstenite::server::accept;

type Tx = UnboundedSender<i32>;

type Senders = Arc<Mutex<Vec<Tx>>>;


async fn value(senders: Senders) -> Result<()> {
    println!("[] PRODUCE VALUES");

    for i in 0..10 {
        println!("------------------------------------------");
        println!("PRODUCING: {}", i);

        let peers = senders.lock().unwrap();

        for recp in peers.iter() {
            recp.unbounded_send(i).unwrap();
        }


        let ten_millis = time::Duration::from_millis(1000);
        thread::sleep(ten_millis);
    }

    println!("[] TERMINATED");
    future::ready(Ok(())).await
}

async fn web_printer(addr: SocketAddr, stream: TcpStream, senders: Senders) -> Result<()> {
    println!("New WebSocket connection: {}", addr);
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);
    println!("PRINTER ID [{}] | WAITING", addr);

    let (tx, mut rx) = unbounded();
    senders.lock().unwrap().push(tx);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();



    loop {
        while let Some(v) = rx.next().await {
            println!("PRINTER ID [{}] | RECEIVED: {:#?}", addr, v);
            println!("PRINTER ID [{}] | SENDING TO WS: {:#?}", addr, v);

            let  mess = Message::Text(v.to_string());
            ws_sender.send(mess).await?;
        }
    }

    println!("PRINTER ID [{}] | DONE", addr);



    println!("PRINTER ID [{}] | DONE", addr);

    Ok(())
}

#[tokio::main(threaded_scheduler)]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:8080";
    let mut listener = TcpListener::bind(&addr).await.expect("Can't listen");
    println!("Listening on: {}", addr);

    let senders: Senders = Arc::new(Mutex::new(vec![]));

    let value_job = tokio::spawn(value(Arc::clone(&senders)));

    while let Ok((socket, _)) = listener.accept().await {
        let addr = socket
            .peer_addr()
            .expect("connected streams should have a peer address");
        println!("Peer address: {}", addr);

        tokio::spawn(
            // Process each socket concurrently.
            web_printer(addr, socket, Arc::clone(&senders))
        );
    }

    Ok(())
}
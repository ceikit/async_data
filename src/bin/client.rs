use futures_util::{
    future, pin_mut,
    stream::TryStreamExt,
    StreamExt,
};
use tokio_tungstenite::connect_async;
use tungstenite::{connect, Message};
use url::Url;

#[tokio::main]
async fn main() {
    let (mut socket, response) =
        connect(Url::parse("ws://127.0.0.1:8080/").unwrap()).expect("Can't connect");

    println!("WebSocket handshake has been successfully completed");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    // let (write, read) = socket.split();


    socket
        .write_message(Message::Text("Hello WebSocket".into()))
        .unwrap();


    // let casa = read.for_each(|f| {
    //     let msg = f.expect("Error reading message");
    //     println!("Received: {}", msg);
    //     future::ready(())
    // });
    //
    // casa.await;

    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
    }

    socket.close(None);
}
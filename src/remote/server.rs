use bevy::prelude::*;

use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{
        Message,
        Result
    },
};

use crate::{
    CliArgs
};


fn setup(args: Res<CliArgs>) {
    log::info!("in setup()");

    let Ok(rt) = tokio::runtime::Runtime::new()
        .inspect_err(|err| log::error!("unable to  initialize tokio runtime: {err}"))
        else {
            return;
        };


    rt.spawn(run(args.connect_to));

    log::info!("all done");
}

async fn run(addr: SocketAddr) {
    log::info!("launching server to listen for incoming websocket requests");

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("failed to bind");
    log::info!("Listening at {addr}, listener = {listener:?}");

    
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }

    log::info!("Listening");
}

#[derive(Debug)]
struct Connection {
    peer: SocketAddr,
    stream: TcpStream,
}

async fn accept_connection(stream: TcpStream) -> Result<()> {
    use std::time::Duration;

    let addr = stream
        .peer_addr()
        .expect("connected streams have peer address");

    log::info!("peer address: {addr}");

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("websocket handshake");

    let duration = Duration::from_secs(5);
    let mut interval = tokio::time::interval(duration);

    log::info!("new websocket connection: {addr}");

    let (mut ws_write, mut ws_read) = ws_stream.split();

    loop {
        tokio::select! {
            msg = ws_read.next() => {
                let Some(msg) = msg else {
                    log::warn!("breaking");
                    break;
                };
                let msg = msg?;

                log::info!("received: {msg:?}");

                if msg.is_text() || msg.is_binary() {
                    // just echo it
                    ws_write.send(msg).await?;
                } else if msg.is_close() {
                    log::warn!("msg.is_close(), breaking...");
                    break;
                }

            }

            _ = interval.tick() => {
                ws_write.send(Message::Text("tick".to_owned())).await?;
            }

        }
    }

    Ok(())
}

async fn handle_connection(ctn: Connection) -> Result<()> {
    let ws_stream = accept_async(ctn.stream).await?;

    log::info!("new websocket connection: {}", ctn.peer);

    let (mut ws_write, mut ws_read) = ws_stream.split();


    // echo every connection
    loop {
        tokio::select! {

            msg = ws_read.next() => {
                // bubble up any error
                let msg = msg.transpose()?;
                // break the loop if the stream is finished
                let Some(msg) = msg else { 
                    log::info!("stream finished");
                    break;
                };

                log::info!("recvd: {msg:?}");

                // echo it back
                if msg.is_text() || msg.is_binary() {
                    ws_write.send(msg).await?;
                    continue;
                }

                if msg.is_close() {
                    log::info!("closing out");
                    break;
                }


            }
        }
    }

    Ok(())
}


pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
        ;
    }
}

use bevy::prelude::*;

use futures_util::{SinkExt, StreamExt};
use std::{net::SocketAddr, time::Duration};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, Result},
};

use crate::{
    CliArgs
};


fn setup(args: Res<CliArgs>) {

    let Ok(rt) = tokio::runtime::Runtime::new()
        .inspect_err(|err| log::error!("unable to  initialize tokio runtime: {err}"))
        else {
            return;
        };

    rt.spawn(run(args.connect_to));
}

async fn run(addr: SocketAddr) {
    log::info!("launching server to listen for incoming websocket requests");

    let listener = TcpListener::bind(&addr).await.expect("can't listen");
    info!("Listening at {addr}");

    while let Ok((stream, _)) = listener.accept().await {
        let Ok(peer) = stream.peer_addr()
            .inspect(|err| log::error!("unable to get peer address: {err}"))
            else {
                log::warn!("connected streams should have a peer address, dropping this connection...");
                continue;
            };

        info!("Connection at {peer}");
        let ctn = Connection {
            peer,
            stream
        };
        tokio::spawn(accept_connection(ctn));
    }
}

#[derive(Debug)]
struct Connection {
    peer: SocketAddr,
    stream: TcpStream,
}

async fn accept_connection(ctn: Connection) {
    use Error::*;
    let result = handle_connection(ctn).await;
    match result {
        Ok(_) => { return },
        Err(ConnectionClosed) => {
            log::info!("connection closed by client");
        }
        Err(err) => {
            log::error!("Error handling connection: {err}");
        }
    }
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
                let Some(msg) = msg else { break; };

                log::info!("recvd: {msg:?}");

                // echo it back
                if msg.is_text() || msg.is_binary() {
                    ws_write.send(msg).await?;
                    continue;
                }

                if msg.is_close() {
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

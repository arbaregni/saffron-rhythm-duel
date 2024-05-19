use bevy::prelude::*;

use std::net::SocketAddr;

use futures_util::{
    SinkExt,
    StreamExt
};
use tokio::{
    net::{
        TcpListener,
    },
    sync::mpsc,
};
use tokio_tungstenite::{
    tungstenite::{
        Message as WsMessage,
    },
};
use serde::{
    Deserialize,
    Serialize
};

use crate::{
    CliArgs
};
use crate::lane::{
    Lane
};

/// Message sent from user to user to communicate game state.
/// We will use this for local -> remote and remote -> local
/// since comms are meant to be symmetric
#[derive(Debug, Clone)]
#[derive(Deserialize, Serialize)]
pub enum GameMessage {
    LaneHit {
        lane: Lane
    }
}

#[derive(Resource)]
#[derive(Debug)]
pub struct Listener {
    receive_msg: Option<mpsc::Receiver<GameMessage>>,
    _runtime: tokio::runtime::Runtime,
}
impl Listener {
    pub fn init(_cli: &CliArgs) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .build()
            .expect("failed to initialize tokio runtime");

        let listen_at = "127.0.0.1:0"; // the OS will assign us a port to use

        // incoming remote messages -> local game events
        let (send_msg, receive_msg) = mpsc::channel(1024);

        // local game events -> outgoing remote messages

        // start up the listening thread
        rt.spawn(listen_for_incoming(send_msg, listen_at));

        Self {
            receive_msg: Some(receive_msg),
            // we need to keep the runtime around, other wise our tasks will be dropped
            _runtime: rt,
        }
    }

    /// Return the remote message, if there is one
    pub fn message(&mut self) -> Option<GameMessage> {
        use mpsc::error::TryRecvError::*;
        match self.receive_msg.as_mut()?.try_recv() {
            // If we successfully receive a message, return that
            Ok(msg) => Some(msg),
            // If there's nothing at the moment, return none
            Err(Empty) => None,
            // If we've disconnected then we log it as an error
            // TODO: gracefully close the receiving end
            Err(Disconnected) => {
                log::error!("remote message channel disconnected, no more data can be received");
                self.receive_msg = None;
                None
            }
        }
    }
}

async fn listen_for_incoming(send_msg: mpsc::Sender<GameMessage>, listen_at: &str) {
    log::info!("attempting to bind to {listen_at}");
    let listener = TcpListener::bind(&listen_at).await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    log::info!("succesfully bound at {listener:?}, port = {local_addr}");

    log::info!("waiting for a connection");
    let (stream, _) = listener.accept().await.unwrap();
    log::info!("accepted connection: {stream:?}");

    log::info!("attempting to upgrade connection");
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("websocket handshake");
    log::info!("new websocket connection");

    let (mut _ws_write, mut ws_read) = ws_stream.split();

    loop {

        // send whatever we get from the stream to the channel

        tokio::select! {

            // read an incoming message from the client
            incoming = ws_read.next() => {
                let incoming = match incoming {
                    Some(incoming) => incoming,
                    None => {
                        log::info!("closed connection");
                        break;
                    }
                };
                let Ok(incoming) = incoming
                    .inspect_err(|e| log::warn!("error reading incoming message {e}, closing connection"))
                    else { break; };

                log::info!("received: {incoming:?}");
                if incoming.is_close() {
                    log::info!("client closed connection");
                    break;
                }
                let Ok(text) = incoming.to_text()
                    // drop the message if we can't convert it to text
                    .inspect_err(|e| log::warn!("unable to convert incoming message to text: {e}"))
                    else { continue; };


                // parse the incoming message
                let Ok(incoming) = serde_json::from_str(text)
                    .inspect_err(|e| log::warn!("bad request: {e}"))
                    else { continue; };

                let Ok(_) = send_msg.send(incoming).await
                    .inspect_err(|e| {
                        log::error!("unable to send to remote message channel: {e}")
                    })
                    else { break; };

            }


            // TODO: check channel for outgoing messages
            // outgoing = 

        } // end tokio::select!


    } // end loop

    // TODO: we should go back into a "listening state" here,
    // or somehow have the listening go into a "not accepting anymore players" mode
}



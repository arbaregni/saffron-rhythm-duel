use bevy::prelude::*;

use std::net::{
    SocketAddr,
    IpAddr,
};

use futures_util::{
    SinkExt,
    StreamExt,
    stream::Stream,
    sink::{Sink},
};
use tokio::{
    net::{
        TcpListener,
    },
    sync::mpsc,
};
use tungstenite::protocol::{
    Message as WsMessage
};
use serde::{
    Deserialize,
    Serialize
};
use url::Url;

use crate::{
    ConnectionMode,
    CliArgs,
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
    /// Channel that receives the remote user's GameMessages
    receive_msg: Option<mpsc::Receiver<GameMessage>>,
    /// Channel that sends the local user's GameMessages
    send_msg: Option<mpsc::Sender<GameMessage>>,
    /// Keep the tokio runtime around that is computing our background tasks.
    _runtime: tokio::runtime::Runtime,
}
impl Listener {
    pub fn init(cli: &CliArgs) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .build()
            .expect("failed to initialize tokio runtime");


        // incoming remote messages -> local game events
        let (incoming_tx, incoming_rx) = mpsc::channel(1024);

        // local game events -> outgoing messages
        let (outgoing_tx, outgoing_rx) = mpsc::channel(1024);
        
        match &cli.mode {
            Some(ConnectionMode::Serve { port }) => {
                // if supplying zero, the OS will give us a port to use
                let port = port.unwrap_or(0);

                let ip = IpAddr::from([127u8, 0, 0, 1]);
                let listen_at = SocketAddr::new(ip, port);

                // local game events -> outgoing remote messages

                // start up the listening thread
                rt.spawn(listen_for_incoming(incoming_tx, outgoing_rx, listen_at));
            }
            Some(ConnectionMode::Connect { remote_url }) => {
                rt.spawn(connect_to_remote(incoming_tx, outgoing_rx, remote_url.clone()));
            }
            None => {
                // bleh, just do nothing.
                // this drops the connection so we'll see some errors logged,
                // but everything should be fine.
            }
        }

        Self {
            receive_msg: Some(incoming_rx),
            send_msg: Some(outgoing_tx),
            // we need to keep the runtime around, other wise our tasks will be dropped
            _runtime: rt,
        }
    }

    /// Return the remote message, if there is one
    pub fn try_recv_message(&mut self) -> Option<GameMessage> {
        use mpsc::error::TryRecvError::*;
        match self.receive_msg.as_mut()?.try_recv() {
            // If we successfully receive a message, return that
            Ok(msg) => Some(msg),
            // If there's nothing at the moment, return none
            Err(Empty) => None,
            // If we've disconnected then we log it as an error
            // TODO: gracefully close the receiving end
            Err(Disconnected) => {
                log::error!("remote message channel (incoming_rx) disconnected, no more data can be received from remote user");
                self.receive_msg = None;
                None
            }
        }
    }

    /// Send a game message
    pub fn try_send_message(&mut self, message: GameMessage) {
        let Some(send_msg) = self.send_msg.as_mut()
            // if the channel's dropped, just return silently.
            else { 
                bevy::log::warn_once!("no channel (outgoing_tx) configured");
                return;
            };
        
        match send_msg.blocking_send(message) {
            Ok(_) => { },
            Err(_) => {
                log::warn!("could not process: dropping outgoing message");
            }
        };
    }
}

async fn listen_for_incoming(mut incoming_tx: mpsc::Sender<GameMessage>, mut outgoing_rx: mpsc::Receiver<GameMessage>, listen_at: SocketAddr) {
    let Ok(listener) = TcpListener::bind(listen_at).await
        .inspect_err(|e| log::error!("failed to bind to {listen_at}: {e}"))
        else { return; };

    let local_addr = listener
        .local_addr()
        .map(|s| s.to_string())
        .unwrap_or("<not found>".to_owned());
    log::info!("succesfully bound to {local_addr}");

    loop {
        log::info!("waiting for a connection");
        let Ok((stream, _)) = listener
            .accept()
            .await
            .inspect_err(|e| log::error!("failed to connect: {e}"))
            else { continue; };

        log::info!("accepted connection, attempting to upgrade to websocket");

        let Ok(ws_stream) = tokio_tungstenite::accept_async(stream)
            .await
            .inspect_err(|e| log::error!("failed to upgrade websocket: {e}"))
            else { continue; };

        log::info!("new websocket connection");

        handle_connection(ws_stream, &mut incoming_tx, &mut outgoing_rx).await;

        log::info!("client lost, back to listening");
    }
}

/// Connects to the remote and listens for updates to game state
async fn connect_to_remote(mut incoming_tx: mpsc::Sender<GameMessage>, mut outgoing_rx: mpsc::Receiver<GameMessage>, remote: Url) {
    loop {
        log::info!("attempting to connect to remote");
        let Ok((ws_stream, _)) = tokio_tungstenite::connect_async(remote.clone())
            .await
            .inspect_err(|e| log::error!("failed to connect to remote: {e}"))
            else {
                // TODO: maybe a more graceful way of retrying this
                return;
            };

        log::info!("new websocket connection");

        handle_connection(ws_stream, &mut incoming_tx, &mut outgoing_rx).await;
    }

}

/// Runs until the connection to remote is lost
async fn handle_connection<S>(ws_stream: S, incoming_tx: &mut mpsc::Sender<GameMessage>, outgoing_rx: &mut mpsc::Receiver<GameMessage>)
where S: Stream<Item = Result<WsMessage,tungstenite::error::Error>> + Sink<WsMessage>
{
    let (mut ws_write, mut ws_read) = ws_stream.split();

    log::info!("handling connection");
    loop {
        tokio::select! {

            // read an incoming message from the client
            incoming = ws_read.next() => {
                let incoming = match incoming {
                    Some(incoming) => incoming,
                    None => {
                        log::info!("closed incoming connection");
                        break;
                    }
                };
                let Ok(incoming) = incoming
                    .inspect_err(|e| log::warn!("error reading incoming message {e}, closing connection"))
                    else { break; };

                log::debug!("received: {incoming:?}");
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

                let Ok(_) = incoming_tx.send(incoming).await
                    .inspect_err(|e| {
                        log::error!("unable to send to remote message channel: {e}")
                    })
                    else { break; };

            }


            // the game system may send us messages to rely to the remote user.
            // when this happens, write them to the websocket
            outgoing = outgoing_rx.recv() => {
                log::debug!("received outgoing message: {outgoing:?}");
                let outgoing = match outgoing {
                    Some(outgoing) => outgoing,
                    None => {
                        log::info!("closing outgoing connection");
                        break;
                    }
                };

                let Ok(outgoing_json) = serde_json::to_string(&outgoing)
                    .inspect_err(|e| log::error!("serialization failed: {e}"))
                    else { continue; };

                let outgoing_ws_msg = WsMessage::text(outgoing_json);

                log::debug!("sending: {outgoing_ws_msg:?}");
                let Ok(_) = ws_write.send(outgoing_ws_msg)
                    .await
                    .inspect_err(|_| log::error!("unable to send on websocket"))
                    else { continue; };
            }

        } // end tokio::select!
    } // end loop

}



use bevy::prelude::*;

use std::net::{
    SocketAddr,
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

use url::Url;

use crate::{
    ConnectionMode,
    CliArgs,
    settings::UserSettings,
};

use super::{
    GameMessage,
    widgets::NetStatus,
};

#[derive(Resource)]
#[derive(Debug)]
/// Struct that holds the communications to remote player
pub struct Comms {
    /// Channel that receives the remote user's GameMessages
    receive_msg: Option<mpsc::Receiver<GameMessage>>,
    /// Channel that sends the local user's GameMessages
    send_msg: Option<mpsc::Sender<GameMessage>>,
    /// Channel that receives status updates from the background tasks
    pub (in crate::remote) status_rx: Option<mpsc::Receiver<NetStatus>>,

    /// Keep the tokio runtime around that is computing our background tasks.
    _runtime: tokio::runtime::Runtime,
}
impl Comms {
    pub fn init(cli: &CliArgs, settings: &UserSettings) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .build()
            .expect("failed to initialize tokio runtime");


        // incoming remote messages -> local game events
        let (incoming_tx, incoming_rx) = mpsc::channel(1024);

        // local game events -> outgoing messages
        let (outgoing_tx, outgoing_rx) = mpsc::channel(1024);


        // the communicator changes states -> displayed as in-game diagnostics
        let (status_tx, status_rx) = mpsc::channel(4);
        
        let ctn = ConnectionContext {
            incoming_tx,
            outgoing_rx,
            status_tx,
        };

        match &cli.mode {
            ConnectionMode::Listen { port } => {
                // specify on the command line, or fall back to the settingsured settings
                let port = port.unwrap_or(settings.port);

                let ip = settings.host_addr;
                let listen_at = SocketAddr::new(ip, port);

                // local game events -> outgoing remote messages

                // start up the listening thread
                let task = ctn.listen_for_incoming(listen_at);
                rt.spawn(task);
            }
            ConnectionMode::Connect { remote_url } => {
                let task = ctn.connect_to_remote(remote_url.clone());
                rt.spawn(task);
            }
        }

        Self {
            receive_msg: Some(incoming_rx),
            send_msg: Some(outgoing_tx),
            status_rx: Some(status_rx),
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
                bevy::log::warn_once!("no channel (outgoing_tx) settingsured");
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

type WsMessage = tungstenite::protocol::Message;
type WsMessageResult = Result<WsMessage, tungstenite::error::Error>;

/// Acts as the glue between the game objects and the remote player.
/// This gets sent over to a background thread 
/// and only communicates with the game through the `Comms` struct
struct ConnectionContext {
    incoming_tx: mpsc::Sender<GameMessage>,
    outgoing_rx: mpsc::Receiver<GameMessage>,
    status_tx: mpsc::Sender<NetStatus>,
}
impl ConnectionContext {
    async fn update_status(&mut self, msg: NetStatus) {
        match self.status_tx.send(msg).await {
            Ok(_) => {},
            Err(e) => {
                log::error!("error updating status at status_tx: {e}");
            }
        }
    }
    async fn listen_for_incoming(mut self, listen_at: SocketAddr) {
        self.update_status(NetStatus::Listening(format!(
            "attempting to listen at {listen_at}"
        ))).await;

        let Ok(listener) = TcpListener::bind(listen_at).await
            .inspect_err(|e| log::error!("failed to bind to {listen_at}: {e}"))
            else { return; };

        let local_addr = listener
            .local_addr()
            .map(|s| s.to_string())
            .unwrap_or("<not found>".to_owned());
        log::info!("succesfully bound to {local_addr}");

        loop {
            log::info!("waiting for a connection on {local_addr}");
            self.update_status(NetStatus::Listening(format!(
                "waiting for a connection on {local_addr}"
            ))).await;

            let stream = match listener.accept().await {
                Ok((stream, _)) => stream,
                Err(e) => {
                    log::error!("failed to connect: {e}");
                    self.update_status(NetStatus::Error(format!(
                        "failed to accept connection: {e}"
                    ))).await;
                    continue;
                }
            };

            log::info!("accepted connection, attempting to upgrade to websocket");

            let ws_stream = match tokio_tungstenite::accept_async(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    log::error!("failed to upgrade websocket: {e}");
                    self.update_status(NetStatus::Error(format!(
                        "failed to upgrade connection: {e}"
                    ))).await;
                    continue;
                }
            };

            log::info!("new websocket connection");

            self.handle_connection(ws_stream).await;

            log::info!("client lost, back to listening");
        }
    }

    /// Connects to the remote and listens for updates to game state
    async fn connect_to_remote(mut self, remote: Url) {
        loop {
            log::info!("attempting to connect to remote");
            self.update_status(NetStatus::Connecting(format!(
                "attempting to connect to {remote}"
            ))).await;

            let ws_stream = match tokio_tungstenite::connect_async(remote.clone()).await {
                Ok((ws, _)) => ws,
                Err(e) => {
                    log::error!("failed to connect to remote: {e}");
                    self.update_status(NetStatus::Error(format!(
                        "failed to connect to remote: {e}"
                    ))).await;
                    // TODO: maybe a more graceful way of retrying this
                    return;
                }
            };

            log::info!("new websocket connection");

            self.handle_connection(ws_stream).await;
        }

    }


    /// Runs until the connection to remote is lost
    async fn handle_connection<S>(&mut self, ws_stream: S)
        where S: Stream<Item = WsMessageResult> + Sink<WsMessage>
    {
        let (mut ws_write, mut ws_read) = ws_stream.split();

        log::info!("handling connection");
        self.update_status(NetStatus::Connected).await;

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

                        let Ok(_) = self.incoming_tx.send(incoming).await
                            .inspect_err(|e| {
                                log::error!("unable to send to remote message channel: {e}")
                            })
                            else { break; };

                    }


                    // the game system may send us messages to rely to the remote user.
                    // when this happens, write them to the websocket
                    outgoing = self.outgoing_rx.recv() => {
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

            self.update_status(NetStatus::Disconnected).await;

        }


}


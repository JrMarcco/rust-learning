use anyhow::Result;
use dashmap::DashMap;
use futures::{stream::SplitStream, SinkExt, StreamExt};
use std::{
    fmt::{self, Display},
    net::SocketAddr,
    sync::Arc,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

const MAX_MESSAGE: usize = 128;

#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

#[derive(Debug)]
struct Peer {
    username: String,
    stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}

#[derive(Debug)]
enum Message {
    UserJoined(String),
    UserLeft(String),
    Chat { sender: String, content: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    let console_layer = console_subscriber::spawn();

    tracing_subscriber::registry()
        .with(layer)
        .with(console_layer)
        .init();

    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await?;
    info!("Chat server listening on {}", addr);

    let state = Arc::new(State::default());

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Client {} connected", addr);

        // handle client in a new task
        let state_cloned = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = handle_client(state_cloned, addr, stream).await {
                warn!("Failed to handle client: {}, {}", addr, e);
            }
        });
    }
}

async fn handle_client(state: Arc<State>, addr: SocketAddr, stream: TcpStream) -> Result<()> {
    // split stream into lines codec
    let mut stream = Framed::new(stream, LinesCodec::new());
    stream.send("Enter your username: ").await?;

    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };

    // if username is empty, disconnect
    if username.is_empty() {
        return Ok(());
    }

    let mut peer = state.add(addr, username, stream).await;

    let msg = Arc::new(Message::UserJoined(peer.username.clone()));
    info!("{}", msg);
    state.broadcast(addr, msg).await;

    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("Failed to read line from {}: {}", addr, e);
                break;
            }
        };

        let msg = Arc::new(Message::Chat {
            sender: peer.username.clone(),
            content: line,
        });
        info!("{}", msg);
        state.broadcast(addr, msg).await;
    }

    // when loop ends, peer has left the chat or line reading failed
    state.peers.remove(&addr);
    let msg = Arc::new(Message::UserLeft(peer.username));
    state.broadcast(addr, msg).await;

    Ok(())
}

impl State {
    async fn broadcast(&self, addr: SocketAddr, msg: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() == &addr {
                continue;
            }

            if let Err(e) = peer.value().send(msg.clone()).await {
                warn!("Failed to send message to {}: {}", peer.key(), e);
                // if sending fails, remove peer from state
                self.peers.remove(peer.key());
            }
        }
    }

    // add peer to state and return peer
    async fn add(
        &self,
        addr: SocketAddr,
        username: String,
        stream: Framed<TcpStream, LinesCodec>,
    ) -> Peer {
        let (tx, mut rx) = mpsc::channel(MAX_MESSAGE);
        self.peers.insert(addr, tx);

        let (mut stream_tx, stream_rx) = stream.split();

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Err(e) = stream_tx.send(msg.to_string()).await {
                    warn!("Failed to send message to {}: {}", addr, e);
                    break;
                }
            }
        });

        Peer {
            username,
            stream: stream_rx,
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::UserJoined(username) => write!(f, "{} joined the chat", username),
            Message::UserLeft(username) => write!(f, "{} left the chat", username),
            Message::Chat { sender, content } => write!(f, "{}: {}", sender, content),
        }
    }
}

use crate::transfer::service::{init_tracing, World};
use futures::{future, prelude::*};
use std::net::Ipv4Addr;
use std::net::{IpAddr, SocketAddr};
use tarpc::{
    context,
    server::{self, incoming::Incoming, Channel},
    tokio_serde::formats::Json,
};

// This is the type that implements the generated World trait. It is the business logic
// and is used to start the server.
#[derive(Clone)]
struct Server(SocketAddr);

#[tarpc::server]
impl World for Server {
    async fn hello(self, _: context::Context, name: String) -> String {
        format!("Hello, {name}! You are connected from {}", self.0)
    }

    async fn bye(self, _: context::Context) -> String {
        format!("Bye!")
    }
}

pub async fn create_server() -> anyhow::Result<()> {
    init_tracing("TARPC Server")?;
    let server_addr = (IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    listener.config_mut().max_frame_length(usize::MAX);
    listener
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
        .map(|channel| {
            let server = Server(channel.transport().peer_addr().unwrap());
            channel.execute(server.serve())
        })
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}

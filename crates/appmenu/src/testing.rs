use std::os::unix::net::UnixStream;

use zbus::connection::Builder;
use zbus::{Connection, Guid};

/// Two ends of a peer connection, no bus daemon. `serve` adds what the
/// server end offers from the start.
pub async fn pair(
    serve: impl FnOnce(Builder<'static>) -> Builder<'static>,
) -> (Connection, Connection) {
    let (server, client) = UnixStream::pair().unwrap();
    let server = serve(Builder::unix_stream(server).server(Guid::generate()).p2p()).build();
    let client = Builder::unix_stream(client).p2p().build();
    let (server, client) = futures_lite::future::zip(server, client).await;
    (server.unwrap(), client.unwrap())
}

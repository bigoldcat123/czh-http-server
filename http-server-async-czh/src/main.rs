use std::error::Error;

use futures::{SinkExt, StreamExt, future::select};
use http_server_async_czh::encoder::ResponseEncoder;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = tokio::net::TcpListener::bind("localhost:7788").await?;
    while let Ok((mut client, _)) = server.accept().await {
        let (r, w) = client.split();
        let writer = FramedWrite::new(w, ResponseEncoder::new());
        let mut e = FramedRead::new(r, LinesCodec::new());
        while let Some(Ok(e)) = e.next().await {
            println!("{:?}", e);
        }
    }
    Ok(())
}

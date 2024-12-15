// TODO: the `echo` server uses non-async primitives.
//  When running the tests, you should observe that it hangs, due to a
//  deadlock between the caller and the server.
//  Use `spawn_blocking` inside `echo` to resolve the issue.
use std::io::{Read, Write};
use tokio::net::TcpListener;

pub async fn echo(listener: TcpListener) -> Result<(), anyhow::Error> {
    loop {
        eprintln!("accepting...");
        let (socket, _) = listener.accept().await?;
        let peer_addr = socket.peer_addr().unwrap();
        eprintln!("accepted [{:#?}]", peer_addr);
        let mut socket = socket.into_std()?;
        socket.set_nonblocking(false)?;
        let mut buffer = Vec::new();
        // tokio::task::spawn_blocking(move || {
        eprintln!("read [{:#?}]", peer_addr);
            socket.read_to_end(&mut buffer)?;
        eprintln!("write [{:#?}]", peer_addr);
            socket.write_all(&buffer)?;
        // }).await??;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::panic;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::task::JoinSet;
    use console_subscriber;

    async fn bind_random() -> (TcpListener, SocketAddr) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        eprintln!("listening on {addr:#?}");
        (listener, addr)
    }

    #[tokio::test]
    async fn test_echo() {
        console_subscriber::init();

        let (listener, addr) = bind_random().await;
        tokio::spawn(echo(listener));

        let requests = vec![
            "hello here we go with a long message",
            "world",
            "foo",
            "bar",
        ];
        let mut join_set = JoinSet::new();

        for request in requests {
            eprintln!("spawn for request: <{request}>");
            join_set.spawn(async move {
                eprintln!("connecting...");
                let mut socket = tokio::net::TcpStream::connect(addr).await.unwrap();
                let local_addr = socket.local_addr().unwrap();
                eprintln!("connected...{:#?}", local_addr);
                let (mut reader, mut writer) = socket.split();

                eprintln!("send the request[{:#?}]: <{request}>", local_addr);
                // Send the request
                writer.write_all(request.as_bytes()).await.unwrap();
                // Close the write side of the socket
                writer.shutdown().await.unwrap();

                eprintln!("read the response[{:#?}] of <{request}>", local_addr);
                // Read the response
                let mut buf = Vec::with_capacity(request.len());
                reader.read_to_end(&mut buf).await.unwrap();
                assert_eq!(&buf, request.as_bytes());
            });
        }

        while let Some(outcome) = join_set.join_next().await {
            if let Err(e) = outcome {
                if let Ok(reason) = e.try_into_panic() {
                    panic::resume_unwind(reason);
                }
            }
        }
    }
}

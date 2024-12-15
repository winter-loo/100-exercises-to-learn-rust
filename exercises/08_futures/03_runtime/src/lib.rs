// TODO: Implement the `fixed_reply` function. It should accept two `TcpListener` instances,
//  accept connections on both of them concurrently, and always reply to clients by sending
//  the `Display` representation of the `reply` argument as a response.
use std::fmt::Display;
use std::panic;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::task::JoinSet;

pub async fn fixed_reply<T>(first: TcpListener, second: TcpListener, reply: T)
where
    // `T` cannot be cloned. How do you share it between the two server tasks?
    T: Display + Send + Sync + 'static,
{
    let reply = Box::new(format!("{}", reply));
    let reply2 = reply.clone();

    // create a static lifetime
    let reply = Box::leak(reply);
    let reply2 = Box::leak(reply2);

    let mut join_set = JoinSet::new();
    join_set.spawn(async move {
        loop {
            // see [panic boundary] section
            // In short, this panic will be captured by `join_set.join_next().await`
            let (mut socket, _) = first.accept().await.unwrap();
            let (_, mut tx) = socket.split();
            let _ = tx.write_all(reply.as_bytes()).await;
        }
    });
    join_set.spawn(async move {
        loop {
            let (mut socket, _) = second.accept().await.unwrap();
            let (_, mut tx) = socket.split();
            let _ = tx.write_all(reply2.as_bytes()).await;
        }
    });

    while let Some(outcome) = join_set.join_next().await {
        if let Err(e) = outcome {
            if let Ok(reason) = e.try_into_panic() {
                panic::resume_unwind(reason);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::panic;
    use tokio::io::AsyncReadExt;
    use tokio::task::JoinSet;

    async fn bind_random() -> (TcpListener, SocketAddr) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        (listener, addr)
    }

    #[tokio::test]
    async fn test_echo() {
        let (first_listener, first_addr) = bind_random().await;
        let (second_listener, second_addr) = bind_random().await;
        let reply = "Yo";
        tokio::spawn(fixed_reply(first_listener, second_listener, reply));

        let mut join_set = JoinSet::new();

        for _ in 0..3 {
            for addr in [first_addr, second_addr] {
                join_set.spawn(async move {
                    let mut socket = tokio::net::TcpStream::connect(addr).await.unwrap();
                    let (mut reader, _) = socket.split();

                    // Read the response
                    let mut buf = Vec::new();
                    reader.read_to_end(&mut buf).await.unwrap();
                    assert_eq!(&buf, reply.as_bytes());
                });
            }
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

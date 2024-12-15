use tokio::net::TcpListener;

// TODO: write an echo server that accepts incoming TCP connections and
//  echoes the received data back to the client.
//  `echo` should not return when it finishes processing a connection, but should
//  continue to accept new connections.
//
// Hint: you should rely on `tokio`'s structs and methods to implement the echo server.
// In particular:
// - `tokio::net::TcpListener::accept` to process the next incoming connection
// - `tokio::net::TcpStream::split` to obtain a reader and a writer from the socket
// - `tokio::io::copy` to copy data from the reader to the writer
pub async fn echo(listener: TcpListener) -> Result<(), anyhow::Error> {
    loop {
        match listener.accept().await {
            Ok((client, _)) => {
                tokio::spawn(async move {
                    loop {
                        // Wait for the socket to be readable
                        if let Err(_) = client.readable().await {
                            break;
                        }

                        // Creating the buffer **after** the `await` prevents it from
                        // being stored in the async task.
                        let mut buf = [0; 4096];

                        // Try to read data, this may still fail with `WouldBlock`
                        // if the readiness event is a false positive.
                        let n = match client.try_read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => n,
                            Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                                continue;
                            }
                            Err(_) => {
                                break;
                            }
                        };

                        // Wait for the socket to be writable
                        if let Err(_) = client.writable().await {
                            break;
                        }

                        // Try to write data, this may still fail with `WouldBlock`
                        // if the readiness event is a false positive.
                        match client.try_write(&buf[..n]) {
                            Ok(_) => {
                                continue;
                            }
                            Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                                continue;
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                println!("couldn't get client: {:?}", e);
                break;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_echo() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(echo(listener));

        let requests = vec!["hello", "world", "foo", "bar"];

        for request in requests {
            let mut socket = tokio::net::TcpStream::connect(addr).await.unwrap();
            let (mut reader, mut writer) = socket.split();

            // Send the request
            writer.write_all(request.as_bytes()).await.unwrap();
            // Close the write side of the socket
            writer.shutdown().await.unwrap();

            // Read the response
            let mut buf = Vec::with_capacity(request.len());
            reader.read_to_end(&mut buf).await.unwrap();
            assert_eq!(&buf, request.as_bytes());
        }
    }
}

// src/bin/server.rs
use std::{error::Error, io};
use tokio::{
    io::{AsyncReadExt, Interest},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() {
    let bind_addr = "0.0.0.0:1234";
    listen(bind_addr).await;
}

async fn listen(bind_addr: &str) {
    let listener = TcpListener::bind(bind_addr).await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_stream(stream).await.unwrap();
        });
    }
}

async fn handle_stream(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    println!("Connection from {}", stream.peer_addr().unwrap());
    let mut reply_queue: Vec<Vec<u8>> = Vec::new();
    let mut buf: [u8; 1024];
    loop {
        let ready = stream
            .ready(Interest::READABLE | Interest::WRITABLE)
            .await?;
        if ready.is_readable() {
            buf = [0; 1024];
            match stream.try_read(&mut buf) {
                Ok(0) => {
                    println!("Client disconnected");
                    return Ok(());
                }
                Ok(n) => {
                    println!("read {} bytes", n);
                    let mut result_buffer: Vec<u8> = Vec::with_capacity(n);
                    buf.take(n as u64).read_to_end(&mut result_buffer).await?;
                    reply_queue.push(result_buffer.to_vec());
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        if ready.is_writable() {
            if let Some(msg) = reply_queue.pop() {
                match stream.try_write(&msg) {
                    Ok(n) => {
                        println!("Wrote {} bytes", n);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
        }
    }
}

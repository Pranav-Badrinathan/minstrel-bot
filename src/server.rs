use std::{net::SocketAddr, num::NonZeroU64};

use tokio::{
	sync::{mpsc, watch}, 
	net::{TcpListener, TcpStream}, 
	io::{AsyncReadExt, AsyncWriteExt, ErrorKind}
};

use crate::passthrough;

pub async fn server_init(mut shutdown_rcv: watch::Receiver<()>){
	let addr = SocketAddr::from(([127, 0, 0, 1], 4242));

	// When the receiver get's something, it will prompt the webserver to shutdown.
	let listener = TcpListener::bind(&addr).await
		.expect("Listener creation error");

	println!("Listening on: {}", addr);

	// Define the connection loop here, but don't await it yet. (Won't execute)
	let connect_loop = async { 
		loop {
			match listener.accept().await {
				Ok((stream, addr)) => {
					println!("New client connected on: {:#?}", addr);
					tokio::spawn(handle_connection(stream));
				}
				Err(e) => println!("Connection error: Couldn't connect to client.\n{:#?}", e),
			}
		}
	};

	// Instead of awaiting, wait in tandem for a shutdown signal here.
	// So either the connection loop errors or we get a shutdown signal.
	tokio::select! {
		_ = shutdown_rcv.changed() => {},
		_ = connect_loop => {},
	}
}

async fn handle_connection(mut stream: TcpStream) {
	let mut id_buf = [0u8; 8];
	let mut guild_id: NonZeroU64 = NonZeroU64::MIN;

	while guild_id == NonZeroU64::MIN {
		match stream.read(id_buf.as_mut_slice()).await {
			Ok(_) => guild_id = u64::from_be_bytes(id_buf).try_into().unwrap_or(NonZeroU64::MIN),
			Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => continue,
			Err(_) => panic!("Guild ID read error! Remove this panic later"),
		};
	}

	let (tx, rx) = mpsc::channel(50);

	// First initialize the stream.
	tokio::spawn(
		passthrough::init_audio_stream(passthrough::OpusStream {
			rx,
			current_frame: None,
			chunk_pos: 0,
			pos: 0,
			guild_id
		})
	);

	loop {
		let mut data_buf = vec![0u8; 10000];

		let size = match stream.read(data_buf.as_mut_slice()).await {
			Ok(0) => continue,
			Ok(n) => n,
			Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
			Err(_) => continue,
		};

		data_buf.truncate(size);
		
		for frame in split_frames(&data_buf).await {
			tx.send(frame).await.unwrap();
		}

		let _ = stream.write_u8(0u8).await;
	}
}

async fn split_frames(data: &[u8]) -> impl Iterator<Item = Vec<u8>> {
	let mut split: Vec<Vec<u8>> = Vec::new();
	let mut pos: usize = 0;
	while pos < data.len() {
		let size = i16::from_le_bytes([data[pos], data[pos+1]]) as usize;
		pos += 2;

		split.push(data[pos..pos+size].to_vec());
		pos += size;
	}

	split.into_iter()
}

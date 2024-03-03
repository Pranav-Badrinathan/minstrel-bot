use std::{net::SocketAddr, num::NonZeroU64};

use tokio::{
	sync::watch, 
	net::{TcpListener, TcpStream}, 
	io::{AsyncReadExt, AsyncWriteExt}
};

use crate::bot;

pub async fn server_init(mut rcv: watch::Receiver<()>){
	let addr = SocketAddr::from(([127, 0, 0, 1], 4242));

	// When the receiver get's something, it will prompt the webserver to shutdown.
	let listener = TcpListener::bind(&addr).await.expect("Listener creation error"); 
	println!("Listening on: {}", addr);

	let connect_loop = async { 
		loop {
			match listener.accept().await {
				Ok((stream, addr)) => {
					println!("New client connected: {:#?}", addr);
					tokio::spawn(handle_connection2(stream));
				}
				Err(e) => println!("Connection error: Couldn't connect to client.\n{:#?}", e),
			}
		}
	};

	tokio::select! {
		_ = rcv.changed() => {},
		_ = connect_loop => {},
	}
}

// async fn handle_connection(mut stream: TcpStream) {
//	let mut id_buf = [0u8; 8];
//	let mut guild_id: NonZeroU64 = NonZeroU64::MIN;
//
//	while guild_id == NonZeroU64::MIN {
//		match stream.read(id_buf.as_mut_slice()).await {
//			Ok(_) => guild_id = u64::from_be_bytes(id_buf).try_into().unwrap_or(NonZeroU64::MIN),
//			Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => continue,
//			Err(_) => panic!("Guild ID read error! Remove this panic later"),
//		};
//	}
//
//	loop {
//		let mut data_buf = vec![0u8; 10000];
//
//		let size = match stream.read(data_buf.as_mut_slice()).await {
//			Ok(0) => continue,
//			Ok(n) => n,
//			Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => continue,
//			Err(_) => break,
//		};
//
//		data_buf.truncate(size);
//		dbg!(data_buf.len());
//
//		let json = br#"{"dca":{"version":1,"tool":{"name":"opus-rs","version":"1.0.0","url":null,"author":null}},"opus":{"mode":"voip","sample_rate":48000,"frame_size":2880,"abr":null,"vbr":true,"channels":1},"info":null,"origin":null,"extra":null}"#;
//	
//		let mut buf = b"DCA1".to_vec();
//		buf.extend(i32::to_le_bytes(json.len() as i32));
//		buf.extend(json);
//		buf.extend(data_buf);
//
//		bot::play_music(
//			AudioSet { 
//				guild_id,
//				audio_data: buf,
//		}).await;
//
//		let _ = stream.write_u8(0u8).await;
//	}
// }
async fn handle_connection2(mut stream: TcpStream) {
	use tokio::sync::mpsc;
	let mut id_buf = [0u8; 8];
	let mut guild_id: NonZeroU64 = NonZeroU64::MIN;

	while guild_id == NonZeroU64::MIN {
		match stream.read(id_buf.as_mut_slice()).await {
			Ok(_) => guild_id = u64::from_be_bytes(id_buf).try_into().unwrap_or(NonZeroU64::MIN),
			Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => continue,
			Err(_) => panic!("Guild ID read error! Remove this panic later"),
		};
	}

	let (tx, rx) = mpsc::channel(100);

	// First initialize the stream.
	tokio::spawn(
		bot::play_music2(bot::OpusStream {
			rx,
			current_frame: None,
			chunk_pos: 0,
			pos: 0,
			guild_id
		})
	);

	loop {
		let mut data_buf = vec![0u8; 1000];

		let size = match stream.read(data_buf.as_mut_slice()).await {
			Ok(0) => continue,
			Ok(n) => n,
			Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => continue,
			Err(_) => break,
		};

		data_buf.truncate(size);
		tx.send(data_buf).await.unwrap();

		let _ = stream.write_u8(0u8).await;
		println!("SENT ACK!");
	}
}

#[derive(Debug)]
pub struct AudioSet {
	pub guild_id: NonZeroU64,
	pub audio_data: Vec<u8>,
}

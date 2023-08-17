use std::net::SocketAddr;

use tokio::{sync::watch, net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}};

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
					tokio::spawn(handle_connection(stream));
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

async fn handle_connection(mut stream: TcpStream) {
	let mut id_buf = [0u8; 8];
	let mut guild_id: u64 = 0u64;

	while guild_id == 0 {
		match stream.read(id_buf.as_mut_slice()).await {
			Ok(_) => guild_id = u64::from_be_bytes(id_buf),
			Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => continue,
			Err(_) => panic!("Guild ID read error! Remove this panic later"),
		};
	}

	loop {
		let mut data_buf = vec![0u8; 1000];

		let size = match stream.read(data_buf.as_mut_slice()).await {
			Ok(0) => continue,
			Ok(n) => n,
			Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => continue,
			Err(_) => break,
		};

		data_buf.truncate(size);
		dbg!(data_buf.len());

		// Add the required size to the start of the frame as per DCA requirements.
		data_buf.splice(0..0, i16::to_le_bytes(size as i16));

		bot::play_music(
			AudioSet { 
				guild_id,
				audio_data: data_buf,
			}).await;

		let _ = stream.write_u8(0u8).await;
	}
}

#[derive(Debug)]
pub struct AudioSet {
	pub guild_id: u64,
	pub audio_data: Vec<u8>,
}

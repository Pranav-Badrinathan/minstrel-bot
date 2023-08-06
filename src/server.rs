use std::net::SocketAddr;

use tokio::{sync::{watch, mpsc}, net::{TcpListener, TcpStream}};

pub async fn server_init(mut rcv: watch::Receiver<u8>, ad_send: mpsc::Sender<AudioSet>){
	let addr = SocketAddr::from(([127, 0, 0, 1], 4242));

	// When the receiver get's something, it will prompt the webserver to shutdown.
	let listener = TcpListener::bind(&addr).await.expect("Listener creation error"); 
	println!("Listening on: {}", addr);

	loop {
		match listener.accept().await {
		    Ok((stream, addr)) => {
				println!("New client connected: {:#?}", addr);
				tokio::spawn(handle_connection(stream));
			}
			Err(e) => println!("Connection error: Couldn't connect to client.\n{:#?}", e),
		}
	}
}

async fn handle_connection(stream: TcpStream) {
	loop {
		let mut buf = vec![0u8; 5000];

		let size = match stream.try_read(buf.as_mut_slice()) {
			Ok(0) => continue,
			Ok(n) => n,
			Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => continue,
			Err(_) => break,
		};

		buf.truncate(size);
		dbg!(buf.len());
	}
	
}

#[derive(Debug)]
pub struct AudioSet {
	pub guild_id: u64,
	pub audio_data: Vec<u8>,
}

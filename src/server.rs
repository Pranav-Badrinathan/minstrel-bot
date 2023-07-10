use std::net::SocketAddr;

use axum::{Router, routing::*, http::Request, body::Body};
use tokio::sync::{watch, mpsc};

pub async fn server_init(mut rcv: watch::Receiver<u8>, ad_send: mpsc::Sender<AudioSet>) {
	let routes = Router::new()
		.route("/", 
			post(|req: Request<Body>| async move {
				if let Some(head_id) = req.headers().get("guild_id") {
					let id = match u64::from_str_radix(head_id.to_str().unwrap(), 10) {
						Ok(id) => id,
						Err(_) => return
					};

					let frame_count = match req.headers().get("frame_count") {
						Some(count) => u8::from_str_radix(count.to_str().unwrap(), 10).unwrap_or(0u8),
						None => 0u8
					};

					// println!("{}", frame_count);

					let body: Vec<u8> = hyper::body::to_bytes(req.into_body()).await.unwrap().into();
					// let body = [[44, 43, 41, 31, 2, 0,  0,  0, 123, 125].to_vec(), body].concat();

					let mut pos: usize = 0;

					for _ in [..frame_count] {

						let frame_size = i16::from_le_bytes([body[pos], body[pos + 1]]);

						ad_send.send(
							AudioSet { 
								guild_id: id,
								audio_data: body[pos..(frame_size as usize) + 2].to_vec(),
							}).await
							.expect("Bad data sending");

						pos += (frame_size as usize) + 2;
					}
				}
			}
		));

	let addr = SocketAddr::from(([127, 0, 0, 1], 4242));

	// When the receiver get's something, it will prompt the webserver to shutdown.
	let serv = axum::Server::bind(&addr)
		.serve(routes.into_make_service())
		.with_graceful_shutdown(async {
			rcv.changed().await.ok();
	});

	// Await the `server` receiving the signal...
	if let Err(e) = serv.await {
		eprintln!("server error: {}", e);
	}
}

#[derive(Debug)]
pub struct AudioSet {
	pub guild_id: u64,
	pub audio_data: Vec<u8>,
}

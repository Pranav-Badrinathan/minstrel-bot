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

					let mut body: Vec<u8> = hyper::body::to_bytes(req.into_body()).await.unwrap().into();
					body = [(body.len() as i16).to_le_bytes().to_vec(), body].concat();

					ad_send.send(
						AudioSet { 
							guild_id: id,
							audio_data: body, })
						.await
						.expect("Bad data sending");
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

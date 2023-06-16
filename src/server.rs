use std::net::SocketAddr;

use axum::{Router, routing::*, http::Request, body::Body};
use tokio::sync::{watch, mpsc};

pub async fn server_init(rcv: watch::Receiver<u8>, ad_send: mpsc::Sender<Vec<u8>>) {

	let routes = Router::new()
		.route("/", 
			post(|req: Request<Body>| async move {
				if let Some(id) = req.headers().get("guild_id") {
					let body: Vec<u8> = hyper::body::to_bytes(req.into_body()).await.unwrap().into();
					ad_send.send(body).await.expect("Bad data sending");
				}
			}
		));

	let addr = SocketAddr::from(([127, 0, 0, 1], 4242));

	// When the receiver get's something, it will prompt the webserver to shutdown.
	axum::Server::bind(&addr)
		.serve(routes.into_make_service()) 
		.with_graceful_shutdown(shutdown(rcv));
}

async fn shutdown(mut rcv: watch::Receiver<u8>) {
	rcv.changed().await.expect("Error getting shutdown message")
}


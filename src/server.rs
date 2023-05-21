use std::net::SocketAddr;

use axum::{Router, routing::*};
use tokio::sync::mpsc;


pub async fn server_init(mut rcv: mpsc::Receiver<crate::State>) {

	let routes = Router::new().route("/", post(|| async { println!("Damn son. We got it.") }));

	let addr = SocketAddr::from(([127, 0, 0, 1], 4242));

	// axum::Server::bind(&addr)
	// 	.serve(routes.into_make_service())
	// 	.await.unwrap();
	
	// Run both the server and the shutdown reciever in parallel. When either the server errors
	// or the shutdown flag is recieved, gracefully shutdown.
	tokio::select! {
		status = axum::Server::bind(&addr)
					.serve(routes.into_make_service()) => {
			if let Err(why) = status { println!("Webserver Error: {why}"); }
		},

		flag = rcv.recv() => {
			if let Some(state) = flag {
				match state {
					crate::State::Shutdown => { /* TODO: Server Shutdown*/ },
					crate::State::Restart => {},
				}
			}
		},
	}
}


use std::net::SocketAddr;

use axum::{Router, routing::*, http::Request, body::Body};
use tokio::sync::watch;


pub async fn server_init(mut rcv: watch::Receiver<u8>) {

	let routes = Router::new().route("/", post(catch_post));

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

		_flag = rcv.changed() => {},
	}
}

pub async fn catch_post(req: Request<Body>) {
	if let Some(id) = req.headers().get("guild_id") {
		tokio::spawn(
			crate::bot::play_music(
				u64::from_str_radix(id.to_str().unwrap(), 10).expect("GuildID not an int")
			)
		);
	}
}


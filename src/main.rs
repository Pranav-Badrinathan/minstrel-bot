mod bot;
mod server;
mod commands { pub mod defs; pub mod run; }

use tokio::{task, sync::{watch, mpsc}};

#[tokio::main]
async fn main() {
	dotenv::dotenv().ok();

	let (s_send, s_recv) = watch::channel(0u8);
	let (ad_send, ad_recv) = mpsc::channel::<Vec<u8>>(100);

	let bot_task = task::spawn(bot::bot_init(s_recv.clone(), ad_recv));
	let server_task = task::spawn(server::server_init(s_recv, ad_send));
	let shutdown_task = task::spawn(shutdown(s_send));

	tokio::try_join!(server_task, bot_task, shutdown_task).expect("Error encountered in Server-Bot concurrency...");
}

// Handle Gracefully shutting down. Use the mpsc channels to send shutdown messages.
async fn shutdown(s_send: watch::Sender<u8>) {
	// Graceful Shutdown with Ctrl-C
	match tokio::signal::ctrl_c().await {
		Ok(_) => {
			println!("Ctrl-C Recieved. Shutting down!");

			// Send the shutdown signal to the server and the bot via the mpsc channel.
			if let Err(_) = s_send.send(0) {
				eprintln!("Server reciever dropped. Can't gracefully shutdown!");
			}

			if let Err(_) =	s_send.send(0) {
				eprintln!("Bot reciever dropped. Can't gracefully shutdown!");
			}
		},
		Err(err) => {
			eprintln!("Error listening to Shutdown signal. Error: {err}");
		},
	}
}


mod bot;
mod server;
mod passthrough;
mod commands { pub mod defs; pub mod run; }

use tokio::{task, sync::watch};

#[tokio::main]
async fn main() {
	dotenv::dotenv().ok();

	// Shutdown send and recv
	let (s_send, s_recv) = watch::channel(());

	let bot_task = task::spawn(bot::bot_init(s_recv.clone()));
	let server_task = task::spawn(server::server_init(s_recv));
	let shutdown_task = task::spawn(shutdown(s_send));

	tokio::try_join!(server_task, bot_task, shutdown_task)
		.expect("Error encountered in Server-Bot concurrency...");
}

// Handle Gracefully shutting down. Use the mpsc channels to send shutdown messages.
async fn shutdown(s_send: watch::Sender<()>) {
	// Graceful Shutdown with Ctrl-C
	match tokio::signal::ctrl_c().await {
		Ok(_) => {
			println!("Ctrl-C Recieved. Shutting down!");

			// Send the shutdown signal to the server and the bot via the mpsc channel.
			if let Err(_) = s_send.send(()) {
				eprintln!("Bot or Server reciever dropped. Can't gracefully shutdown!");
			}
		},
		Err(err) => {
			eprintln!("Error listening to Shutdown signal. Error: {err}");
		},
	}
}


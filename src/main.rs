mod bot;
mod server;

use tokio::{task, sync::mpsc};

pub enum State {
	Shutdown,
	Restart,
}

#[tokio::main]
async fn main() {
	dotenv::dotenv().ok();

	let (s_send, s_recv) = mpsc::channel(100);
	let (b_send, b_recv) = mpsc::channel(100);

	let server_task = task::spawn(server::server_init(s_recv));
	let bot_task = task::spawn(bot::bot_init(b_recv));
	let shutdown_task = task::spawn(shutdown(s_send, b_send));

	tokio::try_join!(server_task, bot_task, shutdown_task).expect("Error encountered in Server-Bot concurrency...");
}

async fn shutdown(s_send: mpsc::Sender<State>, b_send: mpsc::Sender<State>) {
	// Graceful Shutdown with Ctrl-C
	match tokio::signal::ctrl_c().await {
		Ok(_) => {
			println!("Ctrl-C Recieved. Shutting down!");

			// Send the shutdown signal to the server and the bot via the mpsc channel.
			if let Err(_) = s_send.send(State::Shutdown).await {
				eprintln!("Server reciever dropped. Can't gracefully shutdown!");
			}

			if let Err(_) =	b_send.send(State::Shutdown).await {
				eprintln!("Bot reciever dropped. Can't gracefully shutdown!");
			}
		},
		Err(err) => {
			eprintln!("Error listening to Shutdown signal. Error: {err}");
		},
	}
}


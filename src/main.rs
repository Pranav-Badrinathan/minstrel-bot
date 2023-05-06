mod bot;

use tokio::task;
use serenity::{async_trait, prelude::*};

struct Handler;

impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
	dotenv::dotenv().ok();

	let server_task = task::spawn(server_func());
	let bot_task = task::spawn(bot::bot_init());

	tokio::try_join!(server_task, bot_task).expect("Error encountered in Server-Bot concurrency...");
}

async fn server_func() {
	
}


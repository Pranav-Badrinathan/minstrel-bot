use std::sync::Arc;
use lazy_static::lazy_static;
use tokio::sync::{watch, OnceCell};

use serenity::{
	async_trait, 
	prelude::*, 
	model::{
		prelude::Ready,
		application::{Command, Interaction}
	}
};
use songbird::{SerenityInit, Songbird, SongbirdKey};

use crate::commands;

lazy_static!{
	pub static ref SONG: OnceCell<Arc<Songbird>> = OnceCell::new();
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn interaction_create(&self, ctx: Context, interaction:  Interaction) {
		if let Interaction::Command(command) = interaction {
			println!("Recieved an application command: {0}", command.data.name.as_str());

			let resp = match command.data.name.as_str() {
				"ping" => { commands::run::ping(ctx.http, command).await },
				"id" => { commands::run::id(ctx.http, command).await },
				"join" => { commands::run::join(ctx.clone(), command).await },
				"leave" => { commands::run::leave(ctx, command).await },
				_ => { commands::run::unimplemented(ctx.http, command).await },
			};

			if let Err(why) = resp {
				println!("Cannot respond to slash command: {:#?}", why);
			}
		}
	}

	async fn ready(&self, ctx: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);

		// Define the slash commands that are availaible to all guilds.
		let _slash_commands = Command::set_global_commands(&ctx.http, vec![
			commands::defs::ping(),
			commands::defs::id(),
			commands::defs::roll(),
			commands::defs::join(),
			commands::defs::leave(),
		]).await;
	}
}

// Bot initialization function
pub async fn bot_init(mut shutdown_rcv: watch::Receiver<()>) {
	// Token in .env file not on Git. Get the token from discord dev portal. 
	let token = std::env::var("BOT_TOKEN")
		.expect("Token not found in environment!!!");

	let mut client = Client::builder(token, GatewayIntents::all())
		.event_handler(Handler)
		.register_songbird()
		.await
		.expect("Error creating the Client.");

	// Store songbird instance in OnceCell for later use in passthrough.
	let sb = client.data.read().await
		.get::<SongbirdKey>()
		.expect("Failed to get or initialize Songbird")
		.clone();

	SONG.set(sb).expect("Error setting Songbird (SONG)");

	// Run both the bot and the shutdown reciever in parallel. When either the 
	// bot errors (when the client.start() ends) or the shutdown flag is 
	// recieved, gracefully shutdown.
	tokio::select! {
		start = client.start() => {
			if let Err(why) = start {
				println!("Client error: {:?}", why);
			}
		},
		_ = shutdown_rcv.changed() => {
			let sm = client.shard_manager.clone();
			sm.shutdown_all().await;
		},
	}
}

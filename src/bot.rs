use serenity::{prelude::*, async_trait, model::prelude::{interaction::Interaction, Ready}};
use tokio::sync::mpsc;
use crate::State;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn interaction_create(&self, ctx: Context, interaction:  Interaction) {
		if let Interaction::ApplicationCommand(command) = interaction {
			dbg!("Recieved a application command interaction (slash command): {:#?}", &command);
			println!("{0}", command.data.name.as_str());
		}
	}

	async fn ready(&self, ctx: Context, ready: Ready) {
		dbg!("Bot is connected!");
	}
}

pub async fn bot_init(mut rcv: mpsc::Receiver<State>) {
	// Token stored in .env file not on Git. Get the token from discord dev portal.	
	let token = std::env::var("BOT_TOKEN").expect("Token not found in environment!!!");
	let mut client = Client::builder(token, GatewayIntents::empty())
		.event_handler(Handler).await
		.expect("Error creating the Client.");

	// Run both the bot and the shutdown reciever in parallel. When either the bot errors
	// (when the client.start() ends) or the shutdown flag is recieved, gracefully shutdown.
	tokio::select! {
		start = client.start() => {
			if let Err(why) = start {
        		println!("Client error: {:?}", why);
			}
		},
		flag = rcv.recv() => {
			if let Some(state) = flag {
				let sm = client.shard_manager.clone();
				match state {
		    		State::Shutdown => sm.lock().await.shutdown_all().await,
					State::Restart => (),
				}
			}
		},
	}
}

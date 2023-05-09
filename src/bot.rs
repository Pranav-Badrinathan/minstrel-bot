use serenity::
	{prelude::*, async_trait, model::prelude::
		{interaction::Interaction, 
			Ready, 
			command::Command,
			GuildId}};

use tokio::sync::mpsc;
use crate::{State, commands};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn interaction_create(&self, ctx: Context, interaction:  Interaction) {
		if let Interaction::ApplicationCommand(command) = interaction {
			println!("Recieved an application command: {0}", command.data.name.as_str());

			let resp = match command.data.name.as_str() {
				"ping" => { commands::run::ping(ctx.http, command).await },
				"id" => { commands::run::id(ctx.http, command).await },
				"join" => { commands::run::join(ctx.http, command).await },
				_ => { commands::run::unimplemented(ctx.http, command).await },
			};

			if let Err(why) = resp {
				println!("Cannot respond to slash command: {:#?}", why);	
			}
		}
	}

	async fn ready(&self, ctx: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
	
		// Define the slash commands that are availaible to all guilds with this bot.
		let _slash_commands = Command::set_global_application_commands(&ctx.http, |comms| {
			comms
			.create_application_command(|c| { commands::defs::ping(c) })
			.create_application_command(|c| { commands::defs::id(c) })
			.create_application_command(|c| { commands::defs::roll(c) })
		}).await;

		// println!("Set Global Commands: {:#?}", slash_commands);

		// Define slash commands availaible to only a particular guild.
		let _guildslash = GuildId::create_application_command(
			&GuildId(std::env::var("SERV_ID").expect("No Server ID found!").parse().expect("ID not an INT")), 
			&ctx.http, |c|{ commands::defs::join(c) }).await;
	}
}

// Bot initialization function
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

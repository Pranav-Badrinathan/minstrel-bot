use serenity::{prelude::*, async_trait, model::prelude::{interaction::{Interaction, InteractionResponseType}, Ready, command::{Command, CommandOptionType}, GuildId}};
use tokio::sync::mpsc;
use crate::State;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn interaction_create(&self, ctx: Context, interaction:  Interaction) {
		if let Interaction::ApplicationCommand(command) = interaction {
			// dbg!("Recieved a application command interaction (slash command): {:#?}", &command);
			println!("{0}", command.data.name.as_str());

			let resp = match command.data.name.as_str() {
				"ping" => { "Pong".to_string() },
				"id" => {
					format!("Server ID: {0}", command.guild_id.unwrap())
				},
				_ => { "Not implemented yet!".to_string() },
			};

			if let Err(why) = command.create_interaction_response(&ctx.http, |r| {
				r.kind(InteractionResponseType::ChannelMessageWithSource)
				 .interaction_response_data(|m| m.content(resp))
			}).await {
				println!("Cannot respond to slash command: {:#?}", why);	
			}
		}
	}

	async fn ready(&self, ctx: Context, _ready: Ready) {
		dbg!("Bot is connected!");
		
		let _slash_commands = Command::set_global_application_commands(&ctx.http, |commands| {
			commands
			.create_application_command(|c| {
				c.name("ping").description("Pings Minstrel. A test command.")
			})
			.create_application_command(|c| {
				c.name("id").description("Get's the current server's ID.")
			})
			.create_application_command(|c| {
				c.name("roll").description("Rolls dice based on input string.")
					.create_option(|o| {
						o.name("dice").description("eg: 1d20, 2d20kh, 5d6k3l")
							.kind(CommandOptionType::String).required(true)
					})
			})
		}).await;

		// println!("Set Global Commands: {:#?}", slash_commands);

		let _guildslash = GuildId::create_application_command(
			&GuildId(std::env::var("SERV_ID").expect("No Server ID found!").parse().expect("ID not an INT")), 
			&ctx.http, |c|{
			c.name("join").description("The bot joins the voice channel you are currently in")
		}).await;
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

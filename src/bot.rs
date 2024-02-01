use std::sync::Arc;
use lazy_static::lazy_static;
use serenity::
	{prelude::*, async_trait, model::prelude::
		Ready};
use serenity::model::application::{Command, Interaction};

use tokio::sync::{watch, OnceCell};
use crate::{commands, server::AudioSet};

use songbird::{SerenityInit, SongbirdKey, Songbird};

lazy_static!{
	static ref SONG: OnceCell<Arc<Songbird>> = OnceCell::new();
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
	
		// Define the slash commands that are availaible to all guilds with this bot.
		let _slash_commands = Command::set_global_commands(&ctx.http, vec![
			commands::defs::ping(),
			commands::defs::id(),
			commands::defs::roll(),
			commands::defs::join(),
			commands::defs::leave(),
		]).await;

		// println!("Set Global Commands: {:#?}", _slash_commands);

		// Define slash commands availaible to only a particular guild.
		// let _guildslash = GuildId::set_application_commands(
		// 	&GuildId(std::env::var("GUILD_ID").expect("No Server ID found!")
		// 		.parse().expect("ID not an INT")), 
		// 	&ctx.http, |coms|{ coms.create_application_command(|c| commands::defs::join(c)) }).await;
	}
}

// Bot initialization function
pub async fn bot_init(mut rcv: watch::Receiver<()>) {
	// Token stored in .env file not on Git. Get the token from discord dev portal.	
	let token = std::env::var("BOT_TOKEN").expect("Token not found in environment!!!");

	// Custom data that can be accessed in the callback functions
	// let mut cstm_ctx_data: HashMap<String, CustomData> = HashMap::new();
	// cstm_ctx_data.insert("recv".to_string(), CustomData::Receiver(ad_rcv));
	
	let mut client = Client::builder(token, GatewayIntents::all())
		.event_handler(Handler)
		.register_songbird()
		// .type_map_insert::<CustomData>(cstm_ctx_data)
		.await
		.expect("Error creating the Client.");

	let sb = client.data.read().await
		.get::<SongbirdKey>()
		.expect("Failed to get or initialize Songbird")
		.clone();

	SONG.set(sb).expect("Error setting Songbird (SONG)");

	// Run both the bot and the shutdown reciever in parallel. When either the bot errors
	// (when the client.start() ends) or the shutdown flag is recieved, gracefully shutdown.
	tokio::select! {
		start = client.start() => {
			if let Err(why) = start {
        		println!("Client error: {:?}", why);
			}
		},
		_ = rcv.changed() => {
			let sm = client.shard_manager.clone();
			sm.shutdown_all().await;
		},
	}
}

pub async fn play_music(set: AudioSet) {
	use songbird::input::{
		Input,
		LiveInput,
		AudioStream
	};

	let sb = SONG.get().expect("Songbird not found!").clone();

	if let Some(h) = sb.get(set.guild_id) {
		let mut handler = h.lock().await;

		// println!("NEXT PACKET");
		// let audio: Input = Input::new(
		// 	true, 
		// 	Reader::from_memory(set.audio_data), 
		// 	Codec::Opus(OpusDecoderState::new().unwrap()),
		// 	Container::Dca { first_frame: 0 },
		// 	None
		// );
		
		// let audio: Input = Input::Live(
		// 	LiveInput::Raw(
		// 		AudioStream { 
		// 			input: set.audio_data.into(), 
		// 			hint: None 
		// 		}
		// 	),
		// 	None
		// );
		// 	
		// let track_handle = handler.play_input(audio);
		// while track_handle.get_info().await.unwrap().playing != songbird::tracks::PlayMode::End {}
	}
}

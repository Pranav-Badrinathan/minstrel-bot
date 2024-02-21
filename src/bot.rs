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

use crate::{commands, server::AudioSet};

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
		codecs::{CODEC_REGISTRY, PROBE}, 
	};

	let sb = SONG.get().expect("Songbird not found!").clone();

	if let Some(h) = sb.get(set.guild_id) {
		let mut handler = h.lock().await;

		let audio: Input = set.audio_data.into();
		let audio: Input = audio.make_playable_async(&CODEC_REGISTRY, &PROBE).await
			.expect("Can't make audio playable!");

		let track_handle = handler.play_input(audio);

		while track_handle.get_info().await.unwrap().playing == songbird::tracks::PlayMode::Play {}
	}
}

/// Experimental stuff beyond. Will be modified if found to work.

use tokio::sync::mpsc;
use songbird::constants::SILENT_FRAME;

lazy_static! {
	static ref DCA1_HEADER: Vec<u8> = [
		b"DCA1".to_vec(),
		vec![224, 0, 0, 0],
		br#"{"dca":{"version":1,"tool":{"name":"opus-rs","version":"1.0.0","url":null,"author":null}},"opus":{"mode":"voip","sample_rate":48000,"frame_size":960,"abr":null,"vbr":true,"channels":2},"info":null,"origin":null,"extra":null}"#.to_vec()
	].concat();
}

struct OpusStream {
	// 20 ms opus frame Receiver.
	rx: mpsc::Receiver<Vec<u8>>,
	current_frame: Option<Vec<u8>>,
	chunk_pos: usize,
	pos: usize,
}

impl std::io::Read for OpusStream {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		// If the position in the stream is under header len, headers have not
		// been passed in yet. Pass them in.
		// Return or not idk. Testing still.
		//
		// ELSE if there is not frame selected, get one.
		if self.pos < DCA1_HEADER.len() {
			let size = DCA1_HEADER.as_slice().read(buf)?;
			self.pos = size;
			self.chunk_pos =  0;
			return Ok(size);

		} else if self.current_frame.is_none() {
			match self.rx.try_recv() {
				Ok(fr) => {
					self.current_frame = Some(fr);
					self.chunk_pos = 0
				}
				Err(mpsc::error::TryRecvError::Empty) => self.current_frame = Some(SILENT_FRAME.to_vec()),
				Err(mpsc::error::TryRecvError::Disconnected) => return Ok(0),
			}
		}

		// Frame definitely exists, as cond above will fill it with something.
		let frame = self.current_frame.clone().unwrap();

		// Now, we fill the buffer with the frame size information DCA expects
		// to recieve.
		//
		// ELSE we have already given size info, so give actual audio data now.
		if self.chunk_pos < 2 {
			let size = (frame.len() as u16).to_le_bytes().as_slice().read(buf)?;

			self.chunk_pos = size; // Size will always be 2, cause 16 bit.
			self.pos += size;

			Ok(size)
			
		} else {
			let size = frame.as_slice().read(buf)?;

			self.chunk_pos = 0;
			self.pos += size;

			Ok(size)
		}
	}
}

pub async fn play_music2(set: AudioSet) {
	// use songbird::input::{
	// 	Input,
	// 	codecs::{CODEC_REGISTRY, PROBE}, 
	// };
	//
	// let sb = SONG.get().expect("Songbird not found!").clone();
	//
	// if let Some(h) = sb.get(set.guild_id) {
	// 	let mut handler = h.lock().await;
	//
	// 	let audio: Input = set.audio_data.into();
	// 	let audio: Input = audio.make_playable_async(&CODEC_REGISTRY, &PROBE).await
	// 		.expect("Can't make audio playable!");
	//
	// 	let track_handle = handler.play_input(audio);
	//
	// 	while track_handle.get_info().await.unwrap().playing == songbird::tracks::PlayMode::Play {}
	// }
}

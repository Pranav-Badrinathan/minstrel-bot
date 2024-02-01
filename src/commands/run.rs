use std::sync::Arc;

use serenity::
	{all::{CommandDataOptionValue, CommandInteraction}, builder::{CreateInteractionResponse, CreateInteractionResponseMessage}, http::Http, model::prelude::ChannelId, prelude::Context};

pub async fn ping(http:Arc<Http>, c: CommandInteraction) -> Result<(), serenity::Error> {
	c.create_response(http, CreateInteractionResponse::Message(
		CreateInteractionResponseMessage::default()
			.content("Pong! :ping_pong:"))).await
}

pub async fn id(http:Arc<Http>, c: CommandInteraction) -> Result<(), serenity::Error> {
	c.create_response(http, CreateInteractionResponse::Message(
		CreateInteractionResponseMessage::default()
			.content(format!("Server ID: {}", c.guild_id.unwrap()))
			.ephemeral(true))).await
}

pub fn _roll() {

}

pub async fn join(ctx: Context, c: CommandInteraction) -> Result<(), serenity::Error> {
	let resp: String;
	let channel: Option<ChannelId>;

	// Scope cause compiler freaks out about the guild variable persisting through
	// awaits, while it really does not.
	{
		let guild = ctx.cache.guild(c.guild_id.expect("Error getting GuildId!"))
			.expect("Can't get the guild. Did you forget to set the GUILD intents in Client::builder?");	
	
	
		// If a channel is specifically provided, use it. Else if the user is in a voice channel, use that.
		// Else, send out an error as channel is not specified!
		if let Some(id) = c.data.options.get(0) {
			if let CommandDataOptionValue::Channel(x) = id.value {
				channel = Some(x);
				resp = "Connected!".to_string();
			}
			else {
			    channel = None;
				resp = "Provided parameter not a channel! (How did you even manage this?!)".to_string();
			}
		}
		// If invoking user is in a vc, connect. Else, respond with the error.
		else if let Some(vs) = guild.voice_states.get(&c.user.id) {
				channel = Some(vs.channel_id.expect("User is in a channel that does not exist!"));
				resp = "Connected!".to_string();
		}
		// No voice channel is provided AND user is not in a vc. No idea what to connect to. Error.
		else {
				channel = None;
				resp = "You are not in a voice channel! \
					Specify a channel or connect to a voice channel and try again.".to_string();
		}
	}
	if let Some(channel) = channel {
		// Connect with songbird.
		let _ = songbird::get(&ctx).await.expect("Songbird not registered").clone()
			.join(c.guild_id.unwrap(), channel).await;
	}

	c.create_response(&ctx.http, CreateInteractionResponse::Message(
		CreateInteractionResponseMessage::default().content(resp))).await
}

pub async fn leave(ctx: Context, c: CommandInteraction) -> Result<(), serenity::Error> {
	let resp: String;
	let manager = songbird::get(&ctx).await.expect("Songbird not registered.");

	if manager.get(c.guild_id.expect("Error getting GuildID")).is_some() {
		if let Err(why) = manager.remove(c.guild_id.expect("")).await {
			resp = format!("Failed to leave: {}", why);
		}
		else {
			resp = "Left voice channel.".to_string();
		}
	}
	else {
		resp = "Not in a voice channel!".to_string();
	}

	c.create_response(ctx.http, CreateInteractionResponse::Message(
		CreateInteractionResponseMessage::default().content(resp))).await
}


// ----------------------------------------------------------------------

pub async fn unimplemented(http:Arc<Http>, c: CommandInteraction) -> Result<(), serenity::Error> {
	c.create_response(http, CreateInteractionResponse::Message(
		CreateInteractionResponseMessage::default().content("Not implemented yet! :("))).await
}

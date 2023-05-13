use std::sync::Arc;

use serenity::
	{model::prelude::{interaction::
		{application_command::{ApplicationCommandInteraction, CommandDataOptionValue}, InteractionResponseType}, Guild, ChannelId}, 
	http::Http, prelude::Context};

pub async fn ping(http:Arc<Http>, c: ApplicationCommandInteraction) -> Result<(), serenity::Error> {
	c.create_interaction_response(http, |r| {
		r.kind(InteractionResponseType::ChannelMessageWithSource)
		 .interaction_response_data(|m| m.content("Pong! :ping_pong:".to_string()))
	}).await
}

pub async fn id(http:Arc<Http>, c: ApplicationCommandInteraction) -> Result<(), serenity::Error> {
	c.create_interaction_response(http, |r| {
		r.kind(InteractionResponseType::ChannelMessageWithSource)
		 .interaction_response_data(|m| 
				m.content(format!("Server ID: {}", c.guild_id.unwrap())).ephemeral(true))
	}).await
}

pub fn roll() {

}

pub async fn join(ctx: Context, c: ApplicationCommandInteraction) -> Result<(), serenity::Error> {
	let resp: String;
	let guild: Guild = ctx.cache.guild(c.guild_id.expect("Error getting GuildId!"))
		.expect("Can't get the guild. Did you forget to set the GUILD intents in Client::builder?");

	let channel: Option<ChannelId>;

	// If a channel is specifically provided, use it. Else if the user is in a voice channel, use that.
	// Else, send out an error as channel is not specified!
	if let Some(id) = c.data.options.get(0) {
		if let CommandDataOptionValue::Channel(x) = id.resolved.as_ref().unwrap() {
			channel = Some(x.id);
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

	if let Some(channel) = channel {
		// Connect with songbird.
		let _handler = songbird::get(&ctx).await.expect("Songbird not registered")
			.join(c.guild_id.unwrap(), channel).await;
	}
	
	c.create_interaction_response(&ctx.http, |r| {
		r.kind(InteractionResponseType::ChannelMessageWithSource)
		 .interaction_response_data(|m| m.content(resp))
	}).await
}

pub async fn leave(ctx: Context, c: ApplicationCommandInteraction) -> Result<(), serenity::Error> {
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

	c.create_interaction_response(ctx.http, |r| {
		r.kind(InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|m| m.content(resp))
	}).await
}


// ----------------------------------------------------------------------

pub async fn unimplemented(http:Arc<Http>, c: ApplicationCommandInteraction) -> Result<(), serenity::Error> {
	c.create_interaction_response(http, |r| {
		r.kind(InteractionResponseType::ChannelMessageWithSource)
		 .interaction_response_data(|m| m.content("Not implemented yet! :(".to_string()))
	})
	.await
}

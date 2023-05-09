use std::sync::Arc;

use serenity::{model::prelude::interaction::{application_command::ApplicationCommandInteraction, InteractionResponseType}, http::Http};

pub async fn ping(http:Arc<Http>, c: ApplicationCommandInteraction) -> Result<(), serenity::Error>  {
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

pub async fn join(http:Arc<Http>, c: ApplicationCommandInteraction) -> Result<(), serenity::Error> {
	c.create_interaction_response(http, |r| {
		r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
		 .interaction_response_data(|m| m.content("Pong! :ping_pong:".to_string()))
	}).await
}

pub async fn unimplemented(http:Arc<Http>, c: ApplicationCommandInteraction) -> Result<(), serenity::Error> {
	Ok(c.create_interaction_response(http, |r| {
			r.kind(InteractionResponseType::ChannelMessageWithSource)
			 .interaction_response_data(|m| m.content("Not implemented yet! :(".to_string()))
		})
		.await?)
}

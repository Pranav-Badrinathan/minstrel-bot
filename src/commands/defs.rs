use serenity::{
	all::CommandOptionType, 
	builder::{ CreateCommand, CreateCommandOption },
	model::channel::ChannelType
};

pub fn ping() -> CreateCommand {
	CreateCommand::new("ping").description("Pings Minstrel. A test command.")
}

pub fn id() -> CreateCommand {
	CreateCommand::new("id").description("Get's this Discord Server's ID.")
}

pub fn roll() -> CreateCommand {
	CreateCommand::new("roll").description("Rolls dice based on input string.")
		.add_option(
			CreateCommandOption::new(
				CommandOptionType::String, 
				"dice", 
				"eg: 1d20, 2d20kh, 5d6kl3"
			).required(true)
		)
}

pub fn join() -> CreateCommand {
	CreateCommand::new("join").description("The bot joins the voice channel you are currently in.")
		.add_option(
			CreateCommandOption::new(
				CommandOptionType::Channel, 
				"channel", 
				"Leave blank for Minstrel to join the voice channel you are currently in"
		).channel_types(vec![ChannelType::Voice])
	)
}

pub fn leave() -> CreateCommand {
	CreateCommand::new("leave").description("Minstrel leaves the voice channel if it is in one.")
}

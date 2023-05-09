use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};

pub fn ping(c: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	c.name("ping").description("Pings Minstrel. A test command.")
}

pub fn id(c: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	c.name("id").description("Get's this Discord Server's ID.")
}

pub fn roll(c: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	c.name("roll").description("Rolls dice based on input string.")
		.create_option(|o| {
			o.name("dice").description("eg: 1d20, 2d20kh, 5d6k3l")
				.kind(CommandOptionType::String).required(true)
		})
}

pub fn join(c: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	c.name("join").description("The bot joins the voice channel you are currently in.")
}

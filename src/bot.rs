
pub async fn bot_init() {
	let token = std::env::var("BOT_TOKEN").expect("Token not found in environment!!!");
	let mut client = Client::builder(token, GatewayIntents::empty())
		.event_handler(Handler).await
		.expect("Error creating the Client.");

	if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
	else { println!("connected!!"); }
}

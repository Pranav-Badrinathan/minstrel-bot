use tokio::sync::mpsc;


pub async fn server_init(mut rcv: mpsc::Receiver<crate::State>) {

	// Run both the server and the shutdown reciever in parallel. When either the server errors
	// or the shutdown flag is recieved, gracefully shutdown.
	tokio::select! {
		// start = client.start() => {
		// 	if let Err(why) = start {
		//         		println!("Client error: {:?}", why);
		// 	}
		// },
		flag = rcv.recv() => {
			if let Some(state) = flag {
				match state {
					crate::State::Shutdown => { /* TODO: Server Shutdown*/ },
					crate::State::Restart => {},
				}
			}
		},
	}
}


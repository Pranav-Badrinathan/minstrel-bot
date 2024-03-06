use tokio::sync::mpsc;
use lazy_static::lazy_static;

use symphonia::core::io::ReadOnlySource;
use songbird::{
	input::{
		codecs::{CODEC_REGISTRY, PROBE},
		AudioStream,
		Input,
		LiveInput
	},
	constants::SILENT_FRAME, 
};

use crate::bot::SONG;

lazy_static! {
	static ref DCA1_HEADER: Vec<u8> = [
		b"DCA1".to_vec(),
		vec![224, 0, 0, 0],
		br#"{"dca":{"version":1,"tool":{"name":"opus-rs","version":"1.0.0","url":null,"author":null}},"opus":{"mode":"voip","sample_rate":48000,"frame_size":960,"abr":null,"vbr":true,"channels":2},"info":null,"origin":null,"extra":null}"#.to_vec()
	].concat();
}

pub struct OpusStream {
	// 20 ms opus frame Receiver.
	pub rx: mpsc::Receiver<Vec<u8>>,
	pub current_frame: Option<Vec<u8>>,
	pub chunk_pos: usize,
	pub pos: usize,
	pub guild_id: std::num::NonZeroU64
}

impl std::io::Read for OpusStream {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		// If the position in the stream is under header len, headers have not
		// been passed in yet. Pass them in.
		//
		// ELSE if there is not frame selected, get one.
		if self.pos < DCA1_HEADER.len() {
			let size = DCA1_HEADER.as_slice().read(buf)?;
			self.pos = size;
			self.chunk_pos = 0;
			return Ok(size);

		} else if self.current_frame.is_none() {
			match self.rx.try_recv() {
				Ok(fr) => {
					self.current_frame = Some(fr);
				},
				Err(mpsc::error::TryRecvError::Empty) => {
					self.current_frame = Some(SILENT_FRAME.to_vec());
				},
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
			let size = (frame.len() as i16).to_le_bytes().as_slice().read(buf)?;
			self.chunk_pos = size; // Size will always be 2, cause 16 bit.
			self.pos += size;
			Ok(size)
			
		} else {
			let part_fr_offset = self.chunk_pos - 2;
			let size = frame[part_fr_offset..].as_ref().read(buf)?;
			self.chunk_pos += size;
			self.pos += size;

			if self.chunk_pos >= (frame.len() + 1) {
				self.current_frame = None;
				self.chunk_pos = 0;
			}

			Ok(size)
		}
	}
}

pub async fn init_audio_stream(stream: OpusStream) {
	let sb = SONG.get().expect("Songbird not found!").clone();
	
	if let Some(h) = sb.get(stream.guild_id) {
		let src: ReadOnlySource<OpusStream> = ReadOnlySource::new(stream);

		let live = LiveInput::Raw(AudioStream {
			input:Box::new(src),
			hint:None
		}).promote(&CODEC_REGISTRY, &PROBE).unwrap();

		let inp = Input::Live(live, None);

		let mut handler = h.lock().await;
		handler.play_input(inp);
	}
}

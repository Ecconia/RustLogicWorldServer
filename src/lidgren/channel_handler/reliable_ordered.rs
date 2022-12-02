use crate::lidgren::data_structures::MessageHeader;
use crate::util::custom_iterator::CustomIterator;

const WINDOW_SIZE: usize = 64;

struct InternalMessage {
	header: MessageHeader,
	data: Vec<u8>,
}

pub struct ReliableOrderedHandler {
	latest_sequence_index: u16,
	cycle_buffer: [Option<InternalMessage>; WINDOW_SIZE],
}

impl ReliableOrderedHandler {
	pub fn new() -> ReliableOrderedHandler {
		const DEFAULT: Option<InternalMessage> = None; //Workaround, for the Rust compiler...
		ReliableOrderedHandler {
			latest_sequence_index: 0,
			cycle_buffer: [DEFAULT; WINDOW_SIZE],
		}
	}
	
	pub fn handle(
		&mut self,
		header: MessageHeader,
		mut message_data_iterator: CustomIterator,
		output_list: &mut Vec<(MessageHeader, Vec<u8>)>,
	) {
		let relative_sequence_number = (header.sequence_number as i16 - self.latest_sequence_index as i16 + 1024 + 512) % 1024 - 512;
		
		if relative_sequence_number < 0 {
			//TODO: Acknowledge
			
			//Received message is older, than what is already processed, so lets just discard it.
			return;
		}
		
		let data = message_data_iterator.read_bytes(message_data_iterator.remaining()).unwrap();
		
		if relative_sequence_number == 0 {
			//TODO: Acknowledge
			
			//Assert: self.cycle_buffer[self.latest_sequence_index as usize % WINDOW_SIZE] is equal to None!
			
			//We had been waiting for you, enter!
			output_list.push((header, data));
			self.latest_sequence_index += 1;
			
			while let Some(buffered_message) = self.cycle_buffer[self.latest_sequence_index as usize % WINDOW_SIZE].take() {
				output_list.push((buffered_message.header, buffered_message.data));
				self.latest_sequence_index += 1;
			}
			
			return;
		}
		
		if relative_sequence_number > 64 {
			//If a message with a relative index above 63 is received, the packet is either corrupted,
			// or the client is corrupted or malicious.
			//There however is no room to store it. So one has to ignore it, until it is its turn to be received.
			//The only way to gracefully handle this, is by not sending the acknowledge packet and receive
			// this packet until it is relevant to be received.
			//The only alternative is to acknowledge it anyway, and wait for this connection to deadlock and time out the remote side...
			println!("Major issue, received message way too early, it won't fit the buffer. This connection is ruined!");
			return;
		}
		
		//TODO: Acknowledge
		
		//else - Message newer than expected: Just store it!
		
		let index = header.sequence_number as usize % WINDOW_SIZE;
		if let Some(old_message) = &self.cycle_buffer[index] {
			println!("UNEXPECTED UNUSED MESSAGE ON CYCLE BUFFER! {:x?}", old_message.header);
		}
		self.cycle_buffer[index] = Some(InternalMessage {
			header,
			data,
		});
	}
}

use crate::prelude::*;

use crate::lidgren::message_type::MessageType;
use crate::util::custom_iterator::CustomIterator;

pub const MESSAGE_HEADER_LENGTH: usize = 5;

#[derive(Debug)]
pub struct MessageHeader {
	//TBI: Eventually make them private and use getters and constructors?
	pub message_type: MessageType,
	pub fragment: bool,
	pub sequence_number: u16,
	pub bits: u16,
	pub bytes: u16,
}

impl MessageHeader {
	pub fn from_stream(iterator: &mut CustomIterator) -> EhResult<MessageHeader> {
		if iterator.remaining() < MESSAGE_HEADER_LENGTH {
			return exception!("Not enough bytes to read Lidgren header: ", iterator.remaining(), "/", MESSAGE_HEADER_LENGTH);
		}
		
		let message_type_id = iterator.next_unchecked();
		let fragment = (iterator.peek_unchecked() & 1) == 1;
		let sequence_number = (iterator.next_unchecked() as u16 >> 1) | ((iterator.next_unchecked() as u16) << 7);
		let bits = iterator.next_unchecked() as u16 | ((iterator.next_unchecked() as u16) << 8);
		
		let message_type = unwrap_some_or_return!(MessageType::from_id(message_type_id),{
			exception!("There is no message type for id: ", message_type_id).wrap(ex!("While reading Lidgren header"))
		});
		
		//Make sure to not overflow:
		let bytes = if bits >= (0xFFFF - 8) { 0xFFFF / 8 } else { (bits + 7) / 8 };
		
		Ok(MessageHeader {
			message_type,
			fragment,
			sequence_number,
			bits,
			bytes,
		})
	}
}

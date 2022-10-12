use crate::custom_unwrap_option_or_else;

use crate::lidgren::message_type::MessageType;
use crate::util::custom_iterator::CustomIterator;

pub const MESSAGE_HEADER_LENGTH: usize = 5;

pub struct MessageHeader
{
	//TBI: Eventually make them private and use getters and constructors?
	pub message_type: MessageType,
	pub fragment: bool,
	pub sequence_number: u16,
	pub bits: u16,
	pub bytes: u16,
}

impl MessageHeader
{
	pub fn from_stream(iterator: &mut CustomIterator) -> Result<MessageHeader, String>
	{
		if iterator.remaining() < MESSAGE_HEADER_LENGTH {
			return Err(format!("Not enough bytes to read the header! Only got {}/{}", iterator.remaining(), MESSAGE_HEADER_LENGTH));
		}
		
		let message_type_id = iterator.next_unchecked();
		let fragment = (iterator.peek_unchecked() & 1) == 1;
		let sequence_number = (iterator.next_unchecked() as u16 >> 1) | ((iterator.next_unchecked() as u16) << 7);
		let bits = iterator.next_unchecked() as u16 | ((iterator.next_unchecked() as u16) << 8);
		
		let message_type = custom_unwrap_option_or_else!(MessageType::from_id(message_type_id),{
			return Err(format!("Could not find message type for id {}!", message_type_id));
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

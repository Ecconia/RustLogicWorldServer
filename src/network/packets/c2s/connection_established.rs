use crate::prelude::*;

use crate::network::message_pack::reader as mp_reader;
use crate::network::packets::packet_ids::PacketIDs;
use crate::util::custom_iterator::CustomIterator;

pub struct ConnectionEstablished {
}

impl ConnectionEstablished {
	pub fn parse(iterator: &mut CustomIterator) -> EhResult<ConnectionEstablished> {
		let packet_id = exception_wrap!(mp_reader::read_u32(iterator), "While reading ConnectionEstablished packet's id")?;
		if packet_id != PacketIDs::ConnectionEstablished.id() {
			return exception!("ConnectionEstablished packet has wrong packet id: ", packet_id);
		}
		let number = exception_wrap!(mp_reader::read_array(iterator), "While parsing ConnectionEstablished packet's entry count")?;
		if number != 1 {
			return exception!("Expected ConnectionEstablished packet to have one element as array, got: ", number);
		}
		let number = exception_wrap!(mp_reader::read_u32(iterator), "While parsing ConnectionEstablished packet's dummy value")?;
		if number != 0 {
			return exception!("Expected ConnectionEstablished expected integer of value 0, got: ", number);
		}
		
		if iterator.has_more() {
			log_warn!("ConnectionEstablished packet has more bytes than expected, ", iterator.remaining(), " remaining bytes.");
		}
		
		Ok(ConnectionEstablished {})
	}
}

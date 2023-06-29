use crate::prelude::*;
use crate::network::packets::packet_tools::*;

use crate::network::message_pack::reader as mp_reader;
use crate::network::packets::packet_ids::PacketIDs;
use crate::util::custom_iterator::CustomIterator;

pub struct ConnectionEstablished {
}

impl ConnectionEstablished {
	pub fn validate_packet_id(iterator: &mut CustomIterator) -> EhResult<()>{
		expect_packet_id!(iterator, "connection established", PacketIDs::ConnectionEstablished);
		Ok(())
	}
	
	pub fn parse(mut iterator: CustomIterator) -> EhResult<ConnectionEstablished> {
		let iterator = &mut iterator;
		expect_array!(iterator, "ConnectionEstablished", "main content", 1);
		let number = exception_wrap!(mp_reader::read_u32(iterator), "While parsing ConnectionEstablished packet's dummy value")?;
		if number != 0 {
			return exception!("Expected ConnectionEstablished expected integer of value 0, got: ", number);
		}
		
		expect_end_of_packet!(iterator, "ConnectionEstablished");
		
		Ok(ConnectionEstablished {})
	}
}

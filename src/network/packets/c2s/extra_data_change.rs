use crate::prelude::*;
use crate::network::packets::packet_tools::*;

use crate::network::message_pack::reader as mp_reader;
use crate::network::packets::packet_ids::PacketIDs;
use crate::util::custom_iterator::CustomIterator;

pub struct ExtraDataChange {
	pub key: String,
	pub data_type: String,
	pub data_bytes: Vec<u8>,
}

impl ExtraDataChange {
	pub fn validate_packet_id(iterator: &mut CustomIterator) -> EhResult<()>{
		expect_packet_id!(iterator, "extra data change", PacketIDs::ExtraDataChange);
		Ok(())
	}
	
	pub fn parse(mut iterator: CustomIterator) -> EhResult<Self> {
		let iterator = &mut iterator;
		expect_array!(iterator, "extra data change", "main content", 3);
		
		let extra_data_key = mp_reader::read_string(iterator).wrap(ex!("While reading ExtraDataChangePacket extra data key"))?;
		log_debug!("Client is attempting to change ", extra_data_key, " extra data");
		
		let extra_data_type = mp_reader::read_string(iterator).wrap(ex!("While reading ExtraDataChangePacket extra data type"))?;
		
		let extra_data_default = mp_reader::read_bytes(iterator).wrap(ex!("While reading ExtraDataChangePacket new extra data bytes"))?;
		
		Ok(Self {
			key: extra_data_key,
			data_type: extra_data_type,
			data_bytes: extra_data_default,
		})
	}
}

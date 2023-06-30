use crate::prelude::*;
use crate::network::packets::packet_tools::*;

use crate::network::message_pack::reader as mp_reader;
use crate::network::packets::packet_ids::PacketIDs;
use crate::util::custom_iterator::CustomIterator;

pub struct ExtraDataRequest {
	pub key: String,
	pub data_type: String,
	pub default: Vec<u8>,
}

impl ExtraDataRequest {
	pub fn validate_packet_id(iterator: &mut CustomIterator) -> EhResult<()>{
		expect_packet_id!(iterator, "extra data request", PacketIDs::ExtraDataRequest);
		Ok(())
	}
	
	pub fn parse(mut iterator: CustomIterator) -> EhResult<ExtraDataRequest> {
		let iterator = &mut iterator;
		expect_array!(iterator, "extra data request", "main content", 3);
		
		let extra_data_key = exception_wrap!(mp_reader::read_string(iterator), "While reading ExtraDataRequestPacket extra data key")?;
		log_debug!("Client is requesting ", extra_data_key, " extra data");
		
		let extra_data_type = exception_wrap!(mp_reader::read_string(iterator), "While reading ExtraDataRequestPacket extra data type")?;
		
		let extra_data_default = exception_wrap!(mp_reader::read_bytes(iterator), "While reading ExtraDataRequestPacket default extra data bytes")?;
		
		Ok(ExtraDataRequest {
			key: extra_data_key,
			data_type: extra_data_type,
			default: extra_data_default,
		})
	}
}

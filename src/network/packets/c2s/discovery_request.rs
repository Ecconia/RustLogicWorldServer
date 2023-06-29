use crate::prelude::*;
use crate::network::packets::packet_tools::*;

use crate::network::message_pack::reader as mp_reader;
use crate::network::packets::packet_ids::PacketIDs;
use crate::util::custom_iterator::CustomIterator;

pub struct DiscoveryRequest {
	pub intention_to_connect: bool,
	pub request_uid: String,
}

impl DiscoveryRequest {
	pub fn validate_packet_id(iterator: &mut CustomIterator) -> EhResult<()>{
		expect_packet_id!(iterator, "discovery request", PacketIDs::DiscoveryRequest);
		Ok(())
	}
	
	pub fn parse(mut iterator: CustomIterator) -> EhResult<DiscoveryRequest> {
		let iterator = &mut iterator;
		let map_size = exception_wrap!(mp_reader::read_map(iterator), "While reading discovery packet entry map count")?;
		if map_size != 2 {
			return exception!("Discovery packet has wrong map size: ", map_size, " (should be ", 2, ")");
		}
		//Intention to connect:
		let key = exception_wrap!(mp_reader::read_string(iterator), "While reading discovery packet key 'ForConnection'")?;
		if String::from("ForConnection").ne(&key) {
			return exception!("Discovery packet has wrong first map key: ", key, " (should be ", "ForConnection", ")");
		}
		
		let intention_to_connect = exception_wrap!(mp_reader::read_bool(iterator), "While reading discovery packet bool 'intention to connect'")?;
		log_debug!("Wants to connect: ", intention_to_connect);
		
		let key = exception_wrap!(mp_reader::read_string(iterator), "While reading discovery packet key 'RequestGUID'")?;
		if String::from("RequestGUID").ne(&key) {
			return exception!("Discovery packet has wrong second map key: ", key, " (should be ", "RequestGUID", ")");
		}
		
		let request_uid = exception_wrap!(mp_reader::read_string(iterator), "While reading discovery packet GUID string")?;
		log_debug!("Request UUID is: ", request_uid);
		
		expect_end_of_packet!(iterator, "DiscoveryRequest");
		
		Ok(DiscoveryRequest {
			request_uid,
			intention_to_connect,
		})
	}
}

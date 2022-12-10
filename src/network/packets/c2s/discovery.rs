use crate::prelude::*;

use crate::network::message_pack::reader as mp_reader;
use crate::util::custom_iterator::CustomIterator;

pub struct Discovery {
	pub intention_to_connect: bool,
	pub request_uid: String,
}

impl Discovery {
	pub fn parse(iterator: &mut CustomIterator) -> EhResult<Discovery> {
		let packet_id = exception_wrap!(mp_reader::read_int_auto(iterator), "While reading discovery packet id")?;
		if packet_id != 10 {
			return exception!("Discovery packet has wrong packet id: ", packet_id);
		}
		let map_size = exception_wrap!(mp_reader::read_map_auto(iterator), "While reading discovery packet entry map count")?;
		if map_size != 2 {
			return exception!("Discovery packet has wrong map size: ", map_size, " (should be ", 2, ")");
		}
		//Intention to connect:
		let key = custom_unwrap_option_or_else!(exception_wrap!(mp_reader::read_string_auto(iterator), "While reading discovery packet key 'ForConnection'")?, {
			return exception!("Discovery packet has first map key not set");
		});
		if String::from("ForConnection").ne(&key) {
			return exception!("Discovery packet has wrong first map key: ", key, " (should be ", "ForConnection", ")");
		}
		
		let intention_to_connect = exception_wrap!(mp_reader::read_bool_auto(iterator), "While reading discovery packet bool 'intention to connect'")?;
		log_debug!("Wants to connect: ", intention_to_connect);
		
		let key = custom_unwrap_option_or_else!(exception_wrap!(mp_reader::read_string_auto(iterator), "While reading discovery packet key 'RequestGUID'")?, {
			return exception!("Discovery packet has second map key not set");
		});
		if String::from("RequestGUID").ne(&key) {
			return exception!("Discovery packet has wrong second map key: ", key, " (should be ", "RequestGUID", ")");
		}
		
		let request_uid = custom_unwrap_option_or_else!(exception_wrap!(mp_reader::read_string_auto(iterator), "While reading discovery packet GUID string")?, {
			return exception!("Discovery packet has second value not set");
		});
		log_debug!("Request UUID is: ", request_uid);
		
		Ok(Discovery {
			request_uid,
			intention_to_connect,
		})
	}
}

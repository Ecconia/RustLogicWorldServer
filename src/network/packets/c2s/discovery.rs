use crate::error_handling::custom_unwrap_option_or_else;

use crate::network::message_pack::reader as mp_reader;
use crate::util::custom_iterator::CustomIterator;

pub struct Discovery
{
	pub intention_to_connect: bool,
	pub request_uid: String,
}

impl Discovery
{
	pub fn parse(iterator: &mut CustomIterator) -> Result<Discovery, String>
	{
		let packet_id = mp_reader::read_int_auto(iterator);
		if packet_id != 10
		{
			return Err(format!("Discovery had wrong data packet ID type: {}", packet_id));
		}
		let map_size = mp_reader::read_map_auto(iterator);
		if map_size != 2
		{
			return Err(format!("While parsing discovery packet, expected map of size 2, but got {}", map_size));
		}
		//Intention to connect:
		let key = custom_unwrap_option_or_else!(mp_reader::read_string_auto(iterator), {
			return Err(format!("While parsing discovery packet, expected first map key to be present, but got null."));
		});
		if String::from("ForConnection").ne(&key)
		{
			return Err(format!("While parsing discovery packet, expected first map key to be 'ForConnection', but got '{}'.", key));
		}
		
		let intention_to_connect = mp_reader::read_bool_auto(iterator);
		println!("Wants to connect: \x1b[38;2;255;0;150m{}\x1b[m", intention_to_connect);
		
		let key = custom_unwrap_option_or_else!(mp_reader::read_string_auto(iterator), {
			return Err(format!("While parsing discovery packet, expected first map key to be present, but got null."));
		});
		if String::from("RequestGUID").ne(&key)
		{
			return Err(format!("While parsing discovery packet, expected first map key to be 'RequestGUID', but got '{}'.", key));
		}
		
		let request_uid = custom_unwrap_option_or_else!(mp_reader::read_string_auto(iterator), {
			return Err(format!("While parsing discovery packet, expected second value to be a string, but got null."));
		});
		println!("Request UUID is: \x1b[38;2;255;0;150m{}\x1b[m", request_uid);
		
		Ok(Discovery {
			request_uid,
			intention_to_connect,
		})
	}
}

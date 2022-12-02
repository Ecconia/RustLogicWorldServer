use crate::custom_unwrap_option_or_else;

use crate::network::message_pack::reader as mp_reader;
use crate::util::custom_iterator::CustomIterator;

pub struct Connect
{
	pub username: String,
	pub mods: Vec<String>,
	pub version: String,
	//The official LW client always sends a hash (even if it hashed an empty string), this server allows 'null':
	pub password_hash: Option<Vec<u8>>,
	pub hail_payload: Option<String>,
	pub hail_signature: Option<String>,
}

impl Connect
{
	pub fn parse(iterator: &mut CustomIterator) -> Result<Connect, String>
	{
		let packet_id = mp_reader::read_int_auto(iterator);
		if packet_id != 16
		{
			return Err(format!("Discovery packet not from a 0.91 client, but {}, bye!", packet_id));
		}
		
		let entry_count = mp_reader::read_array_auto(iterator);
		if entry_count != 6
		{
			return Err(format!("Client Connect packet has different entry count than 6, got: {}", entry_count));
		}
		
		let mod_count = mp_reader::read_array_auto(iterator);
		println!("Mod count: {}", mod_count);
		let mut mods = Vec::new();
		for _ in 0..mod_count
		{
			let mod_id = custom_unwrap_option_or_else!(mp_reader::read_string_auto(iterator), {
				return Err(format!("Received null mod name, illegal!"));
			});
			println!(" - {}", mod_id);
			mods.push(mod_id);
		}
		
		let user_option_count = mp_reader::read_array_auto(iterator);
		if user_option_count != 1
		{
			return Err(format!("More than one user argument, got: {}", user_option_count));
		}
		let username = custom_unwrap_option_or_else!(mp_reader::read_string_auto(iterator), {
			return Err(format!("Received null username, illegal!"));
		});
		println!("Username: {}", username);
		
		let version = custom_unwrap_option_or_else!(mp_reader::read_string_auto(iterator), {
			return Err(format!("Received null version, unsupported!"));
		});
		let password_hash = mp_reader::read_binary_auto(iterator);
		let hail_payload = mp_reader::read_string_auto(iterator);
		let hail_signature = mp_reader::read_string_auto(iterator);
		println!("Version: {}", version);
		println!("PWHash: {:x?}", password_hash);
		println!("HailPayload: {:?}", hail_payload);
		println!("HailSignature: {:?}", hail_signature);
		
		return Ok(Connect {
			username,
			mods,
			version,
			password_hash,
			hail_payload,
			hail_signature,
		});
	}
}

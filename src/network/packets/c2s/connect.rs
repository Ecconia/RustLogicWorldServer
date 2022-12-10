use crate::error_handling::{custom_unwrap_option_or_else, EhResult, exception_wrap, exception};

use crate::network::message_pack::reader as mp_reader;
use crate::util::custom_iterator::CustomIterator;

pub struct Connect {
	pub username: String,
	pub mods: Vec<String>,
	pub version: String,
	//The official LW client always sends a hash (even if it hashed an empty string), this server allows 'null':
	pub password_hash: Option<Vec<u8>>,
	pub hail_payload: Option<String>,
	pub hail_signature: Option<String>,
}

impl Connect {
	pub fn parse(iterator: &mut CustomIterator) -> EhResult<Connect> {
		let packet_id = exception_wrap!(mp_reader::read_int_auto(iterator), "While reading connect packet id")?;
		if packet_id != 16 {
			return exception!("Connect packet packet has wrong packet id: ", packet_id);
		}
		
		let entry_count = exception_wrap!(mp_reader::read_array_auto(iterator), "While reading connect packet entry count")?;
		if entry_count != 6 {
			return exception!("Connect packet has wrong entry count: ", entry_count, " (should be ", 6, ")");
		}
		
		let mod_count = exception_wrap!(mp_reader::read_array_auto(iterator), "While reading connect packet mod count")?;
		println!("Mod count: {}", mod_count);
		let mut mods = Vec::new();
		for _ in 0..mod_count {
			let mod_id = custom_unwrap_option_or_else!(exception_wrap!(mp_reader::read_string_auto(iterator), "While reading connect packet mod entry")?, {
				return exception!("Connect packet has a mod name not set");
			});
			println!(" - {}", mod_id);
			mods.push(mod_id);
		}
		
		let user_option_count = exception_wrap!(mp_reader::read_array_auto(iterator), "While reading connect packet user option count")?;
		if user_option_count != 1 {
			return exception!("Connect packet has wrong user option count: ", user_option_count, " (should be ", 1, ")");
		}
		let username = custom_unwrap_option_or_else!(exception_wrap!(mp_reader::read_string_auto(iterator), "While reading connect packet username")?, {
			return exception!("Connect packet has username not set");
		});
		println!("Username: {}", username);
		
		let version = custom_unwrap_option_or_else!(exception_wrap!(mp_reader::read_string_auto(iterator), "While reading connect packet client version")?, {
			return exception!("Connect packet has version not set");
		});
		let password_hash = exception_wrap!(mp_reader::read_binary_auto(iterator), "While reading connect packet password hash")?;
		let hail_payload = exception_wrap!(mp_reader::read_string_auto(iterator), "While reading connect packet hail payload")?;
		let hail_signature = exception_wrap!(mp_reader::read_string_auto(iterator), "While reading connect packet hail signature")?;
		println!("Version: {}", version);
		println!("PWHash: {:x?}", password_hash);
		println!("HailPayload: {:?}", hail_payload);
		println!("HailSignature: {:?}", hail_signature);
		
		Ok(Connect {
			username,
			mods,
			version,
			password_hash,
			hail_payload,
			hail_signature,
		})
	}
}

use crate::prelude::*;

use crate::network::message_pack::reader as mp_reader;
use crate::network::packets::packet_ids::PacketIDs;
use crate::util::custom_iterator::CustomIterator;

pub struct ConnectionApproval {
	pub username: String,
	pub mods: Vec<String>,
	pub version: String,
	//The official LW client always sends a hash (even if it hashed an empty string), this server allows 'null':
	pub password_hash: Option<Vec<u8>>,
	pub hail_payload: Option<String>,
	pub hail_signature: Option<String>,
}

impl ConnectionApproval {
	pub fn parse(iterator: &mut CustomIterator) -> EhResult<ConnectionApproval> {
		let packet_id = exception_wrap!(mp_reader::read_u32(iterator), "While reading connect packet id")?;
		if packet_id != PacketIDs::ConnectionApproval.id() {
			return exception!("Connect packet packet has wrong packet id: ", packet_id);
		}
		
		let entry_count = exception_wrap!(mp_reader::read_array(iterator), "While reading connect packet entry count")?;
		if entry_count != 6 {
			return exception!("Connect packet has wrong entry count: ", entry_count, " (should be ", 6, ")");
		}
		
		let mod_count = exception_wrap!(mp_reader::read_array(iterator), "While reading connect packet mod count")?;
		log_debug!("Mod count: ", mod_count);
		let mut mods = Vec::new();
		for _ in 0..mod_count {
			let mod_id = exception_wrap!(mp_reader::read_string(iterator), "While reading connect packet mod entry")?;
			log_debug!(" - ", mod_id);
			mods.push(mod_id);
		}
		
		let user_option_count = exception_wrap!(mp_reader::read_array(iterator), "While reading connect packet user option count")?;
		if user_option_count != 1 {
			return exception!("Connect packet has wrong user option count: ", user_option_count, " (should be ", 1, ")");
		}
		let username = exception_wrap!(mp_reader::read_string(iterator), "While reading connect packet username")?;
		log_debug!("Username: ", username);
		
		let version = exception_wrap!(mp_reader::read_string(iterator), "While reading connect packet client version")?;
		let password_hash = exception_wrap!(mp_reader::optional!(iterator, mp_reader::read_bytes(iterator)), "While reading connect packet password hash")?;
		let hail_payload = exception_wrap!(mp_reader::optional!(iterator, mp_reader::read_string(iterator)), "While reading connect packet hail payload")?;
		let hail_signature = exception_wrap!(mp_reader::optional!(iterator, mp_reader::read_string(iterator)), "While reading connect packet hail signature")?;
		log_debug!("Version: ", version);
		log_debug!("PWHash: ", format!("{:x?}", password_hash));
		log_debug!("HailPayload: ", format!("{:?}", hail_payload));
		log_debug!("HailSignature: ", format!("{:?}", hail_signature));
		
		if iterator.has_more() {
			log_warn!("ConnectionApproval packet has more bytes than expected, ", iterator.remaining(), " remaining bytes.");
		}
		
		Ok(ConnectionApproval {
			username,
			mods,
			version,
			password_hash,
			hail_payload,
			hail_signature,
		})
	}
}

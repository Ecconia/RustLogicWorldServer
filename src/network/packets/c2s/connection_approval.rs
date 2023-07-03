use crate::prelude::*;
use crate::network::packets::packet_tools::*;

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
	pub fn validate_packet_id(iterator: &mut CustomIterator) -> EhResult<()>{
		expect_packet_id!(iterator, "connection approval", PacketIDs::ConnectionApproval);
		Ok(())
	}
	
	pub fn parse(mut iterator: CustomIterator) -> EhResult<ConnectionApproval> {
		let iterator = &mut iterator;
		
		expect_array!(iterator, "ConnectionApproval", "main content", 6);
		let mod_count = mp_reader::read_array(iterator).wrap(ex!("While reading connect packet mod count"))?;
		log_debug!("Mod count: ", mod_count);
		let mut mods = Vec::new();
		for _ in 0..mod_count {
			let mod_id = mp_reader::read_string(iterator).wrap(ex!("While reading connect packet mod entry"))?;
			log_debug!(" - ", mod_id);
			mods.push(mod_id);
		}
		
		expect_array!(iterator, "ConnectionApproval", "user option", 1);
		let username = mp_reader::read_string(iterator).wrap(ex!("While reading connect packet username"))?;
		log_debug!("Username: ", username);
		
		let version = mp_reader::read_string(iterator).wrap(ex!("While reading connect packet client version"))?;
		let password_hash = mp_reader::optional!(iterator, mp_reader::read_bytes(iterator)).wrap(ex!("While reading connect packet password hash"))?;
		let hail_payload = mp_reader::optional!(iterator, mp_reader::read_string(iterator)).wrap(ex!("While reading connect packet hail payload"))?;
		let hail_signature = mp_reader::optional!(iterator, mp_reader::read_string(iterator)).wrap(ex!("While reading connect packet hail signature"))?;
		log_debug!("Version: ", version);
		log_debug!("PWHash: ", format!("{:x?}", password_hash));
		log_debug!("HailPayload: ", format!("{:?}", hail_payload));
		log_debug!("HailSignature: ", format!("{:?}", hail_signature));
		
		expect_end_of_packet!(iterator, "ConnectionApproval");
		
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

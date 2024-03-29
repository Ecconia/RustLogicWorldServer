use crate::prelude::*;

use crate::files::extra_data::manager::GenericExtraData;
use crate::network::message_pack::reader as mp_reader;
use crate::network::packets::packet_tools::*;
use crate::util::custom_iterator::CustomIterator;
use crate::util::succ::succ_types::SuccType;

pub const KEY: &str = "MHG.FlagListOrder";
pub const TYPE: &str = "System.Collections.Generic.List`1[[LogicAPI.Data.ComponentAddress, LogicAPI, Version=1.0.0.0, Culture=neutral, PublicKeyToken=null]]";
pub const FILE_TYPE: &str = "System.Collections.Generic.List<LogicAPI.Data.ComponentAddress>";

#[derive(Default)]
pub struct FlagListOrder {
	pub flags: Vec<u32>,
}

fn parse_data(bytes: &[u8]) -> EhResult<FlagListOrder> {
	let iterator = &mut CustomIterator::borrow(bytes);
	let flag_count = mp_reader::read_array(iterator).wrap(ex!("While reading flag entry count in extra data"))?;
	let mut flags = Vec::with_capacity(flag_count as usize);
	for _ in 0..flag_count {
		expect_array!(iterator, "FlagList ExtraData" , "flag entry", 1);
		let flag_address = mp_reader::read_i32(iterator).wrap(ex!("While reading flag entry in extra data"))?;
		if flag_address < 0 {
			exception!("Flag address must not be negative!")?
		}
		flags.push(flag_address as u32);
	}
	Ok(FlagListOrder {
		flags,
	})
}

impl GenericExtraData for FlagListOrder {
	fn validate_default_bytes(&self, bytes: &[u8]) -> bool {
		let suggested_default = unwrap_or_else_return!(parse_data(bytes), |error| {
			log_warn!("Client sent invalid default extra data:");
			error.print(); //TODO: Format as warning.
			false
		});
		if !suggested_default.flags.is_empty() {
			log_warn!("Client queried extra data ", KEY, ", but its default value is different from the servers default:\n\
				> The client claims that there are flags by default - this is a malicious action as the client cannot know the servers flags yet and thus cause damage to vanilla servers.");
			return false;
		}
		true
	}
	
	fn update_bytes_if_valid(&mut self, bytes: &[u8]) -> bool {
		let new_data = unwrap_or_else_return!(parse_data(bytes), |error| {
			log_warn!("Client sent invalid new extra data:");
			error.print(); //TODO: Format as warning.
			false
		});
		//TODO: Queue listener updates.
		//TODO: Check that all flags actually exist! Else this server is vulnerable.
		self.flags = new_data.flags;
		log_info!("Client change flag list to ", format!("{:?}", self.flags));
		true
	}
	
	fn load_from_file(&mut self, data: &SuccType) -> EhResult<()> {
		let root = data.expect_list().wrap(ex!())?;
		let mut list = Vec::new();
		for entry in root {
			list.push(entry.expect_component_address().wrap(ex!("While parsing flag entry"))?);
		}
		self.flags = list;
		log_debug!("Loaded ExtraData ", "FlagListOrder", " from disk.");
		Ok(())
	}
	
	fn key(&self) -> String {
		KEY.to_string()
	}
	
	fn data_type_network(&self) -> &str {
		TYPE
	}
	
	fn data_type_file(&self) -> &str {
		FILE_TYPE
	}
	
	//TODO: Cache the serialized data until it is changed.
	fn serialize_data(&self) -> Vec<u8> {
		use crate::network::message_pack::writer;
		
		let mut buffer = Vec::new();
		writer::write_array_auto(&mut buffer, self.flags.len() as u32);
		for flag in self.flags.iter() {
			writer::write_array_auto(&mut buffer, 1);
			writer::write_int_auto(&mut buffer, *flag);
		}
		buffer
	}
}

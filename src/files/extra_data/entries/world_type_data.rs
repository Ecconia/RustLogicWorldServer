use crate::prelude::*;

use crate::files::extra_data::manager::GenericExtraData;
use crate::files::world_data::world_structs::Color24;
use crate::network::message_pack::reader as mp_reader;
use crate::network::packets::packet_tools::*;
use crate::util::custom_iterator::CustomIterator;
use crate::util::error_handling::ExceptionDetails;

pub const KEY: &str = "MHG.WorldTypeData";
pub const TYPE: &str = "LogicWorld.SharedCode.Data.GridlandsWorldData"; //Ehm yes

pub struct WorldTypeDataGridlands {
	color_a: Color24,
	color_b: Color24,
	side_a: u32,
	side_b: u32,
}

impl Default for WorldTypeDataGridlands {
	fn default() -> Self {
		Self {
			color_a: Color24 { r: 80, g: 0, b: 0 },
			color_b: Color24 { r: 0, g: 80, b: 0 },
			side_a: 16,
			side_b: 32,
		}
	}
}

fn parse_data(bytes: &[u8]) -> EhResult<WorldTypeDataGridlands> {
	let iterator = &mut CustomIterator::borrow(bytes);
	
	expect_array!(iterator, "WorldTypeDataGridlands ExtraData" , "main content", 4);
	expect_array!(iterator, "WorldTypeDataGridlands ExtraData" , "color a", 3);
	let color_a = Color24 {
		r: mp_reader::read_u8(iterator).wrap(ex!("While reading color R in extra data"))?,
		g: mp_reader::read_u8(iterator).wrap(ex!("While reading color G in extra data"))?,
		b: mp_reader::read_u8(iterator).wrap(ex!("While reading color B in extra data"))?,
	};
	expect_array!(iterator, "WorldTypeDataGridlands ExtraData" , "color b", 3);
	let color_b = Color24 {
		r: mp_reader::read_u8(iterator).wrap(ex!("While reading color R in extra data"))?,
		g: mp_reader::read_u8(iterator).wrap(ex!("While reading color G in extra data"))?,
		b: mp_reader::read_u8(iterator).wrap(ex!("While reading color B in extra data"))?,
	};
	let side_a = mp_reader::read_i32(iterator).wrap(ex!("While reading side A in extra data"))?;
	if side_a < 1 {
		exception!("Gridworld side A is smaller than 1, illegal: ", side_a)?
	}
	let side_b = mp_reader::read_i32(iterator).wrap(ex!("While reading side B in extra data"))?;
	if side_b < 1 {
		exception!("Gridworld side B is smaller than 1, illegal: ", side_b)?
	}
	Ok(WorldTypeDataGridlands {
		color_a,
		color_b,
		side_a: side_a as u32,
		side_b: side_b as u32,
	})
}

impl GenericExtraData for WorldTypeDataGridlands {
	fn validate_default_bytes(&self, bytes: &[u8]) -> bool {
		unwrap_or_return!(parse_data(bytes), |error: ExceptionDetails| {
			log_warn!("Client sent invalid default extra data:");
			error.print(); //TODO: Format as warning.
			false
		});
		//Anything is valid as default (if it can be parsed).
		true
	}
	
	fn update_bytes_if_valid(&mut self, bytes: &[u8]) -> bool {
		let new_data = unwrap_or_return!(parse_data(bytes), |error: ExceptionDetails| {
			log_warn!("Client sent invalid new extra data:");
			error.print(); //TODO: Format as warning.
			false
		});
		//TODO: Queue listener updates.
		//TODO: Check that all flags actually exist! Else this server is vulnerable.
		self.color_a = new_data.color_a;
		self.color_b = new_data.color_b;
		self.side_a = new_data.side_a;
		self.side_b = new_data.side_b;
		log_info!("Client change plot floor...");
		true
	}
	
	fn key(&self) -> String {
		KEY.to_string()
	}
	
	fn data_type_network(&self) -> &str {
		TYPE
	}
	
	fn data_type_file(&self) -> &str {
		TYPE
	}
	
	//TODO: Cache the serialized data until it is changed.
	fn serialize_data(&self) -> Vec<u8> {
		use crate::network::message_pack::writer;
		let mut buffer = Vec::new();
		
		writer::write_array_auto(&mut buffer, 4);
		writer::write_array_auto(&mut buffer, 3);
		writer::write_int_auto(&mut buffer, self.color_a.r as u32);
		writer::write_int_auto(&mut buffer, self.color_a.g as u32);
		writer::write_int_auto(&mut buffer, self.color_a.b as u32);
		writer::write_array_auto(&mut buffer,3);
		writer::write_int_auto(&mut buffer, self.color_b.r as u32);
		writer::write_int_auto(&mut buffer, self.color_b.g as u32);
		writer::write_int_auto(&mut buffer, self.color_b.b as u32);
		writer::write_int_auto(&mut buffer, self.side_a);
		writer::write_int_auto(&mut buffer, self.side_b);
		
		buffer
	}
}

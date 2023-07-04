use crate::prelude::*;

use crate::files::extra_data::manager::GenericExtraData;
use crate::files::world_data::world_structs::Color24;
use crate::network::message_pack::reader as mp_reader;
use crate::network::packets::packet_tools::*;
use crate::util::custom_iterator::CustomIterator;
use crate::util::succ::succ_types::SuccType;

pub const KEY: &str = "MHG.WorldTypeData";
pub const TYPE: &str = "LogicWorld.SharedCode.Data.GridlandsWorldData"; //Ehm yes

pub struct WorldTypeDataGridlands {
	color_a: Color24,
	color_b: Color24,
	side_x: u32,
	side_z: u32,
}

impl Default for WorldTypeDataGridlands {
	fn default() -> Self {
		Self {
			color_a: Color24 { r: 80, g: 0, b: 0 },
			color_b: Color24 { r: 0, g: 80, b: 0 },
			side_x: 16,
			side_z: 32,
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
	let side_x = mp_reader::read_i32(iterator).wrap(ex!("While reading side A in extra data"))?;
	if side_x < 1 {
		exception!("Gridworld side A is smaller than 1, illegal: ", side_x)?
	}
	let side_z = mp_reader::read_i32(iterator).wrap(ex!("While reading side B in extra data"))?;
	if side_z < 1 {
		exception!("Gridworld side B is smaller than 1, illegal: ", side_z)?
	}
	Ok(WorldTypeDataGridlands {
		color_a,
		color_b,
		side_x: side_x as u32,
		side_z: side_z as u32,
	})
}

impl GenericExtraData for WorldTypeDataGridlands {
	fn validate_default_bytes(&self, bytes: &[u8]) -> bool {
		unwrap_or_return!(parse_data(bytes), |error| {
			log_warn!("Client sent invalid default extra data:");
			error.print(); //TODO: Format as warning.
			false
		});
		//Anything is valid as default (if it can be parsed).
		true
	}
	
	fn update_bytes_if_valid(&mut self, bytes: &[u8]) -> bool {
		let new_data = unwrap_or_return!(parse_data(bytes), |error| {
			log_warn!("Client sent invalid new extra data:");
			error.print(); //TODO: Format as warning.
			false
		});
		//TODO: Queue listener updates.
		//TODO: Check that all flags actually exist! Else this server is vulnerable.
		self.color_a = new_data.color_a;
		self.color_b = new_data.color_b;
		self.side_x = new_data.side_x;
		self.side_z = new_data.side_z;
		log_info!("Client change plot floor...");
		true
	}
	
	fn load_from_file(&mut self, data: &SuccType) -> EhResult<()> {
		let root = data.expect_map().wrap(ex!())?;
		
		let color_a_entry = root.get("ColorA").map_ex(ex!("Could not find ", "ColorA", " in data"))?
			.expect_color().wrap(ex!("While parsing ", "ColorA"))?;
		let color_b_entry = root.get("ColorB").map_ex(ex!("Could not find ", "ColorB", " in data"))?
			.expect_color().wrap(ex!("While parsing ", "ColorB"))?;
		let side_a_entry = root.get("BigCellSizeX").map_ex(ex!("Could not find ", "BigCellSizeX", " in data"))?
			.expect_unsigned().wrap(ex!("While parsing ", "BigCellSizeX"))?;
		if side_a_entry <= 0 {
			exception!("Entry ", "BigCellSizeX", " must be bigger than ", "0")?;
		}
		let side_b_entry = root.get("BigCellSizeZ").map_ex(ex!("Could not find ", "BigCellSizeZ", " in data"))?
			.expect_unsigned().wrap(ex!("While parsing ", "BigCellSizeZ"))?;
		if side_b_entry <= 0 {
			exception!("Entry ", "BigCellSizeZ:", " must be bigger than ", "0")?;
		}
		self.color_a = color_a_entry;
		self.color_b = color_b_entry;
		self.side_x = side_a_entry;
		self.side_z = side_b_entry;
		log_debug!("Loaded ExtraData ", "WorldTypeData", " from disk.");
		Ok(())
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
		writer::write_int_auto(&mut buffer, self.side_x);
		writer::write_int_auto(&mut buffer, self.side_z);
		
		buffer
	}
}

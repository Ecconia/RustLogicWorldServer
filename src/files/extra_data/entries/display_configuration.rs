use crate::prelude::*;

use crate::files::extra_data::manager::GenericExtraData;
use crate::files::world_data::world_structs::Color24;
use crate::network::message_pack::reader as mp_reader;
use crate::network::packets::packet_tools::*;
use crate::util::custom_iterator::CustomIterator;

pub const TYPE: &str = "JimmysUnityUtilities.Color24[]";

#[derive(Default)]
pub struct DisplayConfiguration {
	peg_count: u32,
	configuration_index: u32,
	pub data: Option<DisplayConfigurationData>,
}

impl DisplayConfiguration {
	pub fn new(peg_count: u32, configuration_index: u32) -> Self {
		Self {
			peg_count,
			configuration_index,
			data: None,
		}
	}
}

#[derive(Default)]
pub struct DisplayConfigurationData {
	pub colors: Vec<Color24>,
}

fn parse_data(bytes: &[u8]) -> EhResult<DisplayConfigurationData> {
	let iterator = &mut CustomIterator::borrow(bytes);
	let color_count = mp_reader::read_array(iterator).wrap(ex!("While reading color count in display conf extra data"))?;
	let mut colors = Vec::with_capacity(color_count as usize);
	for _ in 0..color_count {
		expect_array!(iterator, "DisplayConfiguration ExtraData" , "color entry", 3);
		let c_red = mp_reader::read_u8(iterator).wrap(ex!("While reading color channel in display conf extra data"))?;
		let c_green = mp_reader::read_u8(iterator).wrap(ex!("While reading color channel in display conf extra data"))?;
		let c_blue = mp_reader::read_u8(iterator).wrap(ex!("While reading color channel in display conf extra data"))?;
		colors.push(Color24 {
			r: c_red,
			g: c_green,
			b: c_blue,
		});
	}
	if iterator.has_more() {
		exception!("There are unread bytes while reading display configuration extra data")?;
	}
	Ok(DisplayConfigurationData {
		colors,
	})
}

impl GenericExtraData for DisplayConfiguration {
	fn validate_default_bytes(&self, bytes: &[u8]) -> bool {
		let suggested_default = unwrap_or_return!(parse_data(bytes), |error| {
			log_warn!("Client sent invalid default extra data:");
			error.print(); //TODO: Format as warning.
			false
		});
		let expected_color_amount = 2u32.pow(self.peg_count);
		if suggested_default.colors.len() != expected_color_amount as usize {
			log_warn!("Client sent display configuration with wrong amount of colors: ", suggested_default.colors.len(), " / ", expected_color_amount);
			return false;
		}
		//TBI: Temporary hack, this ExtraData has no default data (yet), just apply what the client suggests:
		// This becomes an issue, as soon as the deletion of configurations is done by the client - black box.
		if self.data.is_none() {
			unsafe {
				let const_pointer = &self.data as *const Option<DisplayConfigurationData>;
				let mut_pointer = const_pointer as *mut Option<DisplayConfigurationData>;
				*mut_pointer = Some(suggested_default);
			 }
		}
		true
	}
	
	fn update_bytes_if_valid(&mut self, bytes: &[u8]) -> bool {
		let new_data = unwrap_or_return!(parse_data(bytes), |error| {
			log_warn!("Client sent invalid new extra data:");
			error.print(); //TODO: Format as warning.
			false
		});
		let expected_color_amount = 2u32.pow(self.peg_count);
		if new_data.colors.len() != expected_color_amount as usize {
			log_warn!("Client sent invalid new extra data: Amount of colors does not match peg count: ", new_data.colors.len(), " / ", expected_color_amount);
			return false;
		}
		self.data = Some(new_data);
		log_info!("Client change display configuration list #", self.configuration_index, " @", self.peg_count, " pegs.");
		true
	}
	
	fn key(&self) -> String {
		format!("MHG.DisplayConfigurations/{}_pegs/Configuration{}", self.peg_count, self.configuration_index)
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
		if self.data.is_none() {
			panic!("Ehm, so display configuration may not be serialized, when its scheduled for deletion - or basically not set!");
		}
		let data = self.data.as_ref().unwrap();
		writer::write_array_auto(&mut buffer, data.colors.len() as u32);
		for color in data.colors.iter() {
			writer::write_array_auto(&mut buffer, 3);
			writer::write_int_auto(&mut buffer, color.r as u32);
			writer::write_int_auto(&mut buffer, color.g as u32);
			writer::write_int_auto(&mut buffer, color.b as u32);
		}
		buffer
	}
}

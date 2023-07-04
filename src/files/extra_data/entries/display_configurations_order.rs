use crate::prelude::*;

use crate::files::extra_data::manager::GenericExtraData;
use crate::network::message_pack::reader as mp_reader;
use crate::util::custom_iterator::CustomIterator;
use crate::util::error_handling::ExceptionDetails;

pub const TYPE: &str = "System.Int32[]";

#[derive(Default)]
pub struct DisplayConfigurationsOrder {
	pub peg_count: u32,
	pub data: Option<DisplayConfigurationsOrderData>,
}

impl DisplayConfigurationsOrder {
	pub fn new(peg_count: u32) -> Self {
		Self {
			peg_count,
			data: None,
		}
	}
}

pub struct DisplayConfigurationsOrderData {
	pub list: Vec<u32>,
}

fn parse_data(bytes: &[u8]) -> EhResult<DisplayConfigurationsOrderData> {
	let iterator = &mut CustomIterator::borrow(bytes);
	let entry_amount = mp_reader::read_array(iterator).wrap(ex!("While reading display configuration entry count in extra data"))?;
	let mut list = Vec::with_capacity(entry_amount as usize);
	for _ in 0..entry_amount {
		let order_entry = mp_reader::read_i32(iterator).wrap(ex!("While reading display configuration entry in extra data"))?;
		if order_entry < 0 {
			exception!("Display configuration order index must not be negative!")?
		}
		list.push(order_entry as u32);
	}
	if iterator.has_more() {
		exception!("Iterator had more to read while parsing display configuration extra data")?;
	}
	Ok(DisplayConfigurationsOrderData {
		list,
	})
}

impl GenericExtraData for DisplayConfigurationsOrder {
	fn validate_default_bytes(&self, bytes: &[u8]) -> bool {
		let suggested_default = unwrap_or_return!(parse_data(bytes), |error: ExceptionDetails| {
			log_warn!("Client sent invalid default extra data:");
			error.print(); //TODO: Format as warning.
			false
		});
		//Currently we got to trust the client...
		if self.data.is_none() {
			//TODO: REMOVE, temporary fix (to see what the client thinks)!
			// Remove as soon as local storage exists.
			unsafe {
				let const_pointer = &self.data as *const Option<DisplayConfigurationsOrderData>;
				let mut_pointer = const_pointer as *mut Option<DisplayConfigurationsOrderData>;
				*mut_pointer = Some(suggested_default);
			}
		}
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
		self.data = Some(new_data);
		log_info!("Client change display configuration order for ", self.peg_count, " list to ", format!("{:?}", self.data.as_ref().unwrap().list));
		true
	}
	
	fn key(&self) -> String {
		format!("MHG.DisplayConfigurations/{}_pegs/_Order", self.peg_count)
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
		writer::write_array_auto(&mut buffer, data.list.len() as u32);
		for entry in data.list.iter() {
			writer::write_int_auto(&mut buffer, *entry);
		}
		buffer
	}
}

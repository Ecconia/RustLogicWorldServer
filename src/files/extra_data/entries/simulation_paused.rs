use crate::prelude::*;

use crate::files::extra_data::manager::GenericExtraData;
use crate::network::message_pack::reader as mp_reader;
use crate::util::custom_iterator::CustomIterator;
use crate::util::error_handling::ExceptionDetails;

pub const KEY: &str = "MHG.SimulationPaused";
pub const TYPE: &str = "System.Boolean";

#[derive(Default)]
pub struct SimulationPaused {
	pub paused: bool,
}

fn parse_data(bytes: &[u8]) -> EhResult<SimulationPaused> {
	let iterator = &mut CustomIterator::borrow(bytes);
	let bool_value = mp_reader::read_bool(iterator).wrap(ex!("asdf"))?;
	Ok(SimulationPaused {
		paused: bool_value,
	})
}

impl GenericExtraData for SimulationPaused {
	fn validate_default_bytes(&self, bytes: &[u8]) -> bool {
		let suggested_default = unwrap_or_return!(parse_data(bytes), |error: ExceptionDetails| {
			log_warn!("Client sent invalid default extra data:");
			error.print(); //TODO: Format as warning.
			false
		});
		#[allow(clippy::bool_comparison)] //In this case, it is semantically better to spell it out.
		if suggested_default.paused != false {
			log_warn!("Client queried extra data ", KEY, ", but its default value is different from the servers default:\n\
			  > The server starts with the simulation running, and so should the client have the default value.");
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
		self.paused = new_data.paused;
		log_info!("Client set simulation to ", self.paused);
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
		writer::write_bool(&mut buffer, self.paused);
		buffer
	}
}

use crate::prelude::*;

use crate::files::extra_data::manager::GenericExtraData;
use crate::network::message_pack::reader as mp_reader;
use crate::util::custom_iterator::CustomIterator;
use crate::util::error_handling::ExceptionDetails;

pub const KEY: &str = "MHG.SimulationSpeed";
pub const TYPE: &str = "System.Double";

#[derive(Default)]
pub struct SimulationSpeed {
	pub speed: f64,
}

fn parse_data(bytes: &[u8]) -> EhResult<SimulationSpeed> {
	let iterator = &mut CustomIterator::borrow(bytes);
	let bool_value = mp_reader::read_f64(iterator).wrap(ex!("While reading extra data simulation speed"))?;
	Ok(SimulationSpeed {
		speed: bool_value,
	})
}

impl GenericExtraData for SimulationSpeed {
	fn validate_default_bytes(&self, bytes: &[u8]) -> bool {
		let suggested_default = unwrap_ok_or_return!(parse_data(bytes), |error: ExceptionDetails| {
			log_warn!("Client sent invalid default extra data:");
			error.print(); //TODO: Format as warning.
			false
		});
		if suggested_default.speed != 0.0 {
			log_warn!("Client queried extra data ", KEY, ", but its default value is different from the normal default:\n\
			  > TPS suggested by the client is normally on 0, this one was on", suggested_default.speed, ".");
		}
		true
	}
	
	fn update_bytes_if_valid(&mut self, bytes: &[u8]) -> bool {
		let new_data = unwrap_ok_or_return!(parse_data(bytes), |error: ExceptionDetails| {
			log_warn!("Client sent invalid new extra data:");
			error.print(); //TODO: Format as warning.
			false
		});
		//TODO: Queue listener updates.
		if new_data.speed < 0.0 {
			log_warn!("Player tried to set the TPS to a negative value, clearly a malicious action.");
			return false;
		}
		self.speed = new_data.speed;
		log_info!("Client set TPS to ", self.speed);
		true
	}
	
	fn key(&self) -> String {
		KEY.to_owned()
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
		writer::write_float_64(&mut buffer, self.speed);
		buffer
	}
}

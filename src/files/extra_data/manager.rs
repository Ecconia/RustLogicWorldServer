use crate::prelude::*;

use std::collections::HashMap;
use std::net::SocketAddr;
use crate::lidgren::lidgren_server::ServerInstance;
use crate::network::message_pack::pretty_printer::pretty_print_data;
use crate::network::packets::c2s::extra_data_change::ExtraDataChange;
use crate::network::packets::c2s::extra_data_request::ExtraDataRequest;
use crate::network::packets::s2c::extra_data_update::ExtraDataUpdate;
use crate::util::custom_iterator::CustomIterator;

use crate::files::extra_data::entries::{flag_list_order, simulation_speed, simulation_paused, display_configuration, display_configurations_order, world_type_data};

#[derive(Default)]
pub struct ExtraDataManager {
	extra_data_map: HashMap<String, Box<dyn GenericExtraData>>,
}

impl ExtraDataManager {
	pub fn handle_request(&mut self, request_packet: ExtraDataRequest, server: &mut ServerInstance, address: SocketAddr) {
		pretty_print_data(&mut CustomIterator::borrow(&request_packet.default));
		let extra_data = unwrap_or_return!(self.resolve_key(&request_packet.key[..]), {
			log_warn!("Client tried to query unknown ExtraData: '", request_packet.key, "'");
			log_debug!(" Type is btw: ", request_packet.data_type);
		});
		if !Self::validate_request_data(&request_packet, extra_data) {
			log_warn!("Client sent invalid default data for extra data key ", extra_data.key(), ", ignoring packet!");
			return;
		}
		//Request is validated and extra data exists, reply data:
		let packet = Self::pack(extra_data);
		let mut buffer = Vec::new();
		packet.write(&mut buffer);
		server.send_to(address, buffer);
	}
	
	pub fn handle_change(&mut self, change_packet: ExtraDataChange, server: &mut ServerInstance, address: SocketAddr) {
		pretty_print_data(&mut CustomIterator::borrow(&change_packet.data_bytes));
		let extra_data = unwrap_or_return!(self.resolve_key(&change_packet.key[..]), {
			log_warn!("Client tried to update unknown ExtraData: '", change_packet.key, "'");
			log_debug!(" Type is btw: ", change_packet.data_type);
		});
		if extra_data.data_type_network() != change_packet.data_type {
			log_warn!("Client updated extra data with key ", extra_data.key(), ", but expects the data type '", change_packet.data_type, "', while it should expect ", extra_data.data_type_network(), ". Ignoring packet and not validating suggested default data.");
			return;
		}
		if !extra_data.update_bytes_if_valid(&change_packet.data_bytes) {
			return;
		}
		//Update is validated and extra data exists, reply data:
		//TODO: No need to serialize again...
		let packet = Self::pack(extra_data);
		let mut buffer = Vec::new();
		packet.write(&mut buffer);
		server.send_to(address, buffer);
	}
	
	fn validate_request_data(request_packet: &ExtraDataRequest, extra_data: &dyn GenericExtraData) -> bool {
		if extra_data.data_type_network() != &request_packet.data_type[..] {
			log_warn!("Client queried extra data with key ", extra_data.key(), ", but expects the data type '", request_packet.data_type, "', while it should expect ", extra_data.data_type_network(), ".");
			return false;
		}
		extra_data.validate_default_bytes(&request_packet.default[..])
	}
	
	fn pack(extra_data: &dyn GenericExtraData) -> ExtraDataUpdate {
		ExtraDataUpdate {
			key: extra_data.key(),
			data_type: extra_data.data_type_network(),
			data: extra_data.serialize_data(),
		}
	}
	
	fn resolve_key(&mut self, key: &str) -> Option<&mut dyn GenericExtraData> {
		macro_rules! get_for_key {
			($key:expr, $val:expr) => {
				Some(self.extra_data_map.entry(key.to_string()).or_insert_with(|| Box::new($val)).as_mut())
			}
		}
		match key {
			simulation_paused::KEY => {
				get_for_key!(key, simulation_paused::SimulationPaused::default())
			}
			flag_list_order::KEY => {
				get_for_key!(key, flag_list_order::FlagListOrder::default())
			}
			simulation_speed::KEY => {
				get_for_key!(key, simulation_speed::SimulationSpeed::default())
			}
			world_type_data::KEY => {
				//Works, but other worlds have other types... must be primed on world loading.
				get_for_key!(key, world_type_data::WorldTypeDataGridlands::default())
			}
			_ => {
				const PREFIX: &str = "MHG.DisplayConfigurations/";
				const PREFIX_LENGTH: usize = PREFIX.len();
				if key.starts_with(PREFIX) {
					let mut p_key = &key[PREFIX_LENGTH..];
					//At this point there will be a display configuration. Validate/Parse key:
					let position = p_key.chars().position(|l| { l > '9' || l < '0' })?;
					if position == 0 {
						return None; //We are on a letter, but should be on a digit -> wrong format.
					}
					let pegs_string = &p_key[..position];
					if pegs_string.len() > 1000 {
						return None; //Okay no jokes on you. Thats too many pegs.
					}
					let pegs = u32::from_str_radix(pegs_string, 10).expect("Should not happen, input should always only be digits.");
					p_key = &p_key[position..];
					if p_key.eq("_pegs/_Order") {
						log_info!("Got the ORDER exta data for ", pegs, " pegs");
						return get_for_key!(key, display_configurations_order::DisplayConfigurationsOrder::new(pegs));
					}
					if !p_key.starts_with("_pegs/Configuration") {
						return None; //Unknown format.
					}
					p_key = &p_key[("_pegs/Condiguration".len())..];
					if p_key.len() > 1000000{
						return None; //Either junk, or over a million configurations. Lets not support that.
					}
					let position = p_key.chars().position(|l| { l > '9' || l < '0' });
					if position.is_some() {
						return None; //Not expecting any more letters here.
					}
					let configuration_index = u32::from_str_radix(p_key, 10).expect("Should not happen, input should always only be digits.");
					log_info!("Got the Configuration #", configuration_index, " exta data for ", pegs, " pegs");
					return get_for_key!(key, display_configuration::DisplayConfiguration::new(pegs, configuration_index));
				}
				None
			}
		}
	}
}

pub trait GenericExtraData {
	//Return true if successfully validated extra data type and default, else false:
	fn validate_default_bytes(&self, bytes: &[u8]) -> bool;
	fn update_bytes_if_valid(&mut self, bytes: &[u8]) -> bool;
	
	fn key(&self) -> String; //TBI: Has to be owned, one probably could do some lifetime hackery, but not now
	fn data_type_network(&self) -> &str;
	fn data_type_file(&self) -> &str;
	fn serialize_data(&self) -> Vec<u8>;
	//The deserialization function is not yet required.
}

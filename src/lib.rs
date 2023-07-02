pub mod files {
	pub mod world_data {
		pub mod world_file_parser;
		pub mod world_structs;
	}
	pub mod extra_data {
		pub mod manager;
		pub mod entries {
			pub mod display_configuration;
			pub mod display_configurations_order;
			pub mod flag_list_order;
			pub mod simulation_paused;
			pub mod simulation_speed;
			pub mod world_type_data;
		}
	}
	pub mod world_files;
}

pub mod network {
	pub mod message_pack {
		pub mod pretty_printer;
		pub mod reader;
		pub mod writer;
	}
	
	pub mod packets {
		pub mod packet_ids;
		pub mod packet_tools;
		pub mod compression;
		
		pub mod c2s {
			pub mod discovery_request;
			pub mod connection_established;
			pub mod connection_approval;
			pub mod player_position;
			pub mod extra_data_request;
			pub mod extra_data_change;
		}
		
		pub mod s2c {
			pub mod discovery_response;
			pub mod world_initialization_packet;
			pub mod extra_data_update;
		}
	}
}

pub mod lidgren {
	pub mod util {
		pub mod formatter;
	}
	
	pub mod channel_handler {
		pub mod reliable_ordered;
	}
	
	pub mod channel_sender {
		pub mod reliable_ordered;
	}
	
	pub mod data_structures;
	pub mod message_type;
	pub mod lidgren_server;
	pub mod connected_client;
	pub mod data_types;
}

pub mod util {
	pub mod succ {
		pub mod succ_parser;
	}
	
	pub mod error_handling;
	pub mod custom_iterator;
	pub mod log_formatter;
	pub mod ansi_constants;
}

pub mod prelude;
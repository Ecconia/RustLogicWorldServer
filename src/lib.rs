pub mod network {
	pub mod message_pack {
		pub mod pretty_printer;
		pub mod reader;
		pub mod writer;
	}
	
	pub mod packets {
		pub mod packet_ids;
		
		pub mod c2s {
			pub mod discovery_request;
			pub mod connection_approval;
		}
		
		pub mod s2c {
			pub mod discovery_response;
			pub mod world_initialization_packet;
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
	pub mod error_handling;
	pub mod custom_iterator;
	pub mod log_formatter;
	pub mod ansi_constants;
}

pub mod prelude;
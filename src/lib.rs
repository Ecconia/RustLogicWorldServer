pub mod network {
	pub mod message_pack {
		pub mod reader;
		pub mod writer;
	}
	
	pub mod packets {
		pub mod c2s {
			pub mod discovery;
			pub mod connect;
		}
		
		pub mod s2c {
			pub mod discovery_response;
		}
	}
}

pub mod lidgren {
	pub mod util {
		pub mod formatter;
	}
	
	pub mod data_structures;
	pub mod message_type;
	pub mod lidgren_server;
}

pub mod error_handling;

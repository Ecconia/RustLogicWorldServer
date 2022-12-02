use std::net::SocketAddr;
use rand::Rng;

use rust_potato_server::lidgren;
use rust_potato_server::network;
use rust_potato_server::error_handling::custom_unwrap_result_or_else;
use rust_potato_server::util;

use network::packets::c2s::discovery::Discovery;
use network::packets::s2c::discovery_response::DiscoveryResponse;
use network::packets::c2s::connect::Connect;
use network::message_pack::reader as mp_reader;
use lidgren::message_type::MessageType;
use lidgren::lidgren_server::ServerInstance;
use lidgren::lidgren_server::{MessageDetails, PacketCallback};
use rust_potato_server::lidgren::lidgren_server::SendCallback;
use util::custom_iterator::CustomIterator;

struct LWS {}

impl PacketCallback for LWS {
	fn handle_user_packet(&self, _send_callback: &SendCallback, data: Vec<u8>) {
		//There is only data type... just ignore the header and get right to the data.
		
		let mut iterator = CustomIterator::create(&data[..]);
		let it = &mut iterator;
		let packet_id = mp_reader::read_int_auto(it);
		println!("[UserPacket] Received data packet with ID: {}", packet_id);
		
		if packet_id == 17 {
			println!("[UserPacket] Type: ConnectionEstablishedPacket");
			let mut number = mp_reader::read_array_auto(it);
			if number != 1 {
				println!("Error: expected connection-established to have one element as array, got: {}", number);
			}
			number = mp_reader::read_int_auto(it);
			if number != 0 {
				println!("Error: expected connection-established expected integer of value 0, got: {}", number);
			}
			if it.has_more() {
				println!("Error: expected connection-established to stop but have {} remaining bytes.", it.remaining());
			}
			
			//TODO: Respond with the world packet...
		}
	}
	
	fn handle_system_packet(&self, message: MessageDetails, server: &ServerInstance, iterator: &mut CustomIterator) {
		match message.header.message_type {
			MessageType::Discovery => {
				println!("=> Discovery!");
				handle_discovery(server, &message.address, iterator);
			}
			MessageType::Connect => {
				println!("=> Connect!");
				handle_connect(server, &message.address, iterator);
			}
			MessageType::UserReliableOrdered(channel) => {
				println!("=> UserReliableOrdered on channel {}!", channel);
				println!("-Cannot handle yet-");
			}
			_ => {
				println!("Error: Cannot handle {:x?} yet!", message.header.message_type);
			}
		}
	}
}

fn main() {
	let mut rand = rand::thread_rng();
	let random_unique_id = rand.gen();
	let mut server = custom_unwrap_result_or_else!(ServerInstance::start(
		String::from("Logic World"),
		random_unique_id,
		String::from("[::]:43531"),
		Box::new(LWS {}),
	), (|error| {
		println!("Issue starting server:\n{}", error);
	}));
	
	loop {
		println!("====================================");
		server.read_input();
	}
}

fn handle_discovery(server: &ServerInstance, remote_address: &SocketAddr, iterator: &mut CustomIterator)
{
	let request = custom_unwrap_result_or_else!(Discovery::parse(iterator), (|message| {
		println!("Error while parsing the clients Discovery packet: {}", message);
		return;
	}));
	
	//Answer:
	
	let mut result_buffer = Vec::new();
	let response = DiscoveryResponse::simple(
		request.request_uid.clone(),
		420,
		false,
		false,
	);
	response.write(&mut result_buffer);
	
	server.answer_discovery(remote_address, &result_buffer[..]);
}

fn handle_connect(server: &ServerInstance, remote_address: &SocketAddr, iterator: &mut CustomIterator)
{
	if let Err(message) = Connect::parse(iterator) {
		println!("Error while parsing connect packet: {}", message);
		return;
	}
	
	//Send answer:
	
	server.answer_connect(remote_address);
}

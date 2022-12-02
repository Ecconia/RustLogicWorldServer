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
use network::message_pack::writer as mp_writer;
use lidgren::message_type::MessageType;
use lidgren::lidgren_server::ServerInstance;
use lidgren::lidgren_server::{MessageDetails, PacketCallback};
use rust_potato_server::lidgren::lidgren_server::SendCallback;
use util::custom_iterator::CustomIterator;

struct LWS {}

impl PacketCallback for LWS {
	fn handle_user_packet(&self, send_callback: &SendCallback, data: Vec<u8>) {
		//There is only data type... just ignore the header and get right to the data.
		
		let mut iterator = CustomIterator::create(&data[..]);
		let it = &mut iterator;
		let packet_id = mp_reader::read_int_auto(it);
		println!("Received data packet with ID: {}", packet_id);
		
		if packet_id == 17 {
			println!("Received: ConnectionEstablishedPacket");
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
			
			//Respond! - No need!
			
			// let mut result_buffer = Vec::new();
			// result_buffer.push(MessageType::ConnectResponse.to_index());
			// result_buffer.push(0);
			// result_buffer.push(0);
			// result_buffer.push(0);
			// result_buffer.push(0);
			//
			// {
			// 	let buf = &mut result_buffer;
			// 	mp_writer::write_array_auto(buf, 8);
			// }
			//
			// let size = (result_buffer.len() - 5) * 8;
			// result_buffer[3] = size as u8;
			// result_buffer[4] = (size >> 8) as u8;
			//
			// let len = send_callback.send(&result_buffer);
			// println!("{} bytes sent", len);
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
	
	result_buffer.push(MessageType::DiscoveryResponse.to_index());
	result_buffer.push(0);
	result_buffer.push(0);
	result_buffer.push(0);
	result_buffer.push(0);
	
	let response = DiscoveryResponse::simple(request.request_uid.clone(), 420, false, false);
	response.write(&mut result_buffer);
	
	let size = (result_buffer.len() - 5) * 8;
	result_buffer[3] = size as u8;
	result_buffer[4] = (size >> 8) as u8;
	
	let len = server.send(&result_buffer, remote_address);
	println!("{} bytes sent", len);
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

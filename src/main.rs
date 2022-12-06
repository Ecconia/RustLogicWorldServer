use std::net::SocketAddr;
use std::ops::Sub;
use std::thread::sleep;
use std::time::{Duration, Instant};
use rand::Rng;

use rust_potato_server::lidgren;
use rust_potato_server::network;
use rust_potato_server::error_handling::custom_unwrap_result_or_else;
use rust_potato_server::util;

use network::packets::c2s::discovery::Discovery;
use network::packets::s2c::discovery_response::DiscoveryResponse;
use network::packets::c2s::connect::Connect;
use network::message_pack::reader as mp_reader;
use lidgren::lidgren_server::ServerInstance;
use rust_potato_server::lidgren::data_types::DataType;
use util::custom_iterator::CustomIterator;

fn main() {
	let mut rand = rand::thread_rng();
	let random_unique_id = rand.gen();
	let mut server = custom_unwrap_result_or_else!(ServerInstance::start(
		String::from("Logic World"),
		random_unique_id,
		String::from("[::]:43531"),
	), (|error| {
		println!("Issue starting server:\n{}", error);
	}));
	
	let mut packets_to_process = Vec::new();
	let min_tick_duration = Duration::from_millis(16);
	loop {
		let tick_start = Instant::now();
		server.heartbeat();
		if !server.new_data_packets.is_empty() {
			//Swap the packets to process list, so that they can be processed, without blocking the server.
			packets_to_process = std::mem::replace(&mut server.new_data_packets, packets_to_process);
			for user_packet in packets_to_process.drain(..) {
				println!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
				match user_packet.data_type {
					DataType::Discovery => {
						println!("=> Discovery!");
						handle_discovery(&server, user_packet.remote_address, user_packet.data);
					}
					DataType::Connect => {
						println!("=> Connect!");
						handle_connect(&server, user_packet.remote_address, user_packet.data);
					}
					DataType::Data => {
						println!("=> Data!");
						handle_user_packet(&server, user_packet.remote_address, user_packet.data);
					}
				}
			}
		}
		//Don't start another tick, before 16 ms are over.
		//Subject to change in future, but for now don't let this run amok, as there is not much to do.
		let elapsed = tick_start.elapsed();
		if elapsed.le(&min_tick_duration) {
			sleep(min_tick_duration.sub(elapsed));
		}
	}
}

fn handle_user_packet(
	_server: &ServerInstance,
	_address: SocketAddr,
	data: Vec<u8>,
) {
	let mut iterator = CustomIterator::create(&data[..]);
	let it = &mut iterator;
	let packet_id = custom_unwrap_result_or_else!(mp_reader::read_int_auto(it), (|message| {
			println!("While reading user packet ID:\n -> {}", message);
			return;
		}));
	println!("[UserPacket] Received data packet with ID: {}", packet_id);
	
	if packet_id == 17 {
		println!("[UserPacket] Type: ConnectionEstablishedPacket");
		let mut number = custom_unwrap_result_or_else!(mp_reader::read_array_auto(it), (|message| {
				println!("While parsing ConnectionEstablishedPacket's entry count:\n -> {}", message);
				return;
			}));
		if number != 1 {
			println!("Error: expected connection-established to have one element as array, got: {}", number);
			return;
		}
		number = custom_unwrap_result_or_else!(mp_reader::read_int_auto(it), (|message| {
				println!("While parsing ConnectionEstablishedPacket's dummy value:\n -> {}", message);
				return;
			}));
		if number != 0 {
			println!("Error: expected connection-established expected integer of value 0, got: {}", number);
			return;
		}
		if it.has_more() {
			println!("Error: expected connection-established to stop but have {} remaining bytes.", it.remaining());
			return;
		}
		
		//TODO: Respond with the world packet...
	}
}

fn handle_discovery(
	server: &ServerInstance,
	remote_address: SocketAddr,
	data: Vec<u8>,
) {
	let mut iterator = CustomIterator::create(&data[..]);
	let request = custom_unwrap_result_or_else!(Discovery::parse(&mut iterator), (|message| {
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
	
	server.answer_discovery(&remote_address, &result_buffer[..]);
}

fn handle_connect(
	server: &ServerInstance,
	remote_address: SocketAddr,
	data: Vec<u8>,
) {
	let mut iterator = CustomIterator::create(&data[..]);
	if let Err(message) = Connect::parse(&mut iterator) {
		println!("Error while parsing connect packet: {}", message);
		return;
	}
	
	//Send answer:
	
	server.answer_connect(&remote_address);
}

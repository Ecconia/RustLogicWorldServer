use rust_potato_server::prelude::*;

use std::net::SocketAddr;
use std::ops::Sub;
use std::thread::sleep;
use std::time::{Duration, Instant};
use rand::Rng;

use rust_potato_server::lidgren;
use rust_potato_server::network;
use rust_potato_server::util;

use network::packets::c2s::discovery_request::DiscoveryRequest;
use network::packets::s2c::discovery_response::DiscoveryResponse;
use network::packets::c2s::connection_approval::ConnectionApproval;
use network::packets::compression::try_decompress;
use network::message_pack::reader as mp_reader;
use network::message_pack::pretty_printer::pretty_print_data;
use lidgren::lidgren_server::ServerInstance;
use rust_potato_server::files::world_data::world_structs::World;
use rust_potato_server::lidgren::data_types::DataType;
use rust_potato_server::network::packets::c2s::connection_established::ConnectionEstablished;
use rust_potato_server::network::packets::c2s::player_position::PlayerPosition;
use rust_potato_server::network::packets::packet_ids::PacketIDs;
use rust_potato_server::network::packets::s2c::world_initialization_packet::WorldInitializationPacket;
use util::custom_iterator::CustomIterator;

fn main() {
	log_info!("Starting ", "Rust Logic World Server", "!");
	
	log_info!("Starting file reading!");
	let mut world = unwrap_or_print_return!(rust_potato_server::files::world_data::world_file_parser::load_world());
	
	log_info!("Starting network socket!");
	let mut rand = rand::thread_rng();
	let random_unique_id = rand.gen();
	let mut server = unwrap_or_print_return!(exception_wrap!(ServerInstance::start(
		String::from("Logic World"),
		random_unique_id,
		String::from("[::]:43531"),
	), "While starting network server"));
	
	let mut packets_to_process = Vec::new();
	let min_tick_duration = Duration::from_millis(16);
	loop {
		let tick_start = Instant::now();
		server.heartbeat();
		if !server.new_data_packets.is_empty() {
			//Swap the packets to process list, so that they can be processed, without blocking the server.
			packets_to_process = std::mem::replace(&mut server.new_data_packets, packets_to_process);
			for user_packet in packets_to_process.drain(..) {
				log_debug!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
				match user_packet.data_type {
					DataType::Discovery => {
						log_debug!("=> Discovery!");
						handle_discovery(&server, user_packet.remote_address, user_packet.data);
					}
					DataType::Connect => {
						log_debug!("=> Connect!");
						handle_connect(&server, user_packet.remote_address, user_packet.data);
					}
					DataType::Data => {
						log_debug!("=> Data!");
						handle_user_packet(&mut server, user_packet.remote_address, user_packet.data, &mut world);
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

fn get_packet_content_iterator<'a>(data: &'a Vec<u8>) -> EhResult<(u32, CustomIterator<'a>)> {
	let mut iterator = CustomIterator::borrow(&data[..]);
	let packet_id = exception_wrap!(mp_reader::read_u32(&mut iterator), "While reading user packet id")?;
	
	let decompress_start = Instant::now();
	let decompress_result = exception_wrap!(try_decompress(&mut iterator), "asdf")?;
	if let Some(decompressed_bytes) = decompress_result {
		let duration_ms = (Instant::now() - decompress_start).as_millis();
		log_debug!("Decompressed packet in ", duration_ms, "ms");
		Ok((packet_id, CustomIterator::own(decompressed_bytes)))
	} else {
		Ok((packet_id, iterator))
	}
}

fn handle_user_packet(
	server: &mut ServerInstance,
	address: SocketAddr,
	data: Vec<u8>,
	world: &mut World,
) {
	let (packet_id, mut iterator) = unwrap_or_print_return!(
		exception_wrap!(get_packet_content_iterator(&data), "While reading LW header of packet")
	);
	let it = &mut iterator;
	
	match PacketIDs::from_u32(packet_id) {
		Some(PacketIDs::ConnectionEstablished) => {
			log_info!("[UserPacket] Type: ConnectionEstablishedPacket");
			unwrap_or_print_return!(exception_wrap!(ConnectionEstablished::parse(iterator), "While parsing ConnectionEstablished packet"));
			
			//Respond with world packet:
			
			let world_initialization_packet = WorldInitializationPacket::simple(world);
			
			let mut packet_buffer = Vec::new();
			world_initialization_packet.write(&mut packet_buffer);
			log_debug!("The packet about to be sent is ", packet_buffer.len(), " bytes long");
			
			server.send_to(address, packet_buffer);
		}
		Some(PacketIDs::PlayerPosition) => {
			log_info!("[UserPacket] Type: PlayerPositionPacket");
			unwrap_or_print_return!(exception_wrap!(PlayerPosition::parse(iterator), "While parsing PlayerPosition packet"));
		}
		_ => {
			log_warn!("Warning: Received client packet with unknown type ", packet_id);
			pretty_print_data(it);
		}
	}
}

fn handle_discovery(
	server: &ServerInstance,
	remote_address: SocketAddr,
	data: Vec<u8>,
) {
	let mut iterator = CustomIterator::borrow(&data[..]);
	unwrap_or_print_return!(exception_wrap!(DiscoveryRequest::validate_packet_id(&mut iterator), "While validating DiscoveryRequest packet ID"));
	let request = unwrap_or_print_return!(exception_wrap!(DiscoveryRequest::parse(iterator), "While parsing DiscoveryRequest packet"));
	
	//Answer:
	
	let mut result_buffer = Vec::new();
	let response = DiscoveryResponse::simple(
		request.request_uid,
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
	let mut iterator = CustomIterator::borrow(&data[..]);
	unwrap_or_print_return!(exception_wrap!(ConnectionApproval::validate_packet_id(&mut iterator), "While validating ConnectionApproval packet ID"));
	unwrap_or_print_return!(exception_wrap!(ConnectionApproval::parse(iterator), "While parsing ConnectionApproval packet"));
	
	//Send answer:
	
	server.answer_connect(&remote_address);
}

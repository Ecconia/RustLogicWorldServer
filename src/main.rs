use std::net::SocketAddr;

use rust_potato_server::lidgren;
use rust_potato_server::network;
use rust_potato_server::custom_unwrap_result_or_else;

use network::packets::c2s::discovery::Discovery;
use network::packets::s2c::discovery_response::DiscoveryResponse;
use network::packets::c2s::connect::Connect;
use lidgren::util::formatter as lg_formatter;
use lidgren::data_structures::MessageHeader;
use lidgren::message_type::MessageType;
use lidgren::lidgren_server::ServerInstance;
use rust_potato_server::lidgren::lidgren_server::{MessageDetails, PacketCallback};
use rust_potato_server::util::custom_iterator::CustomIterator;

struct LWS {}

impl PacketCallback for LWS {
	fn handle_user_packet(&self, header: MessageHeader) {}
	
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
			MessageType::ConnectionEstablished => {
				println!("=> Connection established!");
				//TODO: Read LG-Float (time)
				println!("-Cannot handle yet-");
			}
			MessageType::Ping => {
				println!("=> Ping!");
				println!("-Cannot handle yet-");
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
	let mut server = custom_unwrap_result_or_else!(ServerInstance::start(
		String::from("Logic World"),
		123,
		String::from("[::]:43531"),
		Box::new(LWS {}),
	), (|error| {
		println!("Issue starting server:\n{}", error);
	}));
	
	loop
	{
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
	
	result_buffer.push(137);
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
	let app_id = lg_formatter::read_string(iterator);
	println!("App ID: '\x1b[38;2;255;0;150m{}\x1b[m'", app_id);
	let remote_id = lg_formatter::read_int_64(iterator);
	println!("Remote ID: \x1b[38;2;255;0;150m{}\x1b[m", remote_id);
	let remote_time = lg_formatter::read_float(iterator);
	println!("Remote time: \x1b[38;2;255;0;150m{}\x1b[m", remote_time);
	
	if let Err(message) = Connect::parse(iterator) {
		println!("Error while parsing connect packet: {}", message);
		return;
	}
	
	//Send answer:
	
	let mut result_buffer = Vec::new();
	
	result_buffer.push(132);
	result_buffer.push(0);
	result_buffer.push(0);
	result_buffer.push(0);
	result_buffer.push(0);
	
	lg_formatter::write_string(&mut result_buffer, "Logic World");
	lg_formatter::write_int_64(&mut result_buffer, remote_id);
	lg_formatter::write_float(&mut result_buffer, 0.5);
	
	let size = (result_buffer.len() - 5) * 8;
	result_buffer[3] = size as u8;
	result_buffer[4] = (size >> 8) as u8;
	
	let len = server.send(&result_buffer, remote_address);
	println!("{} bytes sent", len);
}

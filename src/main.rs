use std::net::{SocketAddr, UdpSocket};
use std::iter::Peekable;
use std::slice::Iter;

use rust_potato_server::lidgren;
use rust_potato_server::network;
use rust_potato_server::custom_unwrap_result_or_else;

use network::packets::c2s::discovery::Discovery;
use network::packets::s2c::discovery_response::DiscoveryResponse;
use network::packets::c2s::connect::Connect;
use lidgren::util::formatter as lg_formatter;
use lidgren::data_structures::MessageHeader;
use lidgren::message_type::MessageType;

fn main() {
	let socket = UdpSocket::bind("127.0.0.1:43531").expect("Could not bind server socket.");
	
	let mut buf = [0; 0xFFFF];
	
	loop
	{
		println!("====================================");
		let (buffer_amount, remote_address) = socket.recv_from(&mut buf).expect("Could not read incoming datagram packet.");
		println!("Received UDP packet from \x1b[38;2;255;0;150m{}\x1b[m port \x1b[38;2;255;0;150m{}\x1b[m size \x1b[38;2;255;0;150m{}\x1b[m",
		         remote_address.ip(), remote_address.port(), buffer_amount
		);
		
		handle_packet(&socket, &remote_address, buffer_amount, &buf);
	}
}

fn handle_packet(socket: &UdpSocket, remote_address: &SocketAddr, buffer_amount: usize, buf: &[u8])
{
	if buffer_amount < 5
	{
		println!("\033[38;2;255;0;0m -> PACKET TOO SHORT\033[m");
		return;
	}
	
	let mut buffer_iterator = buf[0..buffer_amount].iter().peekable();
	
	let header = custom_unwrap_result_or_else!(MessageHeader::from_stream(&mut buffer_iterator), (|message| {
		println!("Error constructing message header: {}", message);
	}));
	println!("Type: \x1b[38;2;255;0;150m{:x?}\x1b[m Fragment: \x1b[38;2;255;0;150m{}\x1b[m Sequence#: \x1b[38;2;255;0;150m{}\x1b[m Bits: \x1b[38;2;255;0;150m{}\x1b[m Bytes: \x1b[38;2;255;0;150m{}\x1b[m",
	         header.message_type, header.fragment, header.sequence_number, header.bits, header.bytes
	);
	
	let remaining = buffer_amount - 5;
	if remaining < header.bytes as usize
	{
		println!("Not enough bytes in packet. Expected {}, but got {}", header.bytes, buffer_amount - 5);
		return;
	}
	
	match header.message_type {
		MessageType::Discovery => {
			println!("=> Discovery!");
			handle_discovery(&socket, &remote_address, &mut buffer_iterator);
		}
		MessageType::Connect => {
			println!("=> Connect!");
			handle_connect(&socket, &remote_address, &mut buffer_iterator);
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
			println!("Error: Cannot handle {:x?} yet!", header.message_type);
		}
	}
}

fn handle_discovery(socket: &UdpSocket, remote_address: &SocketAddr, buffer_iterator: &mut Peekable<Iter<u8>>)
{
	let request = custom_unwrap_result_or_else!(Discovery::parse(buffer_iterator), (|message| {
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
	
	let len = socket.send_to(&result_buffer, remote_address).unwrap();
	println!("{} bytes sent", len);
}

fn handle_connect(socket: &UdpSocket, remote_address: &SocketAddr, buffer_iterator: &mut Peekable<Iter<u8>>)
{
	let app_id = lg_formatter::read_string(buffer_iterator);
	println!("App ID: '\x1b[38;2;255;0;150m{}\x1b[m'", app_id);
	let remote_id = lg_formatter::read_int_64(buffer_iterator);
	println!("Remote ID: \x1b[38;2;255;0;150m{}\x1b[m", remote_id);
	let remote_time = lg_formatter::read_float(buffer_iterator);
	println!("Remote time: \x1b[38;2;255;0;150m{}\x1b[m", remote_time);
	
	if let Err(message) = Connect::parse(buffer_iterator) {
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
	
	let len = socket.send_to(&result_buffer, remote_address).unwrap();
	println!("{} bytes sent", len);
}

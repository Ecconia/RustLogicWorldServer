use std::net::{SocketAddr, UdpSocket};
use std::iter::Peekable;
use std::slice::Iter;

use crate::custom_unwrap_result_or_else;

use crate::lidgren::data_structures::MessageHeader;

pub struct ServerInstance {
	application_name: String,
	application_identifier: u64,
	socket: UdpSocket,
	input_buffer: [u8; 0xFFFF],
	handler: Box<dyn PacketCallback>,
}

pub struct MessageDetails {
	pub header: MessageHeader,
	pub address: SocketAddr,
}

impl ServerInstance {
	pub fn start(
		application_name: String,
		application_identifier: u64,
		target: String,
		handler: Box<dyn PacketCallback>,
	) -> Result<ServerInstance, String> {
		let socket = custom_unwrap_result_or_else!(UdpSocket::bind(target), (|error| {
			return Err(format!("Could not bind server socket! Error: {}", error));
		}));
		
		let input_buffer: [u8; 0xFFFF] = [0; 0xFFFF];
		
		return Ok(ServerInstance {
			socket,
			input_buffer,
			application_name,
			application_identifier,
			handler,
		});
	}
	
	pub fn read_input(&mut self) {
		let (amount_read, remote_address) = self.socket.recv_from(&mut self.input_buffer).expect("Could not read incoming datagram packet.");
		println!("Received UDP packet from \x1b[38;2;255;0;150m{}\x1b[m port \x1b[38;2;255;0;150m{}\x1b[m size \x1b[38;2;255;0;150m{}\x1b[m",
		         remote_address.ip(), remote_address.port(), amount_read
		);
		
		if amount_read < 5
		{
			println!("\033[38;2;255;0;0m -> PACKET TOO SHORT\033[m");
			return;
		}
		
		let mut buffer_iterator = self.input_buffer[0..amount_read].iter().peekable();
		
		let header = custom_unwrap_result_or_else!(MessageHeader::from_stream(&mut buffer_iterator), (|message| {
			println!("Error constructing message header: {}", message);
		}));
		println!("Type: \x1b[38;2;255;0;150m{:x?}\x1b[m Fragment: \x1b[38;2;255;0;150m{}\x1b[m Sequence#: \x1b[38;2;255;0;150m{}\x1b[m Bits: \x1b[38;2;255;0;150m{}\x1b[m Bytes: \x1b[38;2;255;0;150m{}\x1b[m",
		         header.message_type, header.fragment, header.sequence_number, header.bits, header.bytes
		);
		
		let remaining = amount_read - 5;
		if remaining < header.bytes as usize
		{
			println!("Not enough bytes in packet. Expected {}, but got {}", header.bytes, amount_read - 5);
			return;
		}
		
		self.handler.handle_system_packet(
			MessageDetails {
				header,
				address: remote_address,
			},
			&self,
			&mut buffer_iterator,
		);
	}
	
	pub fn send(&self, buffer: &Vec<u8>, address: &SocketAddr) -> usize
	{
		return self.socket.send_to(&buffer, address).unwrap();
	}
}

pub trait PacketCallback {
	fn handle_user_packet(&self, header: MessageHeader);
	fn handle_system_packet(&self, message: MessageDetails, server: &ServerInstance, iterator: &mut Peekable<Iter<u8>>);
}
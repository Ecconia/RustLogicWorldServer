use std::net::{SocketAddr, UdpSocket};

use crate::custom_unwrap_result_or_else;

use crate::lidgren::data_structures::{MESSAGE_HEADER_LENGTH, MessageHeader};
use crate::util::custom_iterator::CustomIterator;

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
		
		if amount_read < MESSAGE_HEADER_LENGTH
		{
			//Drop packet, it cannot even hold a single Lidgren message header.
			println!("Dropping packet, message header won't fit inside.");
			return;
		}
		
		let mut iterator = CustomIterator::create(&self.input_buffer[0..amount_read]);
		
		while iterator.remaining() >= MESSAGE_HEADER_LENGTH {
			
			let header = custom_unwrap_result_or_else!(MessageHeader::from_stream(&mut iterator), (|message| {
				println!("Error constructing message header: {}", message);
			}));
			println!("Type: \x1b[38;2;255;0;150m{:x?}\x1b[m Fragment: \x1b[38;2;255;0;150m{}\x1b[m Sequence#: \x1b[38;2;255;0;150m{}\x1b[m Bits: \x1b[38;2;255;0;150m{}\x1b[m Bytes: \x1b[38;2;255;0;150m{}\x1b[m",
			         header.message_type, header.fragment, header.sequence_number, header.bits, header.bytes
			);
			
			if (iterator.remaining() as u16) < header.bytes {
				println!("Message header declared payload size bigger than rest of packet: {}/{}", header.bytes, iterator.remaining());
				return;
			}
			
			let mut message_data_iterator = custom_unwrap_result_or_else!(iterator.sub_section(header.bytes as usize), (|message| {
				println!("While creating message-sub-iterator: {}", message);
				return;
			}));
			
			self.handler.handle_system_packet(
				MessageDetails {
					header,
					address: remote_address,
				},
				&self,
				&mut message_data_iterator,
			);
		}
		if iterator.remaining() > 0 {
			println!("Dropping packet, there had been additional bytes to read that don't fit a message header. Amount {}", iterator.remaining());
			return;
		}
	}
	
	pub fn send(&self, buffer: &Vec<u8>, address: &SocketAddr) -> usize
	{
		return self.socket.send_to(&buffer, address).unwrap();
	}
}

pub trait PacketCallback {
	fn handle_user_packet(&self, header: MessageHeader);
	fn handle_system_packet(&self, message: MessageDetails, server: &ServerInstance, iterator: &mut CustomIterator);
}
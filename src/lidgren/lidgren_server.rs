use std::collections::HashMap;
use std::io::ErrorKind::WouldBlock;
use std::net::{SocketAddr, UdpSocket};
use std::ops::Add;
use std::time::{Duration, Instant};

use crate::error_handling::{custom_unwrap_option_or_else, custom_unwrap_result_or_else, ResultErrorExt};

use crate::lidgren::data_structures::{MESSAGE_HEADER_LENGTH, MessageHeader};
use crate::util::custom_iterator::CustomIterator;
use crate::lidgren::message_type::MessageType;
use crate::lidgren::connected_client::ConnectedClient;
use crate::lidgren::data_types::DataType;
use crate::lidgren::util::formatter as lg_formatter;

pub struct ServerInstance {
	application_name: String,
	server_unique_id: u64,
	socket: UdpSocket,
	input_buffer: [u8; 0xFFFF],
	user_map: HashMap<SocketAddr, ConnectedClient>,
	time_run_duration: Instant,
	time_cleanup: Instant,
	pub new_data_packets: Vec<DataPacket>,
}

pub struct MessageDetails {
	pub header: MessageHeader,
	pub address: SocketAddr,
}

pub struct DataPacket {
	pub data_type: DataType,
	//TODO: Eventually somehow store the client here, instead of this. But lets not fight the borrow checker yet...
	pub remote_address: SocketAddr,
	pub data: Vec<u8>,
}

impl ServerInstance {
	pub fn start(
		application_name: String,
		server_unique_id: u64,
		target: String,
	) -> Result<ServerInstance, String> {
		let socket = UdpSocket::bind(target).forward_error("Could not bind server socket! Error: {}")?;
		socket.set_nonblocking(true).expect("Could not set the socket to non-blocking mode.");
		
		let input_buffer: [u8; 0xFFFF] = [0; 0xFFFF];
		let now = Instant::now();
		
		Ok(ServerInstance {
			socket,
			input_buffer,
			application_name,
			server_unique_id,
			user_map: HashMap::new(),
			time_run_duration: now,
			time_cleanup: now,
			new_data_packets: Vec::new(),
		})
	}
	
	pub fn heartbeat(&mut self) {
		let duration_between_cleanups = Duration::from_millis(500);
		if self.time_cleanup.elapsed().ge(&duration_between_cleanups) {
			for client in self.user_map.values_mut() {
				client.heartbeat();
			}
			self.time_cleanup = self.time_cleanup.add(duration_between_cleanups);
		}
		
		let start = Instant::now();
		let max_read_duration = Duration::from_millis(100);
		//Read packets until at max 100ms have passed, then the rest of the program should continue (to consume the new packets).
		while start.elapsed().lt(&max_read_duration)
		{
			match self.socket.recv_from(&mut self.input_buffer) {
				Err(err) if err.kind() == WouldBlock => {
					break; //Nothing to read right now, so just stop attempting for now.
				}
				Err(err) => println!("Error while reading from socket: {:?}", err),
				Ok((amount_read, remote_address)) => {
					self.process_packet(amount_read, remote_address);
				}
			}
		}
	}
	
	pub fn process_packet(&mut self, amount_read: usize, remote_address: SocketAddr) {
		println!("====================================");
		println!("Received UDP packet from \x1b[38;2;255;0;150m{}\x1b[m port \x1b[38;2;255;0;150m{}\x1b[m size \x1b[38;2;255;0;150m{}\x1b[m",
		         remote_address.ip(), remote_address.port(), amount_read
		);
		
		if amount_read < MESSAGE_HEADER_LENGTH {
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
			
			if let MessageType::Unused(_) = header.message_type {
				println!("Received Unused/Reserved message type. Stopping parsing.");
				return;
			}
			
			let mut message_data_iterator = custom_unwrap_result_or_else!(iterator.sub_section(header.bytes as usize), (|message| {
				println!("While creating message-sub-iterator: {}", message);
				return;
			}));
			
			if MessageType::is_system(&header.message_type) {
				match header.message_type {
					MessageType::Connect => {
						let app_id = custom_unwrap_result_or_else!(lg_formatter::read_string(&mut message_data_iterator), (|message| {
							println!("While reading app ID in Connect message, encountered issue:\n-> {}", message);
						}));
						if self.application_name.ne(&app_id) {
							println!("Remote {}:{} sent wrong application identifier name '{}'.", remote_address.ip(), remote_address.port(), app_id);
							return;
						}
						//TODO: Actually somehow use the ID? Only useful if routers actually do funky stuff...
						let _remote_id = lg_formatter::read_int_64(&mut message_data_iterator);
						let remote_time = custom_unwrap_result_or_else!(lg_formatter::read_float(&mut message_data_iterator).forward_error("While reading the remote time"), (|message| {
							println!("Dropping packet, as an error has occurred:\n{}", message);
						}));
						println!("Remote time: \x1b[38;2;255;0;150m{}\x1b[m", remote_time);
						self.new_data_packets.push(DataPacket {
							data_type: DataType::Connect,
							remote_address,
							data: message_data_iterator.consume()
						});
					}
					MessageType::Discovery => {
						//Accept!
						self.new_data_packets.push(DataPacket {
							data_type: DataType::Discovery,
							remote_address,
							data: message_data_iterator.consume()
						});
					}
					MessageType::ConnectionEstablished => {
						if message_data_iterator.remaining() != 4 {
							println!("Remote {}:{} sent invalid connection established message, expected exactly 4 bytes, got {}.", remote_address.ip(), remote_address.port(), message_data_iterator.remaining());
							return;
						}
						let remote_time = custom_unwrap_result_or_else!(lg_formatter::read_float(&mut message_data_iterator).forward_error("While reading the remote time"), (|message| {
							println!("Dropping packet, as an error has occurred:\n{}", message);
						}));
						println!("Remote time: {}", remote_time);
						//Register user:
						self.user_map.insert(remote_address.clone(), ConnectedClient::new(remote_address.clone()));
						return; //Nothing to do actually...
					}
					MessageType::Ping => {
						if message_data_iterator.remaining() != 1 {
							println!("Remote {}:{} sent invalid ping message, expected exactly 1 byte, got {}.", remote_address.ip(), remote_address.port(), message_data_iterator.remaining());
							return;
						}
						let ping_number = message_data_iterator.next_unchecked();
						println!("Ping packet: {}", ping_number);
						//Respond:
						{
							let mut result_buffer = Vec::with_capacity(5 + 1 + 4);
							//Header:
							result_buffer.push(MessageType::Pong.to_index());
							result_buffer.push(0);
							result_buffer.push(0);
							result_buffer.push(0);
							result_buffer.push(0);
							//Data:
							result_buffer.push(ping_number);
							let elapsed_time = self.time_run_duration.elapsed().as_millis() as f32 / 1000.0;
							println!("Time passed: {}", elapsed_time);
							lg_formatter::write_float(&mut result_buffer, elapsed_time);
							
							let size = (result_buffer.len() - 5) * 8;
							result_buffer[3] = size as u8;
							result_buffer[4] = (size >> 8) as u8;
							
							let len = self.send(&result_buffer, &remote_address);
							println!("{} bytes sent", len);
						}
						return; //Done here.
					}
					_ => {
						//Reject!
						println!("Rejecting message type {:?} from {}:{} remaining bytes {}",
						         header.message_type, remote_address.ip(), remote_address.port(), message_data_iterator.remaining());
						return;
					}
				}
			} else {
				//Get channel:
				match header.message_type {
					MessageType::UserReliableOrdered(channel) => {
						//Acknowledge:
						{
							//TODO: Do acknowledge somewhere else? (And collected)
							let mut result_buffer = Vec::new();
							result_buffer.push(MessageType::Acknowledge.to_index());
							result_buffer.push(0);
							result_buffer.push(0);
							let length = 1 * 3 * 8;
							result_buffer.push(length as u8);
							result_buffer.push((length >> 8) as u8);
							
							result_buffer.push(header.message_type.to_index());
							result_buffer.push(header.sequence_number as u8);
							result_buffer.push((header.sequence_number >> 8) as u8);
							
							let len = self.send(&result_buffer, &remote_address);
							println!("{} bytes sent", len);
						}
						//Handle:
						if channel != 0 {
							println!("Cannot handle anything but channel 0 yet!");
							return;
						}
						let connected_client = self.user_map.get_mut(&remote_address);
						let connected_client = custom_unwrap_option_or_else!(connected_client, {
							println!("Client sent user-message, while not being connected!");
							return;
						});
						connected_client.handle_new_message(
							&mut self.new_data_packets,
							remote_address,
							header,
							message_data_iterator,
						);
					}
					_ => {
						println!("Unexpected/Unimplemented message type!");
						return;
					}
				};
			}
		}
		if iterator.remaining() > 0 {
			println!("Dropping packet, there had been additional bytes to read that don't fit a message header. Amount {}", iterator.remaining());
			return;
		}
	}
	
	pub fn answer_connect(&self, remote_address: &SocketAddr) {
		let mut result_buffer = Vec::new();
		result_buffer.push(MessageType::ConnectResponse.to_index());
		result_buffer.push(0);
		result_buffer.push(0);
		result_buffer.push(0);
		result_buffer.push(0);
		
		lg_formatter::write_string(&mut result_buffer, &self.application_name);
		lg_formatter::write_int_64(&mut result_buffer, self.server_unique_id);
		let elapsed_time = self.time_run_duration.elapsed().as_millis() as f32 / 1000.0;
		println!("Time passed: {}", elapsed_time);
		lg_formatter::write_float(&mut result_buffer, elapsed_time);
		
		let size = (result_buffer.len() - 5) * 8;
		result_buffer[3] = size as u8;
		result_buffer[4] = (size >> 8) as u8;
		
		let len = self.send(&result_buffer, remote_address);
		println!("{} bytes sent", len);
	}
	
	pub fn answer_discovery(&self, remote_address: &SocketAddr, discovery_payload: &[u8]) {
		let payload_length = discovery_payload.len() * 8;
		//TODO: panic if payload too large!
		
		let mut result_buffer = Vec::new();
		result_buffer.push(MessageType::DiscoveryResponse.to_index());
		result_buffer.push(0);
		result_buffer.push(0);
		result_buffer.push(payload_length as u8);
		result_buffer.push((payload_length >> 8) as u8);
		
		result_buffer.extend_from_slice(discovery_payload);
		
		let len = self.socket.send_to(&result_buffer, remote_address).unwrap();
		println!("{} bytes sent", len);
	}
	
	pub fn send(&self, buffer: &Vec<u8>, address: &SocketAddr) -> usize {
		self.socket.send_to(&buffer, address).unwrap()
	}
}

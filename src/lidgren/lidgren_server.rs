use crate::prelude::*;

use std::collections::HashMap;
use std::io::ErrorKind::WouldBlock;
use std::net::{SocketAddr, UdpSocket};
use std::ops::Add;
use std::time::{Duration, Instant};

use crate::lidgren::data_structures::{MESSAGE_HEADER_LENGTH, MessageHeader};
use crate::util::custom_iterator::CustomIterator;
use crate::lidgren::message_type::MessageType;
use crate::lidgren::connected_client::ConnectedClient;
use crate::lidgren::data_types::DataType;
use crate::lidgren::util::formatter as lg_formatter;

pub struct ServerInstance {
	application_name: String,
	server_unique_id: u64,
	socket: SocketWrapper,
	input_buffer: [u8; 0xFFFF],
	user_map: HashMap<SocketAddr, ConnectedClient>,
	time_run_duration: Instant,
	time_cleanup: Instant,
	pub new_data_packets: Vec<DataPacket>,
}

pub struct SocketWrapper {
	socket: UdpSocket,
}

impl SocketWrapper {
	pub fn send(&self, data: &[u8], address: &SocketAddr) {
		match self.socket.send_to(data, address) {
			Ok(number_bytes) => {
				if number_bytes != data.len() {
					log_error!("ERROR: Failed to send right amount of bytes via socket ", number_bytes, " / ", data.len());
				}
			}
			Err(e) => {
				panic!("Unexpected error while sending via socket {:?}", e);
			}
		}
	}
	
	fn receive(&mut self, input_buffer: &mut [u8]) -> Option<(usize, SocketAddr)> {
		match self.socket.recv_from(input_buffer) {
			Err(err) if err.kind() == WouldBlock => {
				None //No packet in buffer right now, skip reading.
			}
			Err(err) => {
				log_error!("Error while reading from socket: ", format!("{:?}", err));
				None //We got an error, so stop reading
			}
			Ok(value) => {
				Some(value) //All good, forward packet!
			}
		}
	}
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
	) -> EhResult<ServerInstance> {
		let socket = exception_from!(UdpSocket::bind(target), "While binding server socket")?;
		exception_from!(socket.set_nonblocking(true), "While setting socket to non-blocking mode")?;
		
		let input_buffer: [u8; 0xFFFF] = [0; 0xFFFF];
		let now = Instant::now();
		
		Ok(ServerInstance {
			socket: SocketWrapper { socket },
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
		
		//Send messages:
		for client in self.user_map.values_mut() {
			client.send_messages(&mut self.socket);
		}
		
		let start = Instant::now();
		let max_read_duration = Duration::from_millis(100);
		//Read packets until at max 100ms have passed, then the rest of the program should continue (to consume the new packets).
		while start.elapsed().lt(&max_read_duration)
		{
			match self.socket.receive(&mut self.input_buffer) {
				None => break,
				Some((amount_read, remote_address)) => self.process_packet(amount_read, remote_address),
			}
		}
	}
	
	pub fn send_to(&mut self, address: SocketAddr, data: Vec<u8>) {
		let connected_client = self.user_map.get_mut(&address).unwrap_or_else(|| {
			panic!("The user, which this packet was about to be sent to, does not exist... Highly suspicious.");
		});
		connected_client.send_to(data);
	}
	
	pub fn process_packet(&mut self, amount_read: usize, remote_address: SocketAddr) {
		log_debug!("====================================");
		log_debug!("Received UDP packet from ", remote_address.ip(), " port ", remote_address.port(), " size ", amount_read);
		
		if amount_read < MESSAGE_HEADER_LENGTH {
			//Drop packet, it cannot even hold a single Lidgren message header.
			log_warn!("Dropping packet, message header won't fit inside.");
			return;
		}
		
		let mut iterator = CustomIterator::create(&self.input_buffer[0..amount_read]);
		
		while iterator.remaining() >= MESSAGE_HEADER_LENGTH {
			let header = unwrap_or_print_return!(exception_wrap!(MessageHeader::from_stream(&mut iterator), "While constructing lidgren header"));
			log_debug!("Type: ", format!("{:x?}", header.message_type), " Fragment: ", header.fragment, " Sequence#: ", header.sequence_number, " Bits: ", header.bits, " Bytes: ", header.bytes);
			
			if (iterator.remaining() as u16) < header.bytes {
				log_warn!("Message header declared payload size bigger than rest of packet: ", header.bytes, "/", iterator.remaining());
				return;
			}
			
			if let MessageType::Unused(_) = header.message_type {
				log_warn!("Received Unused/Reserved message type. Stopping parsing.");
				return;
			}
			
			let message_data_iterator = unwrap_or_print_return!(exception_wrap!(iterator.sub_section(header.bytes as usize), "While creating message-sub-iterator"));
			
			if MessageType::is_system(&header.message_type) {
				match header.message_type {
					MessageType::Discovery =>
						ServerInstance::handle_packet_discovery(message_data_iterator, remote_address, &mut self.new_data_packets),
					MessageType::Connect =>
						ServerInstance::handle_packet_connect(message_data_iterator, remote_address, &mut self.new_data_packets, &self.application_name),
					MessageType::ConnectionEstablished =>
						ServerInstance::handle_packet_connection_established(message_data_iterator, remote_address, &mut self.user_map),
					MessageType::Ping =>
						ServerInstance::handle_packet_ping(message_data_iterator, remote_address, &self.time_run_duration, &self.socket),
					MessageType::Disconnect =>
						ServerInstance::handle_packet_disconnect(message_data_iterator, remote_address, &mut self.new_data_packets, &mut self.user_map),
					MessageType::Acknowledge =>
						ServerInstance::handle_packet_acknowledged(message_data_iterator, remote_address, &mut self.user_map),
					_ => {
						//Reject!
						log_warn!("Rejecting message type ", format!("{:?}", header.message_type), " from ", remote_address.ip(), ":", remote_address.port(), " remaining bytes ", message_data_iterator.remaining());
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
							
							self.socket.send(&result_buffer, &remote_address);
						}
						//Handle:
						if channel != 0 {
							log_warn!("Cannot handle anything but channel 0 yet!");
							return;
						}
						let connected_client = self.user_map.get_mut(&remote_address);
						let connected_client = unwrap_some_or_return!(connected_client, {
							log_warn!("Client sent user-message, while not being connected!");
						});
						connected_client.handle_new_message(
							&mut self.new_data_packets,
							remote_address,
							header,
							message_data_iterator,
						);
					}
					_ => {
						log_warn!("Unexpected/Unimplemented message type!");
						return;
					}
				};
			}
		}
		if iterator.remaining() > 0 {
			log_warn!("Dropping packet, there had been additional bytes to read that don't fit a message header. Amount ", iterator.remaining());
		}
	}
	
	fn handle_packet_discovery(mut iterator: CustomIterator, remote_address: SocketAddr, new_data_packets: &mut Vec<DataPacket>) {
		//Accept!
		new_data_packets.push(DataPacket {
			data_type: DataType::Discovery,
			remote_address,
			data: iterator.consume(),
		});
	}
	
	fn handle_packet_connect(mut iterator: CustomIterator, remote_address: SocketAddr, new_data_packets: &mut Vec<DataPacket>, application_name: &String) {
		let app_id = unwrap_or_print_return!(exception_wrap!(lg_formatter::read_string(&mut iterator), "While reading app id, in connect message"));
		if application_name.ne(&app_id) {
			log_warn!("Remote ", remote_address.ip(), ":", remote_address.port(), " sent wrong application identifier name '", app_id, "'.");
			return;
		}
		//TODO: Actually somehow use the ID? Only useful if routers actually do funky stuff...
		let _remote_id = lg_formatter::read_int_64(&mut iterator);
		let remote_time = unwrap_or_print_return!(exception_wrap!(lg_formatter::read_float(&mut iterator), "While reading the remote time"));
		log_debug!("Remote time: ", remote_time);
		new_data_packets.push(DataPacket {
			data_type: DataType::Connect,
			remote_address,
			data: iterator.consume(),
		});
	}
	
	fn handle_packet_connection_established(mut iterator: CustomIterator, remote_address: SocketAddr, user_map: &mut HashMap<SocketAddr, ConnectedClient>) {
		if iterator.remaining() != 4 {
			log_warn!("Remote ", remote_address.ip(), ":", remote_address.port(), " sent invalid connection established message, expected exactly 4 bytes, got ", iterator.remaining());
			return;
		}
		let remote_time = unwrap_or_print_return!(exception_wrap!(lg_formatter::read_float(&mut iterator), "While reading the remote time"));
		log_debug!("Remote time: ", remote_time);
		//Register user:
		user_map.insert(remote_address, ConnectedClient::new(remote_address));
	}
	
	fn handle_packet_ping(mut iterator: CustomIterator, remote_address: SocketAddr, time_run_duration: &Instant, socket: &SocketWrapper) {
		if iterator.remaining() != 1 {
			log_warn!("Remote ", remote_address.ip(), ":", remote_address.port(), " sent invalid ping message, expected exactly 1 byte, got ", iterator.remaining());
			return;
		}
		let ping_number = iterator.next_unchecked();
		log_debug!("Ping packet: ", ping_number);
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
			let elapsed_time = time_run_duration.elapsed().as_millis() as f32 / 1000.0;
			log_debug!("Time passed: ", elapsed_time);
			lg_formatter::write_float(&mut result_buffer, elapsed_time);
			
			let size = (result_buffer.len() - 5) * 8;
			result_buffer[3] = size as u8;
			result_buffer[4] = (size >> 8) as u8;
			
			socket.send(&result_buffer, &remote_address);
		}
	}
	
	fn handle_packet_disconnect(mut iterator: CustomIterator, remote_address: SocketAddr, new_data_packets: &mut Vec<DataPacket>, user_map: &mut HashMap<SocketAddr, ConnectedClient>) {
		//First disconnect the client (as in stop sending it data and clean up):
		//TODO: Maybe improve external disconnection...
		user_map.remove(&remote_address); //Brute force way to get rid of it. Deal with the aftermath later...
		let mut i = 0;
		while i < new_data_packets.len() {
			let packet: &DataPacket = new_data_packets.get(i).unwrap(); //As per condition above, we are still in bounds.
			if packet.data_type == DataType::Data && packet.remote_address == remote_address {
				new_data_packets.remove(i);
			} else {
				i += 1;
			}
		}
		//Now read the actual packet content:
		let disconnection_reason = unwrap_or_print_return!(exception_wrap!(lg_formatter::read_string(&mut iterator), "While reading disconnect reason"));
		log_warn!(">> Client disconnected with reason: '", disconnection_reason, "'");
		if iterator.has_more() {
			log_warn!("Warning Disconnect packet had more data to read: ", iterator.remaining(), " bytes");
		}
		
		//TODO: Confirm that removing the packets actually worked...
		log_debug!("Destroyed user data and (hopefully) purged all incoming packets by it.");
		//TODO: Let main application know about this (send pseudo packet).
	}
	
	fn handle_packet_acknowledged(
		mut iterator: CustomIterator,
		remote_address: SocketAddr,
		user_map: &mut HashMap<SocketAddr, ConnectedClient>
	) {
		//Get user in question:
		let connected_client = unwrap_some_or_return!(user_map.get_mut(&remote_address), {
			log_warn!("Warning: Unconnected user sent acknowledge packet - ignoring!");
		});
		//Parse acknowledge data and forward to channel handler:
		if iterator.remaining() % 3 != 0 {
			log_warn!("Warning: Connected user sent invalid acknowledge packet: Length is invalid ", iterator.remaining());
			return;
		}
		while iterator.has_more() {
			//We know, that there will be 3 more bytes available now, proceed with unchecked operations:
			let raw_id = iterator.next_unchecked();
			let message_type = unwrap_some_or_return!(MessageType::from_id(raw_id), {
				log_warn!("Warning: Connected user sent invalid acknowledge packet: Invalid message type id ", raw_id);
			});
			let sequence_number = iterator.next_unchecked() as u16 | ((iterator.next_unchecked() as u16) << 8);
			if MessageType::UserReliableOrdered(0) != message_type {
				log_warn!("Warning: Connected user sent invalid acknowledge packet: Message type, that we most certainly never sent ", format!("{:?}", message_type), " with sequence number ", sequence_number);
				continue;
			}
			log_debug!("Received acknowledge for sequence id ", sequence_number);
			connected_client.received_acknowledge(sequence_number);
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
		log_debug!("Time passed: ", elapsed_time);
		lg_formatter::write_float(&mut result_buffer, elapsed_time);
		
		let size = (result_buffer.len() - 5) * 8;
		result_buffer[3] = size as u8;
		result_buffer[4] = (size >> 8) as u8;
		
		self.socket.send(&result_buffer, remote_address);
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
		
		self.socket.send(&result_buffer, remote_address);
	}
}

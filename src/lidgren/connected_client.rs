use crate::prelude::*;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use crate::lidgren::channel_handler::reliable_ordered::ReliableOrderedHandler;
use crate::lidgren::channel_sender::reliable_ordered::ReliablyOrderedSender;
use crate::lidgren::data_structures::MessageHeader;
use crate::lidgren::data_types::DataType;
use crate::lidgren::util::formatter as lg_formatter;
use crate::util::custom_iterator::CustomIterator;
use crate::lidgren::lidgren_server::DataPacket;

pub struct ConnectedClient {
	pub remote_address: SocketAddr,
	pub channel_handler: Option<ReliableOrderedHandler>,
	fragment_map: HashMap<u32, FragmentData>,
	channel_sender: ReliablyOrderedSender,
}

impl ConnectedClient {
	pub fn new(remote_address: SocketAddr) -> ConnectedClient {
		ConnectedClient {
			remote_address,
			channel_handler: None,
			fragment_map: HashMap::new(),
			channel_sender: ReliablyOrderedSender::new(),
		}
	}
	
	pub fn send_to(&mut self, data: Vec<u8>) {
		if (data.len() + 5) > 1408 {
			//TODO: Enqueue fragmented messages...
			panic!("Packet too big to be sent: ({} + 5) / {} GOT TO IMPLEMENT FRAGMENTING!", data.len(), 1408);
		}
		self.channel_sender.enqueue_packet(data);
	}
	
	pub fn received_acknowledge(&mut self, sequence_id: u16) {
		self.channel_sender.received_acknowledge(sequence_id);
	}
	
	pub fn send_messages(&mut self, send_buffer: &mut Vec<(SocketAddr, Vec<u8>)>) {
		self.channel_sender.send_messages(&self.remote_address, send_buffer);
	}
	
	pub fn heartbeat(&mut self) {
		self.cleanup();
	}
	
	pub fn handle_new_message(&mut self,
	                          new_packets: &mut Vec<DataPacket>,
	                          address: SocketAddr,
	                          header: MessageHeader,
	                          message_data_iterator: CustomIterator,
	) {
		if self.channel_handler.is_none() {
			self.channel_handler = Some(ReliableOrderedHandler::new())
		}
		let channel = self.channel_handler.as_mut().unwrap();
		let mut output_list_to_make_rust_compiler_happy = Vec::new();
		channel.handle(header, message_data_iterator, &mut output_list_to_make_rust_compiler_happy);
		for forward_message in output_list_to_make_rust_compiler_happy {
			self.handle(new_packets, address, forward_message.0, forward_message.1);
		}
	}
	
	pub fn handle(&mut self,
	              new_packets: &mut Vec<DataPacket>,
	              address: SocketAddr,
	              header: MessageHeader,
	              data: Vec<u8>,
	) {
		if !header.fragment {
			new_packets.push(DataPacket {
				data_type: DataType::Data,
				remote_address: address,
				data,
			});
			return;
		}
		//Else we got a fragment to handle, read header:
		let mut iterator = CustomIterator::create(&data[..]);
		if iterator.remaining() < 123 {
			log_warn!("Not enough bytes to read fragment header: ", iterator.remaining(), "/", 122);
			return;
		}
		let fragment_group_id = unwrap_or_print_return!(exception_wrap!(lg_formatter::read_vint_32(&mut iterator), "While reading 'fragment_group_id'"));
		let fragment_bits = unwrap_or_print_return!(exception_wrap!(lg_formatter::read_vint_32(&mut iterator), "While reading 'fragment_bits'"));
		let fragment_chunk_size = unwrap_or_print_return!(exception_wrap!(lg_formatter::read_vint_32(&mut iterator), "While reading 'fragment_chunk_size'"));
		let fragment_index = unwrap_or_print_return!(exception_wrap!(lg_formatter::read_vint_32(&mut iterator), "While reading 'fragment_index'"));
		
		//Copy code from original:
		let _total_bytes = (fragment_bits + 7) / 8;
		let mut _total_num_of_chunks = _total_bytes / fragment_chunk_size;
		if _total_num_of_chunks * fragment_chunk_size < _total_bytes {
			_total_num_of_chunks += 1;
		}
		if fragment_index >= _total_num_of_chunks {
			log_warn!("Remote sent invalid fragment packet, index of fragment bigger than fragment count: ", fragment_index, " / ", _total_num_of_chunks);
			return;
		}
		
		let fragment_data = match self.fragment_map.entry(fragment_group_id) {
			Entry::Occupied(e) => {
				let fragment_data = e.into_mut();
				fragment_data.last_accessed_time = Instant::now(); //Update last touch time, to properly get rid of it.
				if _total_num_of_chunks != fragment_data.announced_chunk_count {
					log_warn!("Remote sent invalid fragment packet, new fragment chunk count ", _total_num_of_chunks, " does not match original ", fragment_data.announced_chunk_count);
					return;
				}
				if fragment_bits != fragment_data.announced_chunk_bits {
					log_warn!("Remote sent invalid fragment packet, new fragment chunk bit size ", fragment_bits, " does not match original ", fragment_data.announced_chunk_bits);
					return;
				}
				fragment_data
			}
			Entry::Vacant(e) => {
				log_debug!("[Fragment] Received new fragment ", fragment_group_id, " with ", _total_num_of_chunks, " chunks each ", fragment_chunk_size, " bytes (", fragment_bits, " bits).");
				
				let fragment_data = FragmentData {
					last_accessed_time: Instant::now(),
					announced_chunk_count: _total_num_of_chunks,
					announced_chunk_bits: fragment_bits,
					buffer: vec![0; _total_bytes as usize],
					chunk_checklist: vec![false; _total_num_of_chunks as usize],
					chunk_amount: 0,
				};
				e.insert(fragment_data)
			}
		};
		
		log_debug!("[Fragment] Got new fragment ", fragment_group_id, " with index ", fragment_index, " / ", _total_num_of_chunks);
		if fragment_data.chunk_checklist[fragment_index as usize] {
			log_debug!("[Fragment] -> already received!");
		} else {
			fragment_data.chunk_checklist[fragment_index as usize] = true;
			fragment_data.chunk_amount += 1;
			let buffer = &mut fragment_data.buffer[..];
			let section = &mut buffer[(fragment_index * fragment_chunk_size) as usize..]; //Starting offset until whenever...
			
			let remaining = iterator.remaining();
			if remaining > fragment_chunk_size as usize {
				//Illegal size, might blow the buffer!
				//TODO: Disconnect this malicious client!
				log_warn!("[Fragment] DANGER: Fragment does have a size bigger than the chunk size ", remaining, " / ", _total_bytes);
				return;
			}
			if fragment_index == _total_num_of_chunks - 1 {
				let expected = _total_bytes % fragment_chunk_size;
				if remaining < expected as usize {
					log_warn!("[Fragment] WARNING: Fragment does not have expected size ", remaining, " / ", expected);
				} else if remaining > expected as usize {
					//TODO: Disconnect dangerous client!
					log_warn!("[Fragment] DANGER: Last Fragment is too big ", remaining, " / ", expected);
					return;
				}
			} else {
				if remaining != fragment_chunk_size as usize {
					log_warn!("[Fragment] WARNING: Fragment does not have expected size ", remaining, " / ", fragment_chunk_size);
				}
			}
			let remaining_bytes = iterator.consume();
			section.copy_from_slice(&remaining_bytes[..]);
			
			if fragment_data.is_complete() {
				let buffer = std::mem::replace(&mut fragment_data.buffer, Vec::with_capacity(0));
				new_packets.push(DataPacket {
					data_type: DataType::Data,
					remote_address: address,
					data: buffer,
				});
			}
		}
	}
	
	pub fn cleanup(&mut self) {
		//Cleanup old fragments...
		let mut to_remove_keys = Vec::new();
		let max_time = Duration::from_millis(10000);
		for (group, data) in self.fragment_map.iter() {
			let elapsed = data.last_accessed_time.elapsed();
			if elapsed.ge(&max_time) {
				if data.is_complete() {
					log_debug!("Removing completed fragment group '", group, "' as it is older than 10 seconds. In fact: ", elapsed.as_millis());
				} else {
					//TODO: Kick the connection!
					log_warn!("Removing UNFINISHED! fragment group '", group, "' as it is older than 10 seconds. In fact: ", elapsed.as_millis());
				}
				to_remove_keys.push(*group);
			}
		}
		for group in to_remove_keys {
			self.fragment_map.remove(&group);
		}
	}
}

struct FragmentData {
	last_accessed_time: Instant,
	announced_chunk_count: u32,
	announced_chunk_bits: u32,
	buffer: Vec<u8>,
	chunk_checklist: Vec<bool>,
	chunk_amount: u32,
}

impl FragmentData {
	fn is_complete(&self) -> bool {
		self.chunk_amount == self.announced_chunk_count
	}
}
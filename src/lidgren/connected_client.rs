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
use crate::lidgren::lidgren_server::{DataPacket, SocketWrapper};

pub struct ConnectedClient {
	pub remote_address: SocketAddr,
	pub channel_handler: Option<ReliableOrderedHandler>,
	fragment_map: HashMap<u32, FragmentData>,
	channel_sender: ReliablyOrderedSender,
	fragment_group_index: u32,
}

macro_rules! vint_size {
	($a:expr) => {
		lg_formatter::vint_length($a)
	};
	($a:expr, $b:expr) => {
		lg_formatter::vint_length($a) + lg_formatter::vint_length($b)
	};
}

impl ConnectedClient {
	pub fn new(remote_address: SocketAddr) -> ConnectedClient {
		ConnectedClient {
			remote_address,
			channel_handler: None,
			fragment_map: HashMap::new(),
			channel_sender: ReliablyOrderedSender::default(),
			fragment_group_index: 1, //Just start at 1, 0 is probably possible too.
		}
	}
	
	pub fn send_to(&mut self, data: Vec<u8>) {
		if (data.len() + 5) <= 1408 {
			self.channel_sender.enqueue_packet(data, false);
			return;
		}
		
		//### Send as fragment: ####################
		
		if data.len() * 8 >= i32::MAX as usize {
			//Message says it all - if for any weird reasons we get a packet with 268MB (268.435.455 bytes) things are doomed :D
			panic!("Attempted to send a packet with way too many 'bits' - Lidgren is using signed ints internally, so the amount of bytes to be sent won't fit that - packet cannot be delivered! Size: {} (<- bytes, for bits *8)", data.len());
		}
		
		let fragment_group_index = self.fragment_group_index;
		self.fragment_group_index += 1;
		if self.fragment_group_index >= u16::MAX as u32 {
			self.fragment_group_index = 1; //Lidgren does for some reasons not allow more than 2 bytes for the index.
		}
		let fragment_total_bits = data.len() as u32 * 8;
		let constant_header_size = vint_size!(fragment_group_index, fragment_total_bits);
		let maximum_data_bytes = 1408 - (5 + constant_header_size);
		
		let (chunk_size, chunk_count) = figure_out_chunk_stuff(maximum_data_bytes, data.len() as u32);
		let mut start = 0_usize;
		let mut end = chunk_size as usize;
		
		let mut header = Vec::with_capacity(constant_header_size as usize + 5);
		lg_formatter::write_vint_32(&mut header, fragment_group_index);
		lg_formatter::write_vint_32(&mut header, fragment_total_bits);
		lg_formatter::write_vint_32(&mut header, chunk_size);
		let capacity = chunk_size as usize + header.len() + vint_size!(chunk_count) as usize;
		
		for index in 0..(chunk_count - 1) {
			let mut new_data = Vec::with_capacity(capacity);
			new_data.extend(header.iter());
			lg_formatter::write_vint_32(&mut new_data, index);
			new_data.extend(data[start..end].iter());
			start = end;
			end += chunk_size as usize;
			self.channel_sender.enqueue_packet(new_data, true);
		}
		{
			let mut new_data = Vec::with_capacity(capacity);
			new_data.extend(header.iter());
			lg_formatter::write_vint_32(&mut new_data, chunk_count - 1);
			new_data.extend(data[start..].iter());
			self.channel_sender.enqueue_packet(new_data, true);
		}
		return;
		
		macro_rules! get_chunk_amount {
			($data_bytes:expr, $chunk_size:expr) => {{
				let mut temp = $data_bytes / $chunk_size;
				if temp * $chunk_size < $data_bytes {
					temp += 1;
				}
				temp
			}}
		}
		
		fn figure_out_chunk_stuff(max_packet_bytes: u32, data_bytes: u32) -> (u32, u32) {
			let mut max_chunk_size = max_packet_bytes - 2; //2 stands for the two extra values in the fragment header, assuming they are as small as can be.
			let mut min_chunk_amount = get_chunk_amount!(data_bytes, max_chunk_size);
			
			let mut actual_header_size = vint_size!(max_chunk_size, min_chunk_amount);
			while max_packet_bytes >= (max_chunk_size + actual_header_size) {
				max_chunk_size -= 1;
				min_chunk_amount = get_chunk_amount!(data_bytes, max_chunk_size);
				actual_header_size = vint_size!(max_chunk_size, min_chunk_amount);
			}
			
			(max_chunk_size, min_chunk_amount)
		}
	}
	
	pub fn received_acknowledge(&mut self, sequence_id: u16) {
		self.channel_sender.received_acknowledge(sequence_id);
	}
	
	pub fn send_messages(&mut self, socket: &mut SocketWrapper) {
		self.channel_sender.send_messages(&self.remote_address, socket);
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
			self.channel_handler = Some(ReliableOrderedHandler::default())
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
		let mut iterator = CustomIterator::borrow(&data[..]);
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
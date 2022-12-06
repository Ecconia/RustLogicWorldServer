use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use crate::lidgren::channel_handler::reliable_ordered::ReliableOrderedHandler;
use crate::lidgren::data_structures::MessageHeader;
use crate::lidgren::util::formatter as lg_formatter;
use crate::util::custom_iterator::CustomIterator;

use crate::lidgren::lidgren_server::{PacketCallback, SendCallback};

pub struct ConnectedClient {
	pub remote_address: SocketAddr,
	pub channel_handler: Option<ReliableOrderedHandler>,
	fragment_map: HashMap<u32, FragmentData>,
}

impl ConnectedClient {
	pub fn new(remote_address: SocketAddr) -> ConnectedClient {
		ConnectedClient {
			remote_address,
			channel_handler: None,
			fragment_map: HashMap::new(),
		}
	}
	
	pub fn heartbeat(&mut self) {
		self.cleanup();
	}
	
	pub fn handle_new_message(&mut self, send_callback: SendCallback, handler: &dyn PacketCallback, header: MessageHeader, message_data_iterator: CustomIterator) {
		if self.channel_handler.is_none() {
			self.channel_handler = Some(ReliableOrderedHandler::new())
		}
		let channel = self.channel_handler.as_mut().unwrap();
		let mut output_list_to_make_rust_compiler_happy = Vec::new();
		channel.handle(header, message_data_iterator, &mut output_list_to_make_rust_compiler_happy);
		for forward_message in output_list_to_make_rust_compiler_happy {
			self.handle(&send_callback, handler, forward_message.0, forward_message.1);
		}
	}
	
	pub fn handle(&mut self, send_callback: &SendCallback, handler: &dyn PacketCallback, header: MessageHeader, data: Vec<u8>) {
		if !header.fragment {
			handler.handle_user_packet(send_callback, data);
			return;
		}
		//TODO: Eventually replace panic with something that the remote cannot trigger. Packets are meant to be dropped with a warning if invalid.
		//Else we got a fragment to handle, read header:
		let mut iterator = CustomIterator::create(&data[..]);
		if iterator.remaining() < 123 {
			panic!("Not enough bytes to read fragment header!");
		}
		let fragment_group_id = lg_formatter::read_vint_32(&mut iterator).unwrap_or_else(|message| {
			panic!("While reading 'fragment_group_id', ran out of bytes:\n-> {}", message);
		});
		let fragment_bits = lg_formatter::read_vint_32(&mut iterator).unwrap_or_else(|message| {
			panic!("While reading 'fragment_bits', ran out of bytes:\n-> {}", message);
		});
		let fragment_chunk_size = lg_formatter::read_vint_32(&mut iterator).unwrap_or_else(|message| {
			panic!("While reading 'fragment_chunk_size', ran out of bytes:\n-> {}", message);
		});
		let fragment_index = lg_formatter::read_vint_32(&mut iterator).unwrap_or_else(|message| {
			panic!("While reading 'fragment_index', ran out of bytes:\n-> {}", message);
		});
		
		//Copy code from original:
		let _total_bytes = (fragment_bits + 7) / 8;
		let mut _total_num_of_chunks = _total_bytes / fragment_chunk_size;
		if _total_num_of_chunks * fragment_chunk_size < _total_bytes {
			_total_num_of_chunks += 1;
		}
		if fragment_index >= _total_num_of_chunks {
			println!("Remote sent invalid fragment packet, index of fragment bigger than fragment count: {} / {}", fragment_index, _total_num_of_chunks);
			return;
		}
		
		let fragment_data = match self.fragment_map.entry(fragment_group_id) {
			Entry::Occupied(e) => {
				let fragment_data = e.into_mut();
				fragment_data.last_accessed_time = Instant::now(); //Update last touch time, to properly get rid of it.
				if _total_num_of_chunks != fragment_data.announced_chunk_count {
					println!("Remote sent invalid fragment packet, new fragment chunk count {} does not match original {}", _total_num_of_chunks, fragment_data.announced_chunk_count);
					return;
				}
				if fragment_bits != fragment_data.announced_chunk_bits {
					println!("Remote sent invalid fragment packet, new fragment chunk bit size {} does not match original {}", fragment_bits, fragment_data.announced_chunk_bits);
					return;
				}
				fragment_data
			}
			Entry::Vacant(e) => {
				println!("[Fragment] Received new fragment {} with {} chunks each {} bytes ({} bits).",
				         fragment_group_id, _total_num_of_chunks, fragment_chunk_size, fragment_bits);
				
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
		
		println!("[Fragment] Got new fragment {} with index {} / {}",
		         fragment_group_id, fragment_index, _total_num_of_chunks);
		if fragment_data.chunk_checklist[fragment_index as usize] {
			println!("[Fragment] -> already received!");
		} else {
			fragment_data.chunk_checklist[fragment_index as usize] = true;
			fragment_data.chunk_amount += 1;
			let buffer = &mut fragment_data.buffer[..];
			let section = &mut buffer[(fragment_index * fragment_chunk_size) as usize..]; //Starting offset until whenever...
			
			let remaining = iterator.remaining();
			if remaining > fragment_chunk_size as usize {
				//Illegal size, might blow the buffer!
				//TODO: Disconnect this malicious client!
				println!("[Fragment] DANGER: Fragment does have a size bigger than the chunk size {} / {}",
				         remaining, _total_bytes);
				return;
			}
			if fragment_index == _total_num_of_chunks - 1 {
				let expected = _total_bytes % fragment_chunk_size;
				if remaining < expected as usize {
					println!("[Fragment] WARNING: Fragment does not have expected size {} / {}",
					         remaining, expected);
				} else if remaining > expected as usize {
					//TODO: Disconnect dangerous client!
					println!("[Fragment] DANGER: Last Fragment is too big {} / {}",
					         remaining, expected);
					return;
				}
			} else {
				if remaining != fragment_chunk_size as usize {
					println!("[Fragment] WARNING: Fragment does not have expected size {} / {}",
					         remaining, fragment_chunk_size);
				}
			}
			let remaining_bytes = iterator.read_bytes(remaining).unwrap_or_else(|message| {
				panic!("Life went wrong, when draining the custom iterator...\n -> {}", message);
			});
			section.copy_from_slice(&remaining_bytes[..]);
			
			if fragment_data.is_complete() {
				let buffer = std::mem::replace(&mut fragment_data.buffer, Vec::with_capacity(0));
				handler.handle_user_packet(send_callback, buffer);
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
					println!("Removing completed fragment group '{}' as it is older than 10 seconds. In fact: {}", group, elapsed.as_millis());
				} else {
					//TODO: Kick the connection!
					println!("Removing UNFINISHED! fragment group '{}' as it is older than 10 seconds. In fact: {}", group, elapsed.as_millis());
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
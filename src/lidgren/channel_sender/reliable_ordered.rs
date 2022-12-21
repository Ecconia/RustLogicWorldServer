use crate::prelude::*;

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use crate::lidgren::lidgren_server::SocketWrapper;
use crate::lidgren::message_type::MessageType;

const WINDOW_SIZE: usize = 64;
const SEQUENCE_NUMBERS: usize = 1024;
//TODO: Use ping time + 0.025...
const TIME_BETWEEN_RESENDS: Duration = Duration::from_millis(400);

pub struct ReliablyOrderedSender {
	packet_queue: VecDeque<(Vec<u8>, bool)>,
	message_buffer: [Option<EnqueuedMessage>; WINDOW_SIZE],
	buffer_oldest: u16,
	buffer_latest: u16,
}

pub struct EnqueuedMessage {
	data: Vec<u8>,
	last_sent: Instant,
	sent_count: u32,
	acknowledged: bool,
}

impl ReliablyOrderedSender {
	pub fn new() -> ReliablyOrderedSender {
		const INIT: Option<EnqueuedMessage> = None;
		ReliablyOrderedSender {
			packet_queue: VecDeque::new(),
			message_buffer: [INIT; WINDOW_SIZE],
			buffer_oldest: 0,
			buffer_latest: 0,
		}
	}
	
	pub fn enqueue_packet(&mut self, data: Vec<u8>, is_fragment: bool) {
		log_debug!("Enqueued packet with ", data.len(), " bytes");
		self.packet_queue.push_back((data, is_fragment));
	}
	
	pub fn send_messages(&mut self, address: &SocketAddr, socket: &mut SocketWrapper) {
		for buffered_message in self.message_buffer.iter_mut().flatten() {
			if !buffered_message.acknowledged && buffered_message.last_sent.elapsed().gt(&TIME_BETWEEN_RESENDS) {
				socket.send(&buffered_message.data[..], address);
				buffered_message.last_sent = Instant::now();
				buffered_message.sent_count += 1;
			}
		}
		
		//Queue more message if room for them:
		let mut space_to_fill = self.get_free_buffer_slots();
		while space_to_fill > 0 && !self.packet_queue.is_empty() {
			log_debug!("+++ Got packet to send! ++++++++++++++++");
			//Bytes to send:
			let (data, is_fragment) = self.packet_queue.pop_front().unwrap(); //There should be no reason for this to be 'None' as it is not empty.
			
			//Next free sequence number of this packet:
			let sequence_number = self.buffer_latest;
			self.buffer_latest = (self.buffer_latest + 1) % SEQUENCE_NUMBERS as u16; //Increment the sequence number.
			
			let buffer_index = sequence_number as usize % WINDOW_SIZE;
			//Sanity check:
			if self.message_buffer[buffer_index].is_some() {
				panic!("'Free' buffer position was not free!");
			}
			
			log_debug!("Packet is ", data.len(), " bytes");
			let mut packet_bytes = Vec::with_capacity(5 + data.len());
			packet_bytes.push(MessageType::UserReliableOrdered(0).to_index());
			packet_bytes.push((sequence_number << 1) as u8 | is_fragment as u8);
			packet_bytes.push((sequence_number >> 7) as u8);
			let length = data.len() * 8;
			packet_bytes.push(length as u8);
			packet_bytes.push((length >> 8) as u8);
			packet_bytes.extend(data);
			log_debug!("Yielding in ", packet_bytes.len(), " bytes");
			
			socket.send(&packet_bytes[..], address); //Copy before storing it, else overhead starts...
			self.message_buffer[buffer_index] = Some(EnqueuedMessage {
				data: packet_bytes,
				last_sent: Instant::now(),
				sent_count: 1, //Already "has been sent" one time by now
				acknowledged: false,
			});
			
			space_to_fill -= 1;
		}
	}
	
	pub fn received_acknowledge(&mut self, sequence_number: u16) {
		//TODO: Add handling, to reset the timeout, if the acknowledge arrived asap.
		let relative_sequence_number = ReliablyOrderedSender::create_relative_index(sequence_number, self.buffer_oldest);
		
		if relative_sequence_number < 0 {
			//We already had this packet acknowledged, ignore it.
			return;
		}
		
		if relative_sequence_number == 0 {
			//Acknowledge happens in time, oldest message got confirmed!
			let mut buffered_index = self.buffer_oldest as usize % WINDOW_SIZE;
			if self.message_buffer[buffered_index].is_none() {
				log_warn!("The client sent an acknowledge packet for the current head of packets, yet there was no packet in the buffer - the acknowledge packet came unexpected.");
				return;
			}
			
			//Delete the current packet:
			self.message_buffer[buffered_index] = None;
			self.buffer_oldest = (self.buffer_oldest + 1) % SEQUENCE_NUMBERS as u16;
			//Prepare for next iteration:
			buffered_index = self.buffer_oldest as usize % WINDOW_SIZE;
			
			//Remove all directly following buffered messages from now that got acknowledged.
			while self.message_buffer[buffered_index].is_some() && self.message_buffer[buffered_index].as_ref().unwrap().acknowledged {
				self.message_buffer[buffered_index] = None;
				self.buffer_oldest = (self.buffer_oldest + 1) % SEQUENCE_NUMBERS as u16;
				//Prepare for next iteration:
				buffered_index = self.buffer_oldest as usize % WINDOW_SIZE;
			}
		}
		
		let relative_sequence_number = ReliablyOrderedSender::create_relative_index(sequence_number, self.buffer_latest);
		
		if relative_sequence_number <= 0 {
			//Regardless if a packet was acknowledged or not, lets set the acknowledge flag:
			if let Some(packet) = &mut self.message_buffer[sequence_number as usize % WINDOW_SIZE] {
				packet.acknowledged = true;
			}
		} else {
			log_warn!("Warning: Received acknowledge too early...");
		}
	}
	
	fn create_relative_index(sequence_number: u16, relative_offset: u16) -> i16 {
		(sequence_number as i16 - relative_offset as i16 + SEQUENCE_NUMBERS as i16 + (SEQUENCE_NUMBERS as i16 / 2)) % SEQUENCE_NUMBERS as i16 - (SEQUENCE_NUMBERS as i16 / 2)
	}
	
	fn get_free_buffer_slots(&self) -> u8 {
		(WINDOW_SIZE - (self.buffer_latest as usize + SEQUENCE_NUMBERS - self.buffer_oldest as usize) % SEQUENCE_NUMBERS) as u8
	}
}

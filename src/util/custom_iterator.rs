use crate::prelude::*;

pub struct CustomIterator<'a> {
	buffer: &'a [u8],
	pointer: usize,
}

impl<'a> CustomIterator<'a> {
	pub fn create(buffer: &[u8]) -> CustomIterator {
		CustomIterator {
			buffer,
			pointer: 0,
		}
	}
	
	pub fn next_unchecked(&mut self) -> u8 {
		let value = self.buffer[self.pointer];
		self.pointer += 1;
		value
	}
	
	pub fn next(&mut self) -> EhResult<u8> {
		if self.pointer >= self.buffer.len() {
			return exception!("Expected more bytes while reading byte, but reached ", self.pointer, "/", self.buffer.len());
		}
		let value = self.buffer[self.pointer];
		self.pointer += 1;
		Ok(value)
	}
	
	pub fn peek_unchecked(&self) -> u8 {
		self.buffer[self.pointer]
	}
	
	pub fn peek(&self) -> EhResult<u8> {
		if self.pointer >= self.buffer.len() {
			return exception!("Expected more bytes while peeking, but reached ", self.pointer, "/", self.buffer.len());
		}
		Ok(self.buffer[self.pointer])
	}
	
	pub fn remaining(&self) -> usize {
		if self.pointer > self.buffer.len() {
			0
		} else {
			self.buffer.len() - self.pointer
		}
	}
	
	pub fn sub_section(&mut self, amount: usize) -> EhResult<CustomIterator> {
		let target_position = self.pointer + amount;
		if target_position > self.buffer.len() {
			return exception!("Expected more bytes while creating sub iterator, but reached (", self.pointer, "+", amount, ")/", self.buffer.len());
		}
		let sub_iterator = CustomIterator::create(
			&self.buffer[self.pointer..target_position],
		);
		self.pointer += amount;
		Ok(sub_iterator)
	}
	
	pub fn read_bytes(&mut self, amount: usize) -> EhResult<Vec<u8>> {
		//If the iterator is exhausted draining it, might cause a call with 0 as amount, then just return an empty vector.
		if amount == 0 {
			return Ok(Vec::new());
		}
		let target_position = self.pointer + amount;
		if target_position > self.buffer.len() {
			return exception!("Expected more bytes while reading bytes, but reached (", self.pointer, "+", amount, ")/", self.buffer.len());
		}
		let result = self.buffer[self.pointer..target_position].to_vec();
		self.pointer += amount;
		Ok(result)
	}
	
	pub fn consume(&mut self) -> Vec<u8> {
		if self.pointer > self.buffer.len() {
			return Vec::new(); //We are already out of bounds - however this might have happened.
		}
		//Else return whatever remains:
		let result = self.buffer[self.pointer..].to_vec();
		self.pointer = self.buffer.len(); //And set the point to the end of the buffer.
		result
	}
	
	pub fn has_more(&self) -> bool {
		self.pointer < self.buffer.len()
	}
	
	pub fn skip(&mut self) {
		self.pointer += 1;
	}
}

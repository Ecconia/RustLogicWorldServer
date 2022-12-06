pub struct CustomIterator<'a> {
	buffer: &'a [u8],
	pointer: usize,
}

impl<'a> CustomIterator<'a> {
	pub fn create(buffer: &[u8]) -> CustomIterator {
		return CustomIterator {
			buffer,
			pointer: 0,
		};
	}
	
	pub fn next_unchecked(&mut self) -> u8 {
		let value = self.buffer[self.pointer];
		self.pointer += 1;
		return value;
	}
	
	pub fn next(&mut self) -> Result<u8, String> {
		if self.pointer >= self.buffer.len() {
			return Err(format!("Expected more bytes while reading, but reached {}/{}", self.pointer, self.buffer.len()));
		}
		let value = self.buffer[self.pointer];
		self.pointer += 1;
		return Ok(value);
	}
	
	pub fn peek_unchecked(&self) -> u8 {
		return self.buffer[self.pointer];
	}
	
	pub fn peek(&self) -> Result<u8, String> {
		if self.pointer >= self.buffer.len() {
			return Err(format!("Expected more bytes while peeking, but reached {}/{}", self.pointer, self.buffer.len()));
		}
		return Ok(self.buffer[self.pointer]);
	}
	
	pub fn remaining(&self) -> usize {
		return if self.pointer > self.buffer.len() {
			0
		} else {
			self.buffer.len() - self.pointer
		};
	}
	
	pub fn sub_section(&mut self, amount: usize) -> Result<CustomIterator, String> {
		let target_position = self.pointer + amount;
		if target_position > self.buffer.len() {
			return Err(format!("Expected more bytes while creating sub iterator, but reached ({}+{})/{}", self.pointer, amount, self.buffer.len()));
		}
		let sub_iterator = CustomIterator::create(
			&self.buffer[self.pointer..target_position],
		);
		self.pointer += amount;
		return Ok(sub_iterator);
	}
	
	pub fn read_bytes(&mut self, amount: usize) -> Result<Vec<u8>, String> {
		//If the iterator is exhausted draining it, might cause a call with 0 as amount, then just return an empty vector.
		if amount == 0 {
			return Ok(Vec::new());
		}
		let target_position = self.pointer + amount;
		if target_position > self.buffer.len() {
			return Err(format!("Expected more bytes while reading bytes, but reached ({}+{})/{}", self.pointer, amount, self.buffer.len()));
		}
		let result = self.buffer[self.pointer..target_position].to_vec();
		self.pointer += amount;
		return Ok(result);
	}
	
	pub fn consume(&mut self) -> Vec<u8> {
		if self.pointer > self.buffer.len() {
			return Vec::new(); //We are already out of bounds - however this might have happened.
		}
		//Else return whatever remains:
		let result = self.buffer[self.pointer..].to_vec();
		self.pointer = self.buffer.len(); //And set the point to the end of the buffer.
		return result;
	}
	
	pub fn has_more(&self) -> bool {
		return self.pointer < self.buffer.len();
	}
	
	pub fn skip(&mut self) {
		self.pointer += 1;
	}
}

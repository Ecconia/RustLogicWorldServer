use crate::prelude::*;

trait DataStorage {
	fn len(&self) -> usize;
	fn get(&self, index: usize) -> u8;
	fn get_range(&self, start: usize, end: usize) -> &[u8];
}

struct DataBorrower<'a> {
	buffer: &'a [u8],
}

impl<'a> DataStorage for DataBorrower<'a> {
	fn len(&self) -> usize {
		self.buffer.len()
	}

	fn get(&self, index: usize) -> u8 {
		self.buffer[index]
	}

	fn get_range(&self, start: usize, end: usize) -> &[u8] {
		&self.buffer[start..end]
	}
}

impl<'a> DataBorrower<'a> {
	fn new(buffer: &'a [u8]) -> Box<dyn DataStorage + 'a> {
		Box::new(DataBorrower {
			buffer,
		})
	}
}

struct DataOwner {
	buffer: Vec<u8>,
}

impl DataOwner {
	fn new(buffer: Vec<u8>) -> Box<dyn DataStorage> {
		Box::new(DataOwner {
			buffer,
		})
	}
}

impl DataStorage for DataOwner {
	fn len(&self) -> usize {
		self.buffer.len()
	}

	fn get(&self, index: usize) -> u8 {
		self.buffer[index]
	}

	fn get_range(&self, start: usize, end: usize) -> &[u8] {
		&self.buffer[start..end]
	}
}

pub struct CustomIterator<'a> {
	buffer: Box<dyn DataStorage + 'a>,
	pointer: usize,
}

//Main iterator functionality:
impl<'a> CustomIterator<'a> {
	fn new(buffer: Box<dyn DataStorage + 'a>) -> CustomIterator<'a> {
		CustomIterator {
			buffer,
			pointer: 0,
		}
	}
	
	pub fn own(buffer: Vec<u8>) -> CustomIterator<'a> {
		CustomIterator::new(DataOwner::new(buffer))
	}
	
	pub fn borrow(buffer: &'a [u8]) -> CustomIterator<'a> {
		CustomIterator::new(DataBorrower::new(buffer))
	}
	
	pub fn next_unchecked(&mut self) -> u8 {
		let value = self.buffer.get(self.pointer);
		self.pointer += 1;
		value
	}
	
	pub fn next(&mut self) -> EhResult<u8> {
		if self.pointer >= self.buffer.len() {
			return exception!("Expected more bytes while reading byte, but reached ", self.pointer, "/", self.buffer.len());
		}
		let value = self.buffer.get(self.pointer);
		self.pointer += 1;
		Ok(value)
	}
	
	pub fn peek_unchecked(&self) -> u8 {
		self.buffer.get(self.pointer)
	}
	
	pub fn peek(&self) -> EhResult<u8> {
		if self.pointer >= self.buffer.len() {
			return exception!("Expected more bytes while peeking, but reached ", self.pointer, "/", self.buffer.len());
		}
		Ok(self.buffer.get(self.pointer))
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
		let sub_iterator = CustomIterator::borrow(
			&self.buffer.get_range(self.pointer, target_position),
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
		let result = self.buffer.get_range(self.pointer, target_position).to_vec();
		self.pointer += amount;
		Ok(result)
	}
	
	pub fn read_slice_unchecked(&mut self, amount: usize) -> &[u8] {
		//If the iterator is exhausted draining it, might cause a call with 0 as amount, then just return an empty vector.
		if amount == 0 {
			return &[];
		}
		let target_position = self.pointer + amount;
		let result = &self.buffer.get_range(self.pointer, target_position); //Will panic, if someone used this method without first checking the amount.
		self.pointer += amount;
		result
	}
	
	pub fn consume(&mut self) -> Vec<u8> {
		if self.pointer > self.buffer.len() {
			return Vec::new(); //We are already out of bounds - however this might have happened.
		}
		//Else return whatever remains:
		let result = self.buffer.get_range(self.pointer, self.buffer.len()).to_vec();
		self.pointer = self.buffer.len(); //And set the point to the end of the buffer.
		result
	}
	
	pub fn has_more(&self) -> bool {
		self.pointer < self.buffer.len()
	}
	
	pub fn skip(&mut self) {
		self.pointer += 1;
	}
	
	//In case that one has to go back, these two methods allow to store and restore the position. Never use for anything else!
	
	pub fn pointer_save(&self) -> usize {
		self.pointer
	}
	
	pub fn pointer_restore(&mut self, pointer: usize) {
		self.pointer = pointer;
	}
}

//Byte reader implementations:
impl<'a> CustomIterator<'a> {
	//### Big endian: ###########
	
	pub fn read_be_u64(&mut self) -> EhResult<u64> {
		if self.remaining() < 8 {
			return exception!("While reading ", "BE u64", " ran out of bytes: ", self.remaining(), "/", "8");
		}
		Ok(u64::from_be_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
	
	pub fn read_be_u32(&mut self) -> EhResult<u32> {
		if self.remaining() < 4 {
			return exception!("While reading ", "BE u32", " ran out of bytes: ", self.remaining(), "/", "4");
		}
		Ok(u32::from_be_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
	
	pub fn read_be_u16(&mut self) -> EhResult<u16> {
		if self.remaining() < 2 {
			return exception!("While reading ", "BE u16", " ran out of bytes: ", self.remaining(), "/", "2");
		}
		Ok(u16::from_be_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
	
	pub fn read_be_i64(&mut self) -> EhResult<i64> {
		if self.remaining() < 8 {
			return exception!("While reading ", "BE i64", " ran out of bytes: ", self.remaining(), "/", "8");
		}
		Ok(i64::from_be_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
	
	pub fn read_be_i32(&mut self) -> EhResult<i32> {
		if self.remaining() < 4 {
			return exception!("While reading ", "BE i32", " ran out of bytes: ", self.remaining(), "/", "4");
		}
		Ok(i32::from_be_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
	
	pub fn read_be_i16(&mut self) -> EhResult<i16> {
		if self.remaining() < 2 {
			return exception!("While reading ", "BE i16", " ran out of bytes: ", self.remaining(), "/", "2");
		}
		Ok(i16::from_be_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
	
	pub fn read_be_f64(&mut self) -> EhResult<f64> {
		if self.remaining() < 8 {
			return exception!("While reading ", "BE f64", " ran out of bytes: ", self.remaining(), "/", "8");
		}
		Ok(f64::from_be_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
	
	pub fn read_be_f32(&mut self) -> EhResult<f32> {
		if self.remaining() < 4 {
			return exception!("While reading ", "BE f32", " ran out of bytes: ", self.remaining(), "/", "4");
		}
		Ok(f32::from_be_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
	
	//### Little endian: #########
	
	pub fn read_le_u64(&mut self) -> EhResult<u64> {
		if self.remaining() < 8 {
			return exception!("While reading ", "LE u64", " ran out of bytes: ", self.remaining(), "/", "8");
		}
		Ok(u64::from_le_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
	
	pub fn read_le_u32(&mut self) -> EhResult<u32> {
		if self.remaining() < 4 {
			return exception!("While reading ", "LE u32", " ran out of bytes: ", self.remaining(), "/", "4");
		}
		Ok(u32::from_le_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
	
	pub fn read_le_u16(&mut self) -> EhResult<u16> {
		if self.remaining() < 2 {
			return exception!("While reading ", "LE u16", " ran out of bytes: ", self.remaining(), "/", "2");
		}
		Ok(u16::from_le_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
	
	pub fn read_le_i64(&mut self) -> EhResult<i64> {
		if self.remaining() < 8 {
			return exception!("While reading ", "LE i64", " ran out of bytes: ", self.remaining(), "/", "8");
		}
		Ok(i64::from_le_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
	
	pub fn read_le_i32(&mut self) -> EhResult<i32> {
		if self.remaining() < 4 {
			return exception!("While reading ", "LE i32", " ran out of bytes: ", self.remaining(), "/", "4");
		}
		Ok(i32::from_le_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
	
	pub fn read_le_i16(&mut self) -> EhResult<i16> {
		if self.remaining() < 2 {
			return exception!("While reading ", "LE i16", " ran out of bytes: ", self.remaining(), "/", "2");
		}
		Ok(i16::from_le_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
	
	pub fn read_le_f64(&mut self) -> EhResult<f64> {
		if self.remaining() < 8 {
			return exception!("While reading ", "LE f64", " ran out of bytes: ", self.remaining(), "/", "8");
		}
		Ok(f64::from_le_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
	
	pub fn read_le_f32(&mut self) -> EhResult<f32> {
		if self.remaining() < 4 {
			return exception!("While reading ", "LE f32", " ran out of bytes: ", self.remaining(), "/", "4");
		}
		Ok(f32::from_le_bytes([
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
			self.next_unchecked(),
		]))
	}
}

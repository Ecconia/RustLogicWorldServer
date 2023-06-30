use crate::prelude::*;

// Note this could also be implemented for Vec
trait SliceExtension {
	fn next_unchecked(&mut self) -> u8 {
		self.next().unwrap()
	}
	fn next(&mut self) -> EhResult<u8>;

	#[deprecated(note = "just index the slice")]
	fn peek_unchecked(&mut self) -> u8 {
		self.peek().unwrap()
	}
	fn peek(&self) -> EhResult<u8>;

	#[deprecated(note = "use .len() instead")]
	fn remaining(&self) -> usize;

	#[deprecated(note = "get a slice of the vec/slice instead")]
	fn sub_section(&mut self, amount: usize) -> EhResult<&[u8]>;

	/// NOTE: should this really allocate?
	fn read_bytes(&mut self, amount: usize) -> EhResult<Vec<u8>>;

	fn read_bytes_const<const AMOUNT: usize>(&mut self) -> EhResult<[u8; AMOUNT]> {
		self.read_bytes(AMOUNT).map(|v| v.try_into().unwrap())
	}

	fn read_slice_unchecked(&mut self, amount: usize) -> &[u8];

	fn has_more(&self) -> bool;

	fn skip(&mut self) {
		let _ = self.next();
	}

	fn pointer_save(&self) -> usize {
		unreachable!();
	}

	// can maybe be implemented with unsafe
	fn pointer_restore(&mut self, pointer: usize) {
		unreachable!();
	}
}

impl SliceExtension for &[u8] {
	fn next(&mut self) -> EhResult<u8> {
		self.split_first()
			.map(|(first, remaining)| {
				*self = remaining;
				*first
			})
			.ok_or_else(|| ExceptionDetails::new(""))
	}

	fn peek(&self) -> EhResult<u8> {
		self.get(0).copied().ok_or_else(|| ExceptionDetails::new(""))
	}

	fn remaining(&self) -> usize {
		self.len()
	}

	fn sub_section(&mut self, amount: usize) -> EhResult<&[u8]> {
		self.get(0..amount).ok_or_else(|| ExceptionDetails::new(""))
	}

	fn read_bytes(&mut self, amount: usize) -> EhResult<Vec<u8>> {
		let (first, last) = self.split_at(amount);
		*self = last;
		Ok(first.to_vec())
	}

	fn read_slice_unchecked(&mut self, amount: usize) -> &[u8] {
		&self[0..amount]
	}

	fn has_more(&self) -> bool {
		self.len() > 0
	}
}

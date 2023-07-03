use crate::prelude::*;

use crate::util::custom_iterator::CustomIterator;
use crate::network::message_pack::reader as mp_reader;

pub fn try_decompress(iterator: &mut CustomIterator) -> EhResult<Option<Vec<u8>>> {
	let iterator_position = iterator.pointer_save();
	let res = try_decompress_inner(iterator).wrap(ex!("While trying to decompress"))?;
	iterator.pointer_restore(iterator_position);
	let ret = unwrap_some_or_return!(res, {
		Ok(None)
	});
	Ok(Some(ret))
}

fn try_decompress_inner(iterator: &mut CustomIterator) -> EhResult<Option<Vec<u8>>> {
	let array_size = unwrap_some_or_return!(mp_reader::try_array(iterator).wrap(ex!("While probing for compression array"))?, {
		Ok(None)
	});
	//There have to be at least two elements, one header (ext) and one chunk:
	if array_size < 2 {
		return Ok(None);
	}
	//Subtract the header of the array size to get the chunk count:
	let whatever_ext_content = unwrap_some_or_return!(mp_reader::try_ext(iterator).wrap(ex!("While probing for compression extension"))?, {
		Ok(None)
	});
	if whatever_ext_content.0 != 98 {
		return Ok(None);
	}
	let chunk_count = array_size as usize - 1;
	let extra_bytes_iterator = &mut CustomIterator::borrow(&whatever_ext_content.1);
	let mut list_of_uncompressed_chunk_sizes = Vec::with_capacity(chunk_count);
	let mut total_uncompressed_bytes = 0usize;
	for _ in 0..chunk_count {
		let uncompressed_bytes = mp_reader::read_u64(extra_bytes_iterator).wrap(ex!("While reading chunk uncompressed size"))? as usize;
		list_of_uncompressed_chunk_sizes.push(uncompressed_bytes);
		total_uncompressed_bytes += uncompressed_bytes;
	}
	if extra_bytes_iterator.has_more() {
		return exception!(format!("Expected extra bytes of decompression section to contain a single number, but have bytes remaining: {}", extra_bytes_iterator.remaining()));
	}
	
	let mut uncompressed_bytes = Vec::<u8>::with_capacity(total_uncompressed_bytes);
	// let mut pointer = 0;
	for uncompressed_chunk_size in list_of_uncompressed_chunk_sizes {
		let compressed_chunk_bytes = mp_reader::read_bytes(iterator).wrap(ex!("While reading chunk compressed bytes"))?;
		let mut uncompressed_chunk_bytes = vec![0; uncompressed_chunk_size];
		
		//Uncompress bytes:
		let actually_read = lz4::block::decompress_to_buffer(
			&compressed_chunk_bytes, 
			Some(uncompressed_chunk_size as i32),
			 &mut uncompressed_chunk_bytes
		).map_ex(ex!("While trying to decompress LZ4 block"))?;
		
		if actually_read != uncompressed_chunk_bytes.len() {
			//Better safe than sorry.
			return exception!(format!("The expected amount of bytes it not in the list: {} / {}", uncompressed_chunk_bytes.len(), actually_read));
		}
		//Check that the expected amount of bytes got received:
		if actually_read != uncompressed_chunk_size {
			return exception!(format!("Unexpected decompressed byte amount: {} / {}", actually_read, uncompressed_chunk_size));
		}
		//Collect in final data:
		uncompressed_bytes.extend(uncompressed_chunk_bytes);
	}
	
	Ok(Some(uncompressed_bytes))
}

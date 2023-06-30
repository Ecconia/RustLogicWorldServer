use crate::network::message_pack::writer as mp_writer;
use crate::network::packets::packet_ids::PacketIDs;

pub struct ExtraDataUpdate<'a> {
	pub key: String,
	pub data_type: &'a str,
	pub data: Vec<u8>,
}

impl<'a> ExtraDataUpdate<'a> {
	pub fn write(&self, buffer: &mut Vec<u8>) {
		//Version:
		mp_writer::write_int_auto(buffer, PacketIDs::ExtraDataUpdate.id());
		
		//Data:
		mp_writer::write_array_auto(buffer, 3);
		mp_writer::write_string_auto(buffer, Some(&self.key));
		mp_writer::write_string_auto(buffer, Some(self.data_type));
		mp_writer::write_binary(buffer, &self.data);
	}
}

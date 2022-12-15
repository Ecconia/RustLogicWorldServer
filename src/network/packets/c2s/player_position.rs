use crate::prelude::*;
use crate::network::packets::packet_tools::*;

use crate::network::message_pack::reader as mp_reader;
use crate::network::packets::packet_ids::PacketIDs;
use crate::util::custom_iterator::CustomIterator;

pub struct PlayerPosition {
}

impl PlayerPosition {
	pub fn parse(iterator: &mut CustomIterator) -> EhResult<PlayerPosition> {
		let packet_id = exception_wrap!(mp_reader::read_u32(iterator), "While reading PlayerPosition packet's id")?;
		if packet_id != PacketIDs::PlayerPosition.id() {
			return exception!("PlayerPosition packet has wrong packet id: ", packet_id);
		}
		
		expect_array!(iterator, "PlayerPosition", "main content", 1);
		//PlayerPositionData:
		{
			expect_array!(iterator, "PlayerPosition", "player position data", 7);
			//Quaternion : BaseWorldRotation
			{
				expect_array!(iterator, "PlayerPosition", "BaseWorldRotation:Quaternion", 4);
				//XYZA
				exception_wrap!(mp_reader::read_f32(iterator), "While reading PlayerPosition packet's BaseWorldRotation/X")?;
				exception_wrap!(mp_reader::read_f32(iterator), "While reading PlayerPosition packet's BaseWorldRotation/Y")?;
				exception_wrap!(mp_reader::read_f32(iterator), "While reading PlayerPosition packet's BaseWorldRotation/Z")?;
				exception_wrap!(mp_reader::read_f32(iterator), "While reading PlayerPosition packet's BaseWorldRotation/A")?;
			}
			//Vector : FeetPosition
			{
				expect_array!(iterator, "PlayerPosition", "FeetPosition:Vector", 3);
				//XYZ
				exception_wrap!(mp_reader::read_f32(iterator), "While reading PlayerPosition packet's FeetPosition/X")?;
				exception_wrap!(mp_reader::read_f32(iterator), "While reading PlayerPosition packet's FeetPosition/Y")?;
				exception_wrap!(mp_reader::read_f32(iterator), "While reading PlayerPosition packet's FeetPosition/Z")?;
			}
			//float : HeadHorizontalRotation
			exception_wrap!(mp_reader::read_f32(iterator), "While reading PlayerPosition packet's HeadHorizontalRotation")?;
			//float : HeadVerticalRotation
			exception_wrap!(mp_reader::read_f32(iterator), "While reading PlayerPosition packet's HeadVerticalRotation")?;
			//float : Scale
			exception_wrap!(mp_reader::read_f32(iterator), "While reading PlayerPosition packet's Scale")?;
			//bool : Flying
			exception_wrap!(mp_reader::read_bool(iterator), "While reading PlayerPosition packet's Flying")?;
			//bool : Teleport To This Position
			exception_wrap!(mp_reader::read_bool(iterator), "While reading PlayerPosition packet's TeleportToThisPosition")?;
		}
		
		expect_end_of_packet!(iterator, "PlayerPosition");
		
		Ok(PlayerPosition {})
	}
}

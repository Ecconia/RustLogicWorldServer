use crate::prelude::*;
use crate::network::packets::packet_tools::*;

use crate::network::message_pack::reader as mp_reader;
use crate::network::packets::packet_ids::PacketIDs;
use crate::util::custom_iterator::CustomIterator;

pub struct PlayerPosition {
}

impl PlayerPosition {
	pub fn validate_packet_id(iterator: &mut CustomIterator) -> EhResult<()>{
		expect_packet_id!(iterator, "player position", PacketIDs::PlayerPosition);
		Ok(())
	}
	
	pub fn parse(mut iterator: CustomIterator) -> EhResult<PlayerPosition> {
		let iterator = &mut iterator;
		
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

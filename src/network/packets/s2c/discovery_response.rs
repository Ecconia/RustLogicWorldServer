use crate::network::message_pack::writer as mp_writer;

pub struct DiscoveryResponse
{
	pub version: String,
	pub request_uid: String,
	pub has_discovery_info: bool,
	pub challenge: Option<String>,
	pub server_list_caption: String,
	pub connected_players_count: u32,
	pub max_player_amount: u32,
	pub requires_password: bool,
	pub requires_verification: bool,
}

impl DiscoveryResponse
{
	pub fn simple(uid: String, max_player_amount: u32, requires_password: bool, requires_verification: bool) -> DiscoveryResponse
	{
		DiscoveryResponse {
			version: String::from("0.91.0.485"),
			request_uid: uid,
			has_discovery_info: true,
			challenge: None,
			server_list_caption: String::from("Rust server does NOT welcome you :)"),
			connected_players_count: 0,
			max_player_amount,
			requires_password,
			requires_verification,
		}
	}
	
	pub fn write(&self, buffer: &mut Vec<u8>)
	{
		//Version:
		mp_writer::write_int_auto(buffer, 13);
		
		//Data:
		mp_writer::write_map_auto(buffer, 9);
		mp_writer::write_string_auto(buffer, Some("ServerVersion"));
		mp_writer::write_string_auto(buffer, Some(&self.version));
		mp_writer::write_string_auto(buffer, Some("RequestGuid"));
		mp_writer::write_string_auto(buffer, Some(&self.request_uid));
		mp_writer::write_string_auto(buffer, Some("HasDiscoveryInfo"));
		mp_writer::write_bool(buffer, self.has_discovery_info);
		mp_writer::write_string_auto(buffer, Some("Challenge"));
		mp_writer::write_string_auto(buffer, if self.challenge.is_none() { None } else { Some(self.challenge.as_ref().unwrap()) });
		mp_writer::write_string_auto(buffer, Some("MOTD"));
		mp_writer::write_string_auto(buffer, Some(&self.server_list_caption));
		mp_writer::write_string_auto(buffer, Some("PlayersConnectedCount"));
		mp_writer::write_int_auto(buffer, self.connected_players_count);
		mp_writer::write_string_auto(buffer, Some("MaxPlayerCapacity"));
		mp_writer::write_int_auto(buffer, self.max_player_amount);
		mp_writer::write_string_auto(buffer, Some("ConnectionRequiresPassword"));
		mp_writer::write_bool(buffer, self.requires_password);
		mp_writer::write_string_auto(buffer, Some("ServerRunningInVerifiedMode"));
		mp_writer::write_bool(buffer, self.requires_verification);
	}
}

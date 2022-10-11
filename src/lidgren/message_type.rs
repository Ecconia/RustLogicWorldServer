use MessageType::*;

#[derive(Debug)]
pub enum MessageType
{
	Unconnected,
	UserUnreliable,
	//Starting at 2:
	UserSequenced(u8),
	UserReliableUnordered,
	//Starting at 35:
	UserReliableSequenced(u8),
	//Starting at 67:
	UserReliableOrdered(u8),
	
	//Starting at 99:
	Unused(u8),
	
	LibraryError,
	Ping,
	Pong,
	Connect,
	ConnectResponse,
	ConnectionEstablished,
	Acknowledge,
	Disconnect,
	Discovery,
	DiscoveryResponse,
	NatPunchMessage,
	NatIntroduction,
	NatIntroductionConfirmRequest,
	NatIntroductionConfirmed,
	ExpandMTURequest,
	ExpandMTUSuccess,
}

impl MessageType
{
	pub fn from_id(id: u8) -> Option<MessageType>
	{
		match id {
			0 => Some(Unconnected),
			1 => Some(UserUnreliable),
			2..=33 => Some(UserSequenced(id - 2)),
			34 => Some(UserReliableUnordered),
			35..=66 => Some(UserReliableSequenced(id - 35)),
			67..=98 => Some(UserReliableOrdered(id - 67)),
			99..=127 => Some(Unused(id - 99)),
			128 => Some(LibraryError),
			129 => Some(Ping),
			130 => Some(Pong),
			131 => Some(Connect),
			132 => Some(ConnectResponse),
			133 => Some(ConnectionEstablished),
			134 => Some(Acknowledge),
			135 => Some(Disconnect),
			136 => Some(Discovery),
			137 => Some(DiscoveryResponse),
			138 => Some(NatPunchMessage),
			139 => Some(NatIntroduction),
			140 => Some(ExpandMTURequest),
			141 => Some(ExpandMTUSuccess),
			142 => Some(NatIntroductionConfirmRequest),
			143 => Some(NatIntroductionConfirmed),
			_ => None
		}
	}
}

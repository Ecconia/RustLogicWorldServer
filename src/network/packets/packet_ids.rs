//File is adjusted for LogicWorld version 0.91.X
use PacketIDs::*;

pub enum PacketIDs {
	//### Join procedure: ###
	//C2S: Multiplayer and pre-join
	DiscoveryRequest,
	//S2C: Answer, contains join instructions
	DiscoveryResponse,
	//C2S: Send actual join request to server
	ConnectionApproval,
	//(S2C): Answering with Lidgren ConnectResponse or Disconnect
	//C2S: Very first data message, indicating ready to converse
	ConnectionEstablished,
	//S2C: Send the world to client
	WorldInitialization,
	//C2S: Indicates that processing the world is done, and player is ready to interact
	ClientLoadedWorld,
	
	//### Building: ###
	//Chat:
	ChatMessageSent, //C2S
	ChatMessageBroadcast, //S2Broadcast
	//Command
	RunCommand, //C2S
	//Building:
	BuildingRequest, //C2S
	BuildActionReceipt, //S2C: Answer to build request
	WorldUpdate, //S2Broadcast: Broadcast world changes
	//ExtraData:
	ExtraDataRequest, //C2S: Request some extra data to be sent -> causes answer
	ExtraDataChange, //C2S: Request some extra data to be changed -> causes broadcast
	ExtraDataUpdate, //S2CB: Update one or more client with the (new) extra data
	//Simulation:
	RequestSimulationSteps, //C2S: Trigger stepping a tick
	ResetToDefaultSimulationSpeed, //C2S: Reset TPS to default
	ServerStrugglingWithSimulationSpeed, //S2Broadcast: Send the amount of missed ticks in seconds
	CircuitStatesUpdate, //S2Broadcast: Update circuit state changes
	//Special:
	RpcCall, //C2S: Send RPC instruction
	RpcConfirm, //S2C: Confirm execution - For callback?
	//Other:
	DebugMessage, //S2C: Send message to clients console
	//Subassembly:
	SubassemblyRequest, //C2S: No clue!
	SubassemblyRequestResponse, //S2C: Answer to no clue!
	//### Player update: ###
	//List of Players
	PlayerList, //S2Broadcast: Update the list of visible Bobbys
	//Player movement:
	PlayerPosition, //C2S: Tell server about the new mental position of Bobby
	SetPlayerPositionData, //S2C: Update the clients position (by force)
	PlayerPositionUpdate, //C2Broadcast: Tell everyone about the new position
	//Player hotbar:
	PlayerHotbar, //C2S: Tell server about the new hotbar content (or so?)
	//Player looks:
	PlayerAppearance, //C2S: Update server with Bobby's new look
	PlayerAppearanceUpdate, //C2Broadcast: Tell everyone about the new look
}

impl PacketIDs {
	pub const fn id(&self) -> u32 {
		match self {
			BuildingRequest => 2,
			ChatMessageSent => 3,
			ExtraDataChange => 4,
			ExtraDataRequest => 5,
			PlayerAppearance => 6,
			PlayerHotbar => 7,
			PlayerPosition => 8,
			RequestSimulationSteps => 9,
			DiscoveryRequest => 10,
			DiscoveryResponse => 11,
			ResetToDefaultSimulationSpeed => 12,
			RpcCall => 13,
			RunCommand => 14,
			ClientLoadedWorld => 15,
			ConnectionApproval => 16,
			ConnectionEstablished => 17,
			WorldInitialization => 18,
			BuildActionReceipt => 19,
			ChatMessageBroadcast => 20,
			CircuitStatesUpdate => 21,
			DebugMessage => 22,
			ExtraDataUpdate => 23,
			PlayerAppearanceUpdate => 24,
			PlayerList => 25,
			PlayerPositionUpdate => 26,
			RpcConfirm => 27,
			ServerStrugglingWithSimulationSpeed => 28,
			SetPlayerPositionData => 29,
			WorldUpdate => 30,
			SubassemblyRequest => 31,
			SubassemblyRequestResponse => 32,
		}
	}
}
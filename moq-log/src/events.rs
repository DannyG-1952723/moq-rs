use std::collections::HashMap;

use crate::logfile::{GroupId, PathId, TimeFormat};

struct Event {
	time: f64,
	name: String,
	data: ProtocolEventData,
	path: Option<PathId>,
	time_format: Option<TimeFormat>,
	protocol_types: Option<Vec<String>>,
	group_id: Option<GroupId>,
	system_info: Option<SystemInformation>,
	custom_fields: HashMap<String, String>
}

struct SystemInformation {
	processor_id: Option<u32>,
	process_id: Option<u32>,
	thread_id: Option<u32>
}

impl SystemInformation {
	fn new(processor_id: Option<u32>, process_id: Option<u32>, thread_id: Option<u32>) -> SystemInformation {
		SystemInformation { processor_id, process_id, thread_id }
	}
}

enum ProtocolEventData {
	MoqEventData(MoqEventData)
}

enum MoqEventData {
	StreamCreated(StreamInfo),
	StreamParsed(StreamInfo),
	SessionStarted(SessionMessage),
	SessionUpdateCreated(SessionUpdate),
	SessionUpdateParsed(SessionUpdate),
	AnnouncePleaseCreated(AnnouncePlease),
	AnnouncePleaseParsed(AnnouncePlease),
	AnnounceCreated(Announce),
	AnnounceParsed(Announce),
	SubscriptionStarted(Subscribe),
	SubscriptionUpdateCreated(SubscribeUpdate),
	SubscriptionUpdateParsed(SubscribeUpdate),
	SubscriptionGapCreated(SubscribeGap),
	SubscriptionGapParsed(SubscribeGap),
	InfoCreated(Info),
	InfoParsed(Info),
	InfoPleaseCreated(InfoPlease),
	InfoPleaseParsed(InfoPlease),
	FetchCreated(Fetch),
	FetchParsed(Fetch),
	FetchUpdateCreated(FetchUpdate),
	FetchUpdateParsed(FetchUpdate),
	GroupCreated(Group),
	GroupParsed(Group),
	FrameCreated(Frame),
	FrameParsed(Frame)
}

struct StreamInfo {
	stream_type: StreamType
}

enum StreamType {
	Session,
	Announced,
	Subscribe,
	Fetch,
	Info,
	Group
}

enum SessionMessage {
	SessionClient(SessionClient),
	SessionServer(SessionServer)
}

struct SessionClient {
	supported_versions: Vec<u64>,
	extension_ids: Vec<u64>
}

struct SessionServer {
	selected_version: u64,
	extension_ids: Vec<u64>
}

struct SessionUpdate {
	session_bitrate: u64
}

struct AnnouncePlease {
	track_prefix_parts: Vec<String>
}

struct Announce {
	announce_status: AnnounceStatus,
	track_suffix_parts: Vec<Vec<String>>
}

enum AnnounceStatus {
	/// Path is no longer available
	Ended,
	/// Path is now available
	Active,
	/// All active paths have been sent
	Live
}

struct Subscribe {
	subscribe_id: u64,
	track_path_parts: Vec<String>,
	track_priority: u64,
	group_order: u64,
	group_expires: u64,
	group_min: u64,
	group_max: u64
}

struct SubscribeUpdate {
	track_priority: u64,
	group_order: u64,
	group_expires: u64,
	group_min: u64,
	group_max: u64
}

struct SubscribeGap {
	group_start: u64,
	group_count: u64,
	group_error_code: u64
}

struct Info {
	track_priority: u64,
	group_latest: u64,
	group_order: u64,
	group_expires: u64
}

struct InfoPlease {
	track_path_parts: Vec<String>
}

struct Fetch {
	track_path_parts: Vec<String>,
	track_priority: u64,
	group_sequence: u64,
	frame_sequence: u64
}

struct FetchUpdate {
	track_priority: u64
}

struct Group {
	subscribe_id: u64,
	group_sequence: u64
}

struct Frame {
	payload: RawInfo
}

struct RawInfo {
	/// The full byte length
	length: Option<u64>,
	/// The byte length of the payload
	payload_length: Option<u64>,
	/// The (potentially truncated) contents, including headers and possibly trailers
	data: Option<Vec<u8>>
}

impl RawInfo {
	fn new(length: Option<u64>, payload_length: Option<u64>, data: Option<Vec<u8>>) -> RawInfo {
		RawInfo { length, payload_length, data }
	}
}

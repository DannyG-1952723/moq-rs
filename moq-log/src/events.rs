use std::collections::HashMap;

use chrono::Utc;

use crate::{logfile::{GroupId, PathId, TimeFormat}, util::{bytes_to_hexstring, HexString, MAX_LOG_DATA_LEN, VERSION_STRING}};

pub struct Event {
	time: i64,
	name: String,
	data: ProtocolEventData,
	path: Option<PathId>,
	time_format: Option<TimeFormat>,
	protocol_types: Option<Vec<String>>,
	group_id: Option<GroupId>,
	system_info: Option<SystemInformation>,
	custom_fields: HashMap<String, String>
}

impl Event {
	pub fn stream_created(stream_type: StreamType) -> Self {
		let event_data = MoqEventData::StreamCreated(Stream::new(stream_type));

		Self::new("stream_created", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn stream_parsed(stream_type: StreamType) -> Self {
		let event_data = MoqEventData::StreamParsed(Stream::new(stream_type));

		Self::new("stream_parsed", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn session_started_client(supported_versions: Vec<u64>, extension_ids: Option<Vec<u64>>) -> Self {
		let event_data = MoqEventData::SessionStarted(SessionMessage::SessionClient(SessionClient::new(supported_versions, extension_ids)));

		Self::new("session_started", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn session_started_server(selected_version: u64, extension_ids: Option<Vec<u64>>) -> Self {
		let event_data = MoqEventData::SessionStarted(SessionMessage::SessionServer(SessionServer::new(selected_version, extension_ids)));

		Self::new("session_started", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn session_update_created(session_bitrate: u64) -> Self {
		let event_data = MoqEventData::SessionUpdateCreated(SessionUpdate::new(session_bitrate));

		Self::new("session_update_created", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn session_update_parsed(session_bitrate: u64) -> Self {
		let event_data = MoqEventData::SessionUpdateParsed(SessionUpdate::new(session_bitrate));

		Self::new("session_update_parsed", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn announce_please_created(track_prefix_parts: Vec<String>) -> Self {
		let event_data = MoqEventData::AnnouncePleaseCreated(AnnouncePlease::new(track_prefix_parts));

		Self::new("announce_please_created", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn announce_please_parsed(track_prefix_parts: Vec<String>) -> Self {
		let event_data = MoqEventData::AnnouncePleaseParsed(AnnouncePlease::new(track_prefix_parts));

		Self::new("announce_please_parsed", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn announce_created(announce_status: AnnounceStatus, track_suffix_parts: Vec<Vec<String>>) -> Self {
		let event_data = MoqEventData::AnnounceCreated(Announce::new(announce_status, track_suffix_parts));

		Self::new("announce_created", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn announce_parsed(announce_status: AnnounceStatus, track_suffix_parts: Vec<Vec<String>>) -> Self {
		let event_data = MoqEventData::AnnounceParsed(Announce::new(announce_status, track_suffix_parts));

		Self::new("announce_parsed", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn subscription_started(subscribe_id: u64, track_path_parts: Vec<String>, track_priority: u64, group_order: u64, group_expires: u64, group_min: u64, group_max: u64) -> Self {
		let event_data = MoqEventData::SubscriptionStarted(Subscribe::new(subscribe_id, track_path_parts, track_priority, group_order, group_expires, group_min, group_max));

		Self::new("subscription_started", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn subscription_update_created(track_priority: u64, group_order: u64, group_expires: u64, group_min: u64, group_max: u64) -> Self {
		let event_data = MoqEventData::SubscriptionUpdateCreated(SubscribeUpdate::new(track_priority, group_order, group_expires, group_min, group_max));

		Self::new("subscription_update_created", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn subscription_update_parsed(track_priority: u64, group_order: u64, group_expires: u64, group_min: u64, group_max: u64) -> Self {
		let event_data = MoqEventData::SubscriptionUpdateParsed(SubscribeUpdate::new(track_priority, group_order, group_expires, group_min, group_max));

		Self::new("subscription_update_parsed", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn subscription_gap_created(group_start: u64, group_count: u64, group_error_code: u64) -> Self {
		let event_data = MoqEventData::SubscriptionGapCreated(SubscribeGap::new(group_start, group_count, group_error_code));

		Self::new("subscription_gap_created", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn subscription_gap_parsed(group_start: u64, group_count: u64, group_error_code: u64) -> Self {
		let event_data = MoqEventData::SubscriptionGapParsed(SubscribeGap::new(group_start, group_count, group_error_code));

		Self::new("subscription_gap_parsed", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn info_created(track_priority: u64, group_latest: u64, group_order: u64, group_expires: u64) -> Self {
		let event_data = MoqEventData::InfoCreated(Info::new(track_priority, group_latest, group_order, group_expires));

		Self::new("info_created", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn info_parsed(track_priority: u64, group_latest: u64, group_order: u64, group_expires: u64) -> Self {
		let event_data = MoqEventData::InfoParsed(Info::new(track_priority, group_latest, group_order, group_expires));

		Self::new("info_parsed", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn info_please_created(track_path_parts: Vec<String>) -> Self {
		let event_data = MoqEventData::InfoPleaseCreated(InfoPlease::new(track_path_parts));

		Self::new("info_please_created", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn info_please_parsed(track_path_parts: Vec<String>) -> Self {
		let event_data = MoqEventData::InfoPleaseParsed(InfoPlease::new(track_path_parts));

		Self::new("info_please_parsed", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn fetch_created(track_path_parts: Vec<String>, track_priority: u64, group_sequence: u64, frame_sequence: u64) -> Self {
		let event_data = MoqEventData::FetchCreated(Fetch::new(track_path_parts, track_priority, group_sequence, frame_sequence));

		Self::new("fetch_created", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn fetch_parsed(track_path_parts: Vec<String>, track_priority: u64, group_sequence: u64, frame_sequence: u64) -> Self {
		let event_data = MoqEventData::FetchParsed(Fetch::new(track_path_parts, track_priority, group_sequence, frame_sequence));

		Self::new("fetch_parsed", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn fetch_update_created(track_priority: u64) -> Self {
		let event_data = MoqEventData::FetchUpdateCreated(FetchUpdate::new(track_priority));

		Self::new("fetch_update_created", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn fetch_update_parsed(track_priority: u64) -> Self {
		let event_data = MoqEventData::FetchUpdateParsed(FetchUpdate::new(track_priority));

		Self::new("fetch_update_parsed", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn group_created(subscribe_id: u64, group_sequence: u64) -> Self {
		let event_data = MoqEventData::GroupCreated(Group::new(subscribe_id, group_sequence));

		Self::new("group_created", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn group_parsed(subscribe_id: u64, group_sequence: u64) -> Self {
		let event_data = MoqEventData::GroupParsed(Group::new(subscribe_id, group_sequence));

		Self::new("group_parsed", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn frame_created(payload: RawInfo) -> Self {
		let event_data = MoqEventData::FrameCreated(Frame::new(payload));

		Self::new("frame_created", ProtocolEventData::MoqEventData(event_data))
	}

	pub fn frame_parsed(payload: RawInfo) -> Self {
		let event_data = MoqEventData::FrameParsed(Frame::new(payload));

		Self::new("frame_parsed", ProtocolEventData::MoqEventData(event_data))
	}

	// Assumes default TimeFormat (relative to epoch, epoch = "1970-01-01T00:00:00.000Z")
	// TODO: Base 'time' value upon chosen TimeFormat
	fn new(event_name: &str, data: ProtocolEventData) -> Self {
		Self {
			time: Utc::now().timestamp_millis(),
			name: format!("{VERSION_STRING}:{event_name}"),
			data,
			path: Some("".to_string()),
			time_format: None,
			protocol_types: None,
			// TODO: Maybe add a group ID
			group_id: None,
			system_info: None,
			custom_fields: HashMap::new()
		}
	}
}

enum ProtocolEventData {
	MoqEventData(MoqEventData)
}

enum MoqEventData {
	StreamCreated(Stream),
	StreamParsed(Stream),
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

struct Stream {
	stream_type: StreamType
}

impl Stream {
	fn new(stream_type: StreamType) -> Self {
		Self { stream_type }
	}
}

pub enum StreamType {
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

impl SessionClient {
	fn new(supported_versions: Vec<u64>, extension_ids: Option<Vec<u64>>) -> Self {
		let extension_ids = extension_ids.unwrap_or(Vec::new());

		Self { supported_versions, extension_ids }
	}
}

struct SessionServer {
	selected_version: u64,
	extension_ids: Vec<u64>
}

impl SessionServer {
	fn new(selected_version: u64, extension_ids: Option<Vec<u64>>) -> Self {
		let extension_ids = extension_ids.unwrap_or(Vec::new());

		Self { selected_version, extension_ids }
	}
}

struct SessionUpdate {
	session_bitrate: u64
}

impl SessionUpdate {
	fn new(session_bitrate: u64) -> Self {
		Self { session_bitrate }
	}
}

struct AnnouncePlease {
	track_prefix_parts: Vec<String>
}

impl AnnouncePlease {
	fn new(track_prefix_parts: Vec<String>) -> Self {
		Self { track_prefix_parts }
	}
}

struct Announce {
	announce_status: AnnounceStatus,
	track_suffix_parts: Vec<Vec<String>>
}

impl Announce {
	fn new(announce_status: AnnounceStatus, track_suffix_parts: Vec<Vec<String>>) -> Self {
		Self { announce_status, track_suffix_parts }
	}
}

pub enum AnnounceStatus {
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

impl Subscribe {
	fn new(subscribe_id: u64, track_path_parts: Vec<String>, track_priority: u64, group_order: u64, group_expires: u64, group_min: u64, group_max: u64) -> Self {
		Self { subscribe_id, track_path_parts, track_priority, group_order, group_expires, group_min, group_max }
	}
}

struct SubscribeUpdate {
	track_priority: u64,
	group_order: u64,
	group_expires: u64,
	group_min: u64,
	group_max: u64
}

impl SubscribeUpdate {
	fn new(track_priority: u64, group_order: u64, group_expires: u64, group_min: u64, group_max: u64) -> Self {
		Self { track_priority, group_order, group_expires, group_min, group_max }
	}
}

struct SubscribeGap {
	group_start: u64,
	group_count: u64,
	group_error_code: u64
}

impl SubscribeGap {
	fn new(group_start: u64, group_count: u64, group_error_code: u64) -> Self {
		Self { group_start, group_count, group_error_code }
	}
}

struct Info {
	track_priority: u64,
	group_latest: u64,
	group_order: u64,
	group_expires: u64
}

impl Info {
	fn new(track_priority: u64, group_latest: u64, group_order: u64, group_expires: u64) -> Self {
		Self { track_priority, group_latest, group_order, group_expires }
	}
}

struct InfoPlease {
	track_path_parts: Vec<String>
}

impl InfoPlease {
	fn new(track_path_parts: Vec<String>) -> Self {
		Self { track_path_parts }
	}
}

struct Fetch {
	track_path_parts: Vec<String>,
	track_priority: u64,
	group_sequence: u64,
	frame_sequence: u64
}

impl Fetch {
	fn new(track_path_parts: Vec<String>, track_priority: u64, group_sequence: u64, frame_sequence: u64) -> Self {
		Self { track_path_parts, track_priority, group_sequence, frame_sequence }
	}
}

struct FetchUpdate {
	track_priority: u64
}

impl FetchUpdate {
	fn new(track_priority: u64) -> Self {
		Self { track_priority }
	}
}

struct Group {
	subscribe_id: u64,
	group_sequence: u64
}

impl Group {
	fn new(subscribe_id: u64, group_sequence: u64) -> Self {
		Self { subscribe_id, group_sequence }
	}
}

struct Frame {
	payload: RawInfo
}

impl Frame {
	fn new(payload: RawInfo) -> Self {
		Self { payload }
	}
}

pub struct RawInfo {
	/// The full byte length
	length: Option<u64>,
	/// The byte length of the payload
	payload_length: Option<u64>,
	/// The (potentially truncated) contents, including headers and possibly trailers
	data: Option<HexString>
}

impl RawInfo {
	pub fn new(length: Option<u64>, data: Option<&[u8]>) -> Self {
		match data {
			Some(payload) => {
				let payload_length: Option<u64> = Some(payload.len().try_into().unwrap());

				// Only log the first MAX_LOG_DATA_LEN bytes
				if payload_length.unwrap() > MAX_LOG_DATA_LEN.try_into().unwrap() {
					let truncated = &payload[..MAX_LOG_DATA_LEN];
					return Self { length, payload_length, data: Some(bytes_to_hexstring(truncated)) };
				}

				Self { length, payload_length, data: Some(bytes_to_hexstring(payload)) }
			},
			None => Self { length, payload_length: None, data: None }
		}
	}
}

struct SystemInformation {
	processor_id: Option<u32>,
	process_id: Option<u32>,
	thread_id: Option<u32>
}

impl SystemInformation {
	fn new(processor_id: Option<u32>, process_id: Option<u32>, thread_id: Option<u32>) -> Self {
		Self { processor_id, process_id, thread_id }
	}
}

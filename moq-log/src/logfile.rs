use std::collections::HashMap;

use chrono::NaiveDateTime;

pub struct QlogFileSeq {
	log_file: LogFile,
	trace: TraceSeq
}

struct LogFile {
	file_schema: String,
	serialization_format: String,
	title: Option<String>,
	description: Option<String>,
	event_schemas: Vec<String>
}

impl LogFile {
	// TODO: Add support for other file schemas
	// TODO: Add support for other serialization formats
	pub fn new(title: Option<String>, description: Option<String>) -> LogFile {
		LogFile {
			file_schema: "urn:ietf:params:qlog:file:sequential".to_string(),
			serialization_format: "application/qlog+json-seq".to_string(),
			title,
			description,
			// TODO: Maybe add QUIC events to this
			// TODO: Change MoQ event space (this is a placeholder)
			event_schemas: vec!["urn:ietf:params:qlog:events:moq".to_string()]
		}
	}
}

struct TraceSeq {
	title: Option<String>,
	description: Option<String>,
	common_fields: Option<CommonFields>,
	vantage_point: Option<VantagePoint>,
}

struct CommonFields {
	path: Option<PathId>,
	time_format: Option<TimeFormat>,
	reference_time: Option<ReferenceTime>,
	protocol_types: Option<Vec<String>>,
	group_id: Option<GroupId>,
	custom_fields: HashMap<String, String>
}

type PathId = String;
type GroupId = String;

enum TimeFormat {
	RelativeToEpoch,
	RelativeToPreviousEvent
}

struct ReferenceTime {
	clock_type: ClockType,
	epoch: Epoch,
	// TODO: Look at what to do with timezones
	wall_clock_time: Option<NaiveDateTime>
}

enum ClockType {
	System,
	Monotonic,
	Other(String)
}

enum Epoch {
	// TODO: Look at what to do with timezones
	Rfc3339DateTime(NaiveDateTime),
	Unknown
}

struct VantagePoint {
	name: Option<String>,
	vp_type: VantagePointType,
	flow: Option<VantagePointType> // TODO: Make required if vp_type == Network
}

enum VantagePointType {
	/// Initiates the connection
	Client,
	/// Accepts the connection
	Server,
	/// Observer in between client and server
	Network,
	Unknown
}

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

enum ProtocolEventData {
	LogLevelEventData(LogLevelEventData)
}

enum LogLevelEventData {
	LogLevelError(LogLevelError),
	LogLevelWarning(LogLevelWarning),
	LogLevelInfo(LogLevelInfo),
	LogLevelDebug(LogLevelDebug),
	LogLevelVerbose(LogLevelVerbose)
}

// TODO: Look at what to do with the extension (see RFC draft)
struct LogLevelError {
	code: Option<u64>,
	message: Option<String>
}

// TODO: Look at what to do with the extension (see RFC draft)
struct LogLevelWarning {
	code: Option<u64>,
	message: Option<String>
}

// TODO: Look at what to do with the extension (see RFC draft)
struct LogLevelInfo {
	message: Option<String>
}

// TODO: Look at what to do with the extension (see RFC draft)
struct LogLevelDebug {
	message: Option<String>
}

// TODO: Look at what to do with the extension (see RFC draft)
struct LogLevelVerbose {
	message: Option<String>
}

struct RawInfo {
	length: Option<u64>,
	payload_length: Option<u64>,
	data: Option<Vec<u8>>
}

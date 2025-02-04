use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};

// TODO: Maybe implement the Default trait for (almost) all structs

pub struct QlogFileSeq {
	log_file_details: LogFile,
	trace: TraceSeq
}

impl QlogFileSeq {
	fn new(log_file_details: LogFile, trace: TraceSeq) -> QlogFileSeq {
		QlogFileSeq { log_file_details, trace }
	}
}

struct LogFile {
	/// Identifies the concrete log file schema
	file_schema: String,
	/// Indicates the serialization format using a media type
	serialization_format: String,
	title: Option<String>,
	description: Option<String>,
	/// Identifies concrete event namespaces and their associated types
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

impl TraceSeq {
	fn new(title: Option<String>, description: Option<String>, common_fields: Option<CommonFields>, vantage_point: Option<VantagePoint>) -> TraceSeq {
		TraceSeq { title, description, common_fields, vantage_point }
	}
}

struct CommonFields {
	path: Option<PathId>,
	time_format: Option<TimeFormat>,
	reference_time: Option<ReferenceTime>,
	protocol_types: Option<Vec<String>>,
	group_id: Option<GroupId>,
	custom_fields: HashMap<String, String>
}

impl CommonFields {
	fn new(path: Option<PathId>, time_format: Option<TimeFormat>, reference_time: Option<ReferenceTime>, protocol_types: Option<Vec<String>>, group_id: Option<GroupId>, custom_fields: Option<HashMap<String, String>>) -> CommonFields {
		let custom_fields = custom_fields.unwrap_or(HashMap::new());

		CommonFields { path, time_format, reference_time, protocol_types, group_id, custom_fields }
	}
}

type PathId = String;
type GroupId = String;

enum TimeFormat {
	/// Relative to the ReferenceTime 'epoch' field
	RelativeToEpoch,
	/// Delta-encoded value, based on the previously logged value
	RelativeToPreviousEvent
}

struct ReferenceTime {
	clock_type: ClockType,
	epoch: Epoch,
	wall_clock_time: Option<DateTime<FixedOffset>>
}

impl ReferenceTime {
	/// clock_type defaults to System when None
	///
	/// epoch defaults to "1970-01-01T00:00:00.000Z" when None
	fn new(clock_type: Option<ClockType>, epoch: Option<Epoch>, wall_clock_time: Option<DateTime<FixedOffset>>) -> ReferenceTime {
		let clock_type = clock_type.unwrap_or(ClockType::System);
		let epoch = epoch.unwrap_or(Epoch::Rfc3339DateTime(DateTime::parse_from_rfc3339("1970-01-01T00:00:00.000Z").unwrap()));

		if clock_type == ClockType::Monotonic && epoch != Epoch::Unknown {
			panic!("When using the 'monotonic' clock type, the epoch field must have the value 'unknown'");
		}

		ReferenceTime { clock_type, epoch, wall_clock_time }
	}
}

#[derive(PartialEq, Eq)]
enum ClockType {
	System,
	Monotonic,
	Other(String)
}

#[derive(PartialEq, Eq)]
enum Epoch {
	Rfc3339DateTime(DateTime<FixedOffset>),
	Unknown
}

/// Vantage point from which a trace originates
struct VantagePoint {
	name: Option<String>,
	vp_type: VantagePointType,
	/// The direction of the data flow (e.g., Client: "packet sent" event goes in direction of the server, Server: "packet sent" event goes in direction of the client)
	flow: Option<VantagePointType>
}

impl VantagePoint {
	fn new(name: Option<String>, vp_type: VantagePointType, flow: Option<VantagePointType>) -> VantagePoint {
		if vp_type == VantagePointType::Network {
			if let None = flow {
				panic!("The 'flow' field is required if the type is 'network'");
			}
		}

		VantagePoint { name, vp_type, flow }
	}
}

#[derive(PartialEq, Eq)]
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

impl SystemInformation {
	fn new(processor_id: Option<u32>, process_id: Option<u32>, thread_id: Option<u32>) -> SystemInformation {
		SystemInformation { processor_id, process_id, thread_id }
	}
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
/// Used to log details of an internal error that might not get reflected on the wire
struct LogLevelError {
	code: Option<u64>,
	message: Option<String>
}

// TODO: Look at what to do with the extension (see RFC draft)
/// Used to log details of an internal warning that might not get reflected on the wire
struct LogLevelWarning {
	code: Option<u64>,
	message: Option<String>
}

// TODO: Look at what to do with the extension (see RFC draft)
/// Used mainly for implementations that want to use qlog as their one and only logging format but still want to support unstructured string messages
struct LogLevelInfo {
	message: Option<String>
}

// TODO: Look at what to do with the extension (see RFC draft)
/// Used mainly for implementations that want to use qlog as their one and only logging format but still want to support unstructured string messages
struct LogLevelDebug {
	message: Option<String>
}

// TODO: Look at what to do with the extension (see RFC draft)
/// Used mainly for implementations that want to use qlog as their one and only logging format but still want to support unstructured string messages
struct LogLevelVerbose {
	message: Option<String>
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

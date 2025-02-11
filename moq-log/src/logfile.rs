use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::util::VERSION_STRING;

#[derive(Serialize)]
pub struct QlogFileSeq {
	#[serde(flatten)]
	log_file_details: LogFile,
	trace: TraceSeq
}

impl QlogFileSeq {
	pub fn new(log_file_details: LogFile, trace: TraceSeq) -> QlogFileSeq {
		QlogFileSeq { log_file_details, trace }
	}
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct LogFile {
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
			event_schemas: vec![format!("urn:ietf:params:qlog:events:{VERSION_STRING}")]
		}
	}
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct TraceSeq {
	title: Option<String>,
	description: Option<String>,
	common_fields: Option<CommonFields>,
	vantage_point: Option<VantagePoint>,
}

impl TraceSeq {
	pub fn new(title: Option<String>, description: Option<String>, common_fields: Option<CommonFields>, vantage_point: Option<VantagePoint>) -> TraceSeq {
		TraceSeq { title, description, common_fields, vantage_point }
	}
}

fn is_empty_or_none(path: &Option<PathId>) -> bool {
    match path {
        Some(p) => p.is_empty(),
        None => true,
    }
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct CommonFields {
	#[serde(skip_serializing_if = "is_empty_or_none")]
	path: Option<PathId>,
	time_format: Option<TimeFormat>,
	reference_time: Option<ReferenceTime>,
	protocol_types: Option<Vec<String>>,
	group_id: Option<GroupId>,
	#[serde(flatten)]						// Adds the custom fields directly to CommonFields when serializing
	custom_fields: HashMap<String, String>
}

impl CommonFields {
	pub fn new(path: Option<PathId>, time_format: Option<TimeFormat>, reference_time: Option<ReferenceTime>, protocol_types: Option<Vec<String>>, group_id: Option<GroupId>, custom_fields: Option<HashMap<String, String>>) -> CommonFields {
		let custom_fields = custom_fields.unwrap_or(HashMap::new());

		CommonFields { path, time_format, reference_time, protocol_types, group_id, custom_fields }
	}
}

impl Default for CommonFields {
	fn default() -> Self {
		Self {
			path: Some("".to_string()),
			time_format: Some(TimeFormat::default()),
			reference_time: Some(ReferenceTime::default()),
			protocol_types: Some(vec!["MoQ".to_string()]),
			group_id: None,
			custom_fields: HashMap::new()
		}
	}
}

pub type PathId = String;
pub type GroupId = String;

#[derive(Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeFormat {
	/// Relative to the ReferenceTime 'epoch' field
	#[default]
	RelativeToEpoch,
	/// Delta-encoded value, based on the previously logged value
	RelativeToPreviousEvent
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct ReferenceTime {
	clock_type: ClockType,
	epoch: Epoch,
	wall_clock_time: Option<DateTime<FixedOffset>>
}

impl ReferenceTime {
	/// clock_type defaults to System when None
	///
	/// epoch defaults to "1970-01-01T00:00:00.000Z" when None
	pub fn new(clock_type: Option<ClockType>, epoch: Option<Epoch>, wall_clock_time: Option<DateTime<FixedOffset>>) -> ReferenceTime {
		let clock_type = clock_type.unwrap_or(ClockType::default());
		let epoch = epoch.unwrap_or(Epoch::default());

		if clock_type == ClockType::Monotonic && epoch != Epoch::Unknown {
			panic!("When using the 'monotonic' clock type, the epoch field must have the value 'unknown'");
		}

		ReferenceTime { clock_type, epoch, wall_clock_time }
	}
}

impl Default for ReferenceTime {
	fn default() -> Self {
		Self { clock_type: ClockType::default(), epoch: Epoch::default(), wall_clock_time: None }
	}
}

#[derive(Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClockType {
	#[default]
	System,
	Monotonic,
	Other(String)
}

#[derive(PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", untagged)]
pub enum Epoch {
	Rfc3339DateTime(DateTime<FixedOffset>),
	Unknown
}

impl Default for Epoch {
	fn default() -> Self {
		Self::Rfc3339DateTime(DateTime::parse_from_rfc3339("1970-01-01T00:00:00.000Z").unwrap())
	}
}

/// Vantage point from which a trace originates
#[skip_serializing_none]
#[derive(Serialize)]
pub struct VantagePoint {
	name: Option<String>,
	// 'type' is a keyword in Rust
	#[serde(rename = "type")]
	vp_type: VantagePointType,
	/// The direction of the data flow (e.g., Client: "packet sent" event goes in direction of the server, Server: "packet sent" event goes in direction of the client)
	flow: Option<VantagePointType>
}

impl VantagePoint {
	pub fn new(name: Option<String>, vp_type: VantagePointType, flow: Option<VantagePointType>) -> VantagePoint {
		if vp_type == VantagePointType::Network {
			if let None = flow {
				panic!("The 'flow' field is required if the type is 'network'");
			}
		}

		VantagePoint { name, vp_type, flow }
	}
}

#[derive(PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VantagePointType {
	/// Initiates the connection
	Client,
	/// Accepts the connection
	Server,
	/// Observer in between client and server
	Network,
	Unknown
}

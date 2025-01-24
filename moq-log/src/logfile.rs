pub struct LogFile {
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

use std::{env, fs::File, io::{BufWriter, Write}, sync::{LazyLock, Mutex}};

use crate::logfile::{CommonFields, LogFile, QlogFileSeq, TraceSeq, VantagePoint};

// Static variable so that a logger variable doesn't need to be passed to every function wherein logging occurs
static QLOG_WRITER: LazyLock<Mutex<QlogWriter>> = LazyLock::new(|| Mutex::new(QlogWriter::init()));

pub struct QlogWriter {
	writer: Option<BufWriter<File>>,
	file_details_written: bool
}

impl QlogWriter {
	fn init() -> Self {
		match env::var("QLOGFILE") {
			Ok(qlog_file_path) => {
				match File::create(qlog_file_path) {
					Ok(file) => Self { writer: Some(BufWriter::new(file)), file_details_written: false },
					Err(e) => panic!("Error creating qlog file: {e}")
				}
			},
			Err(_) => Self { writer: None, file_details_written: true }
		}
	}

	/// Logs the needed details so qlog file readers can interpret the logs correctly
	pub fn log_file_details(file_title: Option<String>, file_description: Option<String>, trace_title: Option<String>, trace_description: Option<String>, vantage_point: Option<VantagePoint>) {
		let mut qlog_writer = QLOG_WRITER.lock().unwrap();

		if let Some(ref mut writer) = qlog_writer.writer {
			let log_file_details = LogFile::new(file_title, file_description);
			let trace = TraceSeq::new(trace_title, trace_description, Some(CommonFields::default()), vantage_point);

			let qlog_file_seq = QlogFileSeq::new(log_file_details, trace);

			// TODO: Write to file correctly (record separators, newlines...)
			let json_representation = serde_json::to_string_pretty(&qlog_file_seq).unwrap();
			writer.write(json_representation.as_bytes()).unwrap();
			writer.flush().unwrap();

			qlog_writer.file_details_written = true;
		}
	}

	// TODO: Update (current implementation is to test if writing works)
	// Flushes write buffer after every log, otherwise won't write to file when exiting the program using ^C
	pub fn log(text: &String) {
		let mut qlog_writer = QLOG_WRITER.lock().unwrap();

		if !qlog_writer.file_details_written {
			panic!("Log the qlog file details before logging events, call 'QlogWriter::log_file_details()' somewhere in the beginning of the program");
		}

		if let Some(ref mut writer) = qlog_writer.writer {
			writer.write(text.as_bytes()).unwrap();
			writer.flush().unwrap();
		}
	}
}

use std::{env, fs::File, io::{BufWriter, Write}, sync::{LazyLock, Mutex}};

// Static variable so that a logger variable doesn't need to be passed to every function wherein logging occurs
static QLOG_WRITER: LazyLock<Mutex<QlogWriter>> = LazyLock::new(|| Mutex::new(QlogWriter::init()));

pub struct QlogWriter {
	writer: Option<BufWriter<File>>
}

impl QlogWriter {
	fn init() -> Self {
		match env::var("QLOGFILE") {
			Ok(qlog_file_path) => {
				match File::create(qlog_file_path) {
					Ok(file) => Self { writer: Some(BufWriter::new(file)) },
					Err(e) => panic!("Error creating qlog file: {e}")
				}
			},
			Err(_) => Self { writer: None }
		}
	}

	// TODO: Update (current implementation is to test if writing works)
	pub fn log(text: &String) {
		if let Some(ref mut writer) = QLOG_WRITER.lock().unwrap().writer {
			writer.write(text.as_bytes()).unwrap();
			writer.flush().unwrap();
		}
	}
}

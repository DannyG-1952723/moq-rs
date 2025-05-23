use std::fmt;

use crate::{coding::*, message, Error};

use moq_async::Close;
use qlog_rs::{events::Event, writer::QlogWriter};

pub(super) struct Writer {
	stream: web_transport::SendStream,
	buffer: bytes::BytesMut,
}

impl Writer {
	pub fn new(stream: web_transport::SendStream) -> Self {
		Self {
			stream,
			buffer: Default::default(),
		}
	}

	pub async fn open(session: &mut web_transport::Session, typ: message::DataType, tracing_id: u64) -> Result<Self, Error> {
		let send = session.open_uni().await?;

		QlogWriter::log_event(Event::moq_stream_created(typ.to_log_type(), tracing_id));

		let mut writer = Self::new(send);
		writer.encode(&typ).await?;

		Ok(writer)
	}

	pub async fn encode<T: Encode + fmt::Debug>(&mut self, msg: &T) -> Result<(), Error> {
		self.buffer.clear();
		msg.encode(&mut self.buffer);

		while !self.buffer.is_empty() {
			self.stream.write_buf(&mut self.buffer).await?;
		}

		Ok(())
	}

	pub async fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
		self.stream.write(buf).await?; // convert the error type
		Ok(())
	}
}

impl Close<Error> for Writer {
	fn close(&mut self, err: Error) {
		self.stream.reset(err.to_code());
	}
}

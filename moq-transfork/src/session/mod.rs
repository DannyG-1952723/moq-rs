use crate::{coding::VarInt, message, AnnouncedConsumer, Error, Path, RouterConsumer, Track, TrackConsumer};

use moq_async::{spawn, Close, OrClose};
use moq_log::{events::Event, writer::QlogWriter};

mod publisher;
mod reader;
mod stream;
mod subscriber;
mod writer;

use publisher::*;
use reader::*;
use stream::*;
use subscriber::*;
use writer::*;

/// A MoqTransfork session, used to publish and/or subscribe to broadcasts.
///
/// A publisher will [Self::publish] tracks, or alternatively [Self::announce] and [Self::route] arbitrary paths.
/// A subscriber will [Self::subscribe] to tracks, or alternatively use [Self::announced] to discover arbitrary paths.
#[derive(Clone)]
pub struct Session {
	webtransport: web_transport::Session,
	publisher: Publisher,
	subscriber: Subscriber,
	tracing_id: u64
}

impl Session {
	fn new(mut session: web_transport::Session, stream: Stream, tracing_id: u64) -> Self {
		let publisher = Publisher::new(session.clone());
		let subscriber = Subscriber::new(session.clone());

		let this = Self {
			webtransport: session.clone(),
			publisher: publisher.clone(),
			subscriber: subscriber.clone(),
			tracing_id
		};

		spawn(async move {
			let res = tokio::select! {
				res = Self::run_session(stream, tracing_id) => res,
				res = Self::run_bi(session.clone(), publisher, tracing_id) => res,
				res = Self::run_uni(session.clone(), subscriber, tracing_id) => res,
			};

			if let Err(err) = res {
				tracing::warn!(?err, "terminated");
				session.close(err.to_code(), &err.to_string());
			}
		});

		this
	}

	/// Perform the MoQ handshake as a client.
	pub async fn connect<T: Into<web_transport::Session>>(session: T) -> Result<Self, Error> {
		let mut session = session.into();
		// TODO: Think about what to do with the tracing ID here
		let mut stream = Stream::open(&mut session, message::ControlType::Session, 0).await?;
		let tracing_id = Self::connect_setup(&mut stream).await.or_close(&mut stream)?;
		Ok(Self::new(session, stream, tracing_id))
	}

	async fn connect_setup(setup: &mut Stream) -> Result<u64, Error> {
		let client = message::ClientSetup {
			versions: [message::Version::CURRENT].into(),
			extensions: Default::default(),
			tracing_id: rand::random_range(0..=VarInt::MAX.into())
		};

		let versions: Vec<u64> = client.versions.iter().map(|&v| v.into()).collect();
		QlogWriter::log_event(Event::session_started_client_created(versions, Some(client.extensions.keys()), client.tracing_id));

		setup.writer.encode(&client).await?;
		let server: message::ServerSetup = setup.reader.decode().await?;

		QlogWriter::log_event(Event::session_started_server_parsed(server.version.into(), Some(server.extensions.keys()), client.tracing_id));

		tracing::info!(version = ?server.version, "connected");

		Ok(client.tracing_id)
	}

	/// Perform the MoQ handshake as a server
	pub async fn accept<T: Into<web_transport::Session>>(session: T) -> Result<Self, Error> {
		let mut session = session.into();
		let mut stream = Stream::accept(&mut session).await?;
		let kind: message::ControlType = stream.reader.decode().await?;

		// TODO: Think about what to do with the tracing ID here
		QlogWriter::log_event(Event::stream_parsed(kind.to_log_type(), 0));

		if kind != message::ControlType::Session {
			return Err(Error::UnexpectedStream(kind));
		}

		let tracing_id = Self::accept_setup(&mut stream).await.or_close(&mut stream)?;
		Ok(Self::new(session, stream, tracing_id))
	}

	async fn accept_setup(control: &mut Stream) -> Result<u64, Error> {
		let client: message::ClientSetup = control.reader.decode().await?;

		let versions: Vec<u64> = client.versions.iter().map(|&v| v.into()).collect();
		QlogWriter::log_event(Event::session_started_client_parsed(versions, Some(client.extensions.keys()), client.tracing_id));

		if !client.versions.contains(&message::Version::CURRENT) {
			return Err(Error::Version(client.versions, [message::Version::CURRENT].into()));
		}

		let server = message::ServerSetup {
			version: message::Version::CURRENT,
			extensions: Default::default(),
		};

		QlogWriter::log_event(Event::session_started_server_created(server.version.into(), Some(server.extensions.keys()), client.tracing_id));

		control.writer.encode(&server).await?;

		tracing::info!(version = ?server.version, "connected");

		Ok(client.tracing_id)
	}

	async fn run_session(mut stream: Stream, tracing_id: u64) -> Result<(), Error> {
		while let Some(info) = stream.reader.decode_maybe::<message::Info>().await? {
			QlogWriter::log_event(Event::info_parsed(info.track_priority.try_into().unwrap(), info.group_latest, info.group_order as u64, tracing_id));
		}
		Err(Error::Cancel)
	}

	async fn run_uni(mut session: web_transport::Session, subscriber: Subscriber, tracing_id: u64) -> Result<(), Error> {
		loop {
			let mut stream = Reader::accept(&mut session).await?;
			let subscriber = subscriber.clone();

			spawn(async move {
				Self::run_data(&mut stream, subscriber, tracing_id).await.or_close(&mut stream).ok();
			});
		}
	}

	async fn run_data(stream: &mut Reader, mut subscriber: Subscriber, tracing_id: u64) -> Result<(), Error> {
		let kind: message::DataType = stream.decode().await?;

		QlogWriter::log_event(Event::stream_parsed(kind.to_log_type(), tracing_id));

		match kind {
			message::DataType::Group => subscriber.recv_group(stream, tracing_id).await,
		}
	}

	async fn run_bi(mut session: web_transport::Session, publisher: Publisher, tracing_id: u64) -> Result<(), Error> {
		loop {
			let mut stream = Stream::accept(&mut session).await?;
			let publisher = publisher.clone();

			spawn(async move {
				Self::run_control(&mut stream, publisher, tracing_id)
					.await
					.or_close(&mut stream)
					.ok();
			});
		}
	}

	async fn run_control(stream: &mut Stream, mut publisher: Publisher, tracing_id: u64) -> Result<(), Error> {
		let kind: message::ControlType = stream.reader.decode().await?;

		QlogWriter::log_event(Event::stream_parsed(kind.to_log_type(), tracing_id));

		match kind {
			message::ControlType::Session => Err(Error::UnexpectedStream(kind)),
			message::ControlType::Announce => publisher.recv_announce(stream, tracing_id).await,
			message::ControlType::Subscribe => publisher.recv_subscribe(stream, tracing_id).await,
			message::ControlType::Fetch => publisher.recv_fetch(stream, tracing_id).await,
			message::ControlType::Info => publisher.recv_info(stream, tracing_id).await,
		}
	}

	/// Publish a track, automatically announcing and serving it.
	pub fn publish(&mut self, track: TrackConsumer) -> Result<(), Error> {
		self.publisher.publish(track)
	}

	/// Optionally announce the provided tracks.
	/// This is advanced functionality if you wish to perform dynamic track generation in conjunction with [Self::route].
	pub fn announce(&mut self, announced: AnnouncedConsumer) {
		self.publisher.announce(announced);
	}

	/// Optionally route unknown paths.
	/// This is advanced functionality if you wish to perform dynamic track generation in conjunction with [Self::announce].
	pub fn route(&mut self, router: RouterConsumer) {
		self.publisher.route(router);
	}

	/// Subscribe to a track and start receiving data over the network.
	pub fn subscribe(&self, track: Track) -> TrackConsumer {
		self.subscriber.subscribe(track, self.tracing_id)
	}

	/// Discover any tracks published by the remote matching a prefix.
	pub fn announced(&self, prefix: Path) -> AnnouncedConsumer {
		self.subscriber.announced(prefix, self.tracing_id)
	}

	/// Close the underlying WebTransport session.
	pub fn close(mut self, err: Error) {
		self.webtransport.close(err.to_code(), &err.to_string());
	}

	/// Block until the WebTransport session is closed.
	pub async fn closed(&self) -> Error {
		self.webtransport.closed().await.into()
	}
}

impl PartialEq for Session {
	fn eq(&self, other: &Self) -> bool {
		self.webtransport == other.webtransport
	}
}

impl Eq for Session {}

use std::{
	collections::{hash_map, HashMap},
	sync::{atomic, Arc},
};

use crate::{
	message,
	model::{Track, TrackConsumer},
	AnnouncedProducer, Error, Path, TrackProducer,
};

use moq_async::{spawn, Lock, OrClose};
use qlog_rs::{events::Event, moq_transfork::data::AnnounceStatus, writer::QlogWriter};

use super::{AnnouncedConsumer, Reader, Stream};

#[derive(Clone)]
pub(super) struct Subscriber {
	session: web_transport::Session,

	tracks: Lock<HashMap<Path, TrackProducer>>,
	subscribes: Lock<HashMap<u64, TrackProducer>>,
	next_id: Arc<atomic::AtomicU64>,
}

impl Subscriber {
	pub fn new(session: web_transport::Session) -> Self {
		Self {
			session,

			tracks: Default::default(),
			subscribes: Default::default(),
			next_id: Default::default(),
		}
	}

	/// Discover any tracks matching a prefix.
	pub fn announced(&self, prefix: Path, tracing_id: u64) -> AnnouncedConsumer {
		let producer = AnnouncedProducer::default();
		let consumer = producer.subscribe_prefix(prefix.clone());

		let mut session = self.session.clone();
		spawn(async move {
			let mut stream = match Stream::open(&mut session, message::ControlType::Announce, tracing_id).await {
				Ok(stream) => stream,
				Err(err) => {
					tracing::warn!(?err, "failed to open announce stream");
					return;
				}
			};

			if let Err(err) = Self::run_announce(&mut stream, prefix, producer, tracing_id)
				.await
				.or_close(&mut stream)
			{
				tracing::warn!(?err, "announced error");
			}
		});

		consumer
	}

	async fn run_announce(stream: &mut Stream, prefix: Path, mut announced: AnnouncedProducer, tracing_id: u64) -> Result<(), Error> {
		let msg = message::AnnouncePlease { prefix: prefix.clone() };

		QlogWriter::log_event(Event::moq_announce_please_created(msg.prefix.to_vec(), tracing_id));

		stream.writer.encode(&msg).await?;

		tracing::debug!(?prefix, "waiting for announcements");

		loop {
			tokio::select! {
				res = stream.reader.decode_maybe::<message::Announce>() => {
					match res? {
						// Handle the announce
						Some(announce) => Self::recv_announce(announce, &prefix, &mut announced, tracing_id)?,
						// Stop if the stream has been closed
						None => return Ok(()),
					}
				},
				// Stop if the consumer is no longer interested
				_ = announced.closed() => return Ok(()),
			}
		}
	}

	fn recv_announce(
		announce: message::Announce,
		prefix: &Path,
		announced: &mut AnnouncedProducer,
		tracing_id: u64
	) -> Result<(), Error> {
		match announce {
			message::Announce::Active { suffix } => {
				let path = prefix.clone().append(&suffix);
				tracing::debug!(?path, "active");
				if !announced.announce(path) {
					return Err(Error::Duplicate);
				}

				// TODO: Check if this is right
				QlogWriter::log_event(Event::moq_announce_parsed(AnnounceStatus::Active, vec![suffix.to_vec()], tracing_id));
			}
			message::Announce::Ended { suffix } => {
				let path = prefix.clone().append(&suffix);
				tracing::debug!(?path, "unannounced");
				if !announced.unannounce(&path) {
					return Err(Error::NotFound);
				}

				// TODO: Check if this is right
				QlogWriter::log_event(Event::moq_announce_parsed(AnnounceStatus::Ended, vec![suffix.to_vec()], tracing_id));
			}
			message::Announce::Live => {
				// TODO: Check if this is right
				QlogWriter::log_event(Event::moq_announce_parsed(AnnounceStatus::Live, vec![], tracing_id));
				announced.live();
			}
		};

		Ok(())
	}

	/// Subscribe to a given track.
	pub fn subscribe(&self, track: Track, tracing_id: u64) -> TrackConsumer {
		let path = track.path.clone();
		let (writer, reader) = track.clone().produce();

		// Check if we can deduplicate this subscription
		match self.tracks.lock().entry(path.clone()) {
			hash_map::Entry::Occupied(entry) => return entry.get().subscribe(),
			hash_map::Entry::Vacant(entry) => entry.insert(writer.clone()),
		};

		let mut this = self.clone();
		let id = self.next_id.fetch_add(1, atomic::Ordering::Relaxed);

		spawn(async move {
			if let Ok(mut stream) = Stream::open(&mut this.session, message::ControlType::Subscribe, tracing_id).await {
				if let Err(err) = this.run_subscribe(id, writer, &mut stream, tracing_id).await.or_close(&mut stream) {
					tracing::warn!(?err, "subscribe error");
				}
			}

			this.subscribes.lock().remove(&id);
			this.tracks.lock().remove(&path);
		});

		reader
	}

	#[tracing::instrument("subscribe", skip_all, fields(?id, track = ?track.path))]
	async fn run_subscribe(&mut self, id: u64, track: TrackProducer, stream: &mut Stream, tracing_id: u64) -> Result<(), Error> {
		self.subscribes.lock().insert(id, track.clone());

		let request = message::Subscribe {
			id,
			path: track.path.clone(),
			priority: track.priority,

			group_order: track.order,

			// TODO
			group_min: None,
			group_max: None,
		};

		QlogWriter::log_event(Event::moq_subscription_started_created(request.id, request.path.to_vec(), request.priority.try_into().unwrap(), request.group_order as u64, request.group_min, request.group_max, tracing_id));

		stream.writer.encode(&request).await?;

		// TODO use the response to correctly populate the track info
		let info: message::Info = stream.reader.decode().await?;

		QlogWriter::log_event(Event::moq_info_parsed(info.track_priority.try_into().unwrap(), info.group_latest, info.group_order as u64, tracing_id));

		tracing::info!(?info, "active");

		loop {
			tokio::select! {
				res = stream.reader.decode_maybe::<message::GroupDrop>() => {
					match res? {
						Some(drop) => {
							tracing::info!(?drop, "dropped");
							// TODO expose updates to application
							// TODO use to detect gaps
							// TODO: Maybe log
						},
						None => break,
					}
				}
				// Close when there are no more subscribers
				_ = track.unused() => break
			};
		}

		tracing::info!("done");

		Ok(())
	}

	pub async fn recv_group(&mut self, stream: &mut Reader, tracing_id: u64) -> Result<(), Error> {
		let group: message::Group = stream.decode().await?;

		QlogWriter::log_event(Event::moq_group_parsed(group.subscribe, group.sequence, tracing_id));

		self.recv_group_inner(stream, group, tracing_id).await.or_close(stream)
	}

	#[tracing::instrument("group", skip_all, err, fields(subscribe = ?group.subscribe, group = group.sequence))]
	pub async fn recv_group_inner(&mut self, stream: &mut Reader, group: message::Group, tracing_id: u64) -> Result<(), Error> {
		let mut group = {
			let mut subs = self.subscribes.lock();
			let track = subs.get_mut(&group.subscribe).ok_or(Error::Cancel)?;

			track.create_group(group.sequence)
		};

		while let Some(frame) = stream.decode_maybe::<message::Frame>().await? {
			let mut frame = group.create_frame(frame.size);
			let mut remain = frame.size;

			while remain > 0 {
				let chunk = stream.read(remain).await?.ok_or(Error::WrongSize)?;

				remain = remain.checked_sub(chunk.len()).ok_or(Error::WrongSize)?;
				tracing::trace!(size = chunk.len(), remain, "chunk");

				frame.write(chunk);
			}

			// TODO: Maybe add the payload
			QlogWriter::log_event(Event::moq_frame_parsed(Some(frame.size.try_into().unwrap()), None, tracing_id));
		}

		Ok(())
	}
}

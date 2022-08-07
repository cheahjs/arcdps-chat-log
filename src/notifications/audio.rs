use std::{
    fs::File,
    io::{Cursor, Read},
    path,
    sync::Arc,
    thread,
};

use anyhow::Context;
use log::{debug, error, info};
use rodio::Decoder;

pub const DEFAULT_PING: &[u8] = include_bytes!("../../sounds/ping.mp3");

/// Static sound data stored in memory.
/// It is `Arc`'ed, so cheap to clone.
#[derive(Clone, Debug)]
pub struct SoundData(Arc<[u8]>);

impl SoundData {
    /// Load the file at the given path and create a new `SoundData` from it.
    pub fn new<P: AsRef<path::Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let file = &mut File::open(path)?;
        SoundData::from_read(file)
    }

    /// Copies the data in the given slice into a new `SoundData` object.
    pub fn from_bytes(data: &[u8]) -> Self {
        SoundData(Arc::from(data))
    }

    /// Creates a `SoundData` from any `Read` object; this involves
    /// copying it into a buffer.
    pub fn from_read<R>(reader: &mut R) -> anyhow::Result<Self>
    where
        R: Read,
    {
        let mut buffer = Vec::new();
        let _ = reader.read_to_end(&mut buffer)?;

        Ok(SoundData::from(buffer))
    }
}

impl From<Arc<[u8]>> for SoundData {
    #[inline]
    fn from(arc: Arc<[u8]>) -> Self {
        SoundData(arc)
    }
}

impl From<Vec<u8>> for SoundData {
    fn from(v: Vec<u8>) -> Self {
        SoundData(Arc::from(v))
    }
}

impl From<Box<[u8]>> for SoundData {
    fn from(b: Box<[u8]>) -> Self {
        SoundData(Arc::from(b))
    }
}

impl AsRef<[u8]> for SoundData {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[derive(Debug)]
pub struct AudioTrack {
    data: Option<Cursor<SoundData>>,
    pub status_message: String,
}

impl AudioTrack {
    pub fn new() -> Self {
        Self {
            data: None,
            status_message: "".to_string(),
        }
    }

    pub fn load_from_path(&mut self, path: &str, default: &'static [u8]) -> anyhow::Result<()> {
        if path.is_empty() {
            self.data = Some(Cursor::new(SoundData::from_bytes(default)));
        } else {
            let sound_data = SoundData::new(path);
            match sound_data {
                Ok(sound_data) => self.data = Some(Cursor::new(sound_data)),
                Err(error) => {
                    self.data = None;
                    self.status_message = format!("Failed to read '{}': {}", path, error);
                    return Err(error).context("failed to read");
                }
            }
        }
        // Test if sound data can be decoded
        match Decoder::new(self.data.as_ref().unwrap().clone()) {
            Ok(_decoder) => self.status_message = format!("Loaded '{}' successfully", path),
            Err(error) => {
                self.data = None;
                self.status_message = format!("Failed to decode '{}': {}", path, error);
            }
        }

        Ok(())
    }

    pub fn play(&self, volume: i32) {
        debug!("playing sound");
        if self.data.is_none() {
            info!("no sound data to play: {}", self.status_message);
            return;
        }
        let data = self.data.as_ref().unwrap().clone();
        thread::spawn(move || match Self::internal_play(data, volume) {
            Ok(_) => debug!("sound played"),
            Err(err) => error!("failed to play sound: {}", err),
        });
    }

    fn internal_play(data: Cursor<SoundData>, volume: i32) -> anyhow::Result<()> {
        let (_stream, stream_handle) =
            rodio::OutputStream::try_default().context("failed to init stream")?;
        let sink = stream_handle
            .play_once(data)
            .context("failed to start playing")?;
        sink.set_volume(volume as f32 / 100f32);
        sink.sleep_until_end();
        Ok(())
    }
}

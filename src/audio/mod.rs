pub mod player;
pub mod sounds;

use std::{
    fs::File,
    io::{Cursor, Read},
    path,
    sync::Arc,
};

use anyhow::Context;
use log::{debug, info};
use rodio::{Decoder, OutputStreamHandle, Sink};

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

#[derive(Debug, Clone)]
pub struct AudioTrack {
    data: Option<Cursor<SoundData>>,
    pub status_message: String,
    path: String,
    volume: i32,
}

impl AudioTrack {
    pub fn new() -> Self {
        Self {
            data: None,
            status_message: "".to_string(),
            path: "".to_string(),
            volume: 100,
        }
    }

    pub fn load_from_path(
        &mut self,
        path: &str,
        default: &'static [u8],
        volume: i32,
    ) -> anyhow::Result<()> {
        self.path = path.to_string();
        self.volume = volume;
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

    pub fn set_volume(&mut self, volume: i32) {
        self.volume = volume;
    }

    pub fn play(self, stream_handle: &OutputStreamHandle) -> anyhow::Result<()> {
        debug!("playing sound {}", self.path);
        match self.data {
            Some(data) => {
                let input =
                    Decoder::new(data).context(format!("failed to decode {}", self.path))?;
                let sink = Sink::try_new(stream_handle)
                    .context(format!("failed to create sink {}", self.path))?;
                sink.set_volume(self.volume as f32 / 100f32);
                sink.append(input);
                sink.detach();
                Ok(())
            }
            None => {
                info!("no sound data to play: {}", self.status_message);
                Err(anyhow::anyhow!(
                    "no sound data to play for {}: {}",
                    self.path,
                    self.status_message
                ))
            }
        }
    }
}

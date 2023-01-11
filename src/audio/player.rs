use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
};

use log::error;

use super::AudioTrack;

pub enum AudioSignal {
    PlayTrack(AudioTrack),
    Terminate,
}

pub struct AudioPlayer {
    sender: Option<Sender<AudioSignal>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        thread::spawn(move || Self::start_event_loop(receiver));
        Self {
            sender: Some(sender),
        }
    }

    pub fn play_track(&self, track: &AudioTrack) {
        if let Some(sender) = &self.sender {
            if let Err(err) = sender.send(AudioSignal::PlayTrack(track.clone())) {
                error!("error sending audio: {:#}", err);
            }
        }
    }

    pub fn release(&self) {
        if let Some(sender) = &self.sender {
            let _ = sender.send(AudioSignal::Terminate);
        }
    }

    fn start_event_loop(receiver: Receiver<AudioSignal>) {
        match rodio::OutputStream::try_default() {
            Ok((_stream, stream_handle)) => loop {
                match receiver.recv() {
                    Ok(event) => match event {
                        AudioSignal::PlayTrack(track) => {
                            if let Err(err) = track.play(&stream_handle) {
                                error!("failed to play track: {:#}", err);
                            }
                        }
                        AudioSignal::Terminate => break,
                    },
                    Err(err) => {
                        error!("failed to recieve audio signals: {:#}", err);
                    }
                }
            },
            Err(err) => error!("failed to create output stream: {:#}", err),
        }
    }
}

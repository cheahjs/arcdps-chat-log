use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
};

use log::error;

use super::AudioTrack;

pub enum AudioSignal {
    PlayTrack(AudioTrack),
    SetDevice(Option<String>),
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

    pub fn set_device(&self, device: Option<String>) {
        if let Some(sender) = &self.sender {
            if let Err(err) = sender.send(AudioSignal::SetDevice(device)) {
                error!("error sending set device: {:#}", err);
            }
        }
    }

    pub fn release(&self) {
        if let Some(sender) = &self.sender {
            let _ = sender.send(AudioSignal::Terminate);
        }
    }

    fn start_event_loop(receiver: Receiver<AudioSignal>) {
        use rodio::cpal::traits::{DeviceTrait, HostTrait};

        let mut current_device_name: Option<String> = None;
        let mut stream_data: Option<rodio::OutputStream> =
            rodio::OutputStreamBuilder::open_default_stream().ok();

        loop {
            match receiver.recv() {
                Ok(event) => match event {
                    AudioSignal::PlayTrack(track) => {
                        if let Some(stream) = &stream_data {
                            if let Err(err) = track.play(stream.mixer()) {
                                error!("failed to play track: {:#}", err);
                            }
                        } else {
                            error!("no output stream available to play track");
                        }
                    }
                    AudioSignal::SetDevice(device_name) => {
                        if device_name != current_device_name {
                            if let Some(name) = &device_name {
                                let host = rodio::cpal::default_host();
                                let mut devices = match host.output_devices() {
                                    Ok(d) => d,
                                    Err(e) => {
                                        error!("failed to list output devices: {:#}", e);
                                        current_device_name = None;
                                        stream_data =
                                            rodio::OutputStreamBuilder::open_default_stream().ok();
                                        continue;
                                    }
                                };
                                let device =
                                    devices.find(|d| d.name().map(|n| n == *name).unwrap_or(false));

                                if let Some(device) = device {
                                    match rodio::OutputStreamBuilder::from_device(device) {
                                        Ok(builder) => match builder.open_stream_or_fallback() {
                                            Ok(stream) => {
                                                stream_data = Some(stream);
                                                current_device_name = device_name;
                                            }
                                            Err(err) => {
                                                error!("failed to create output stream from device '{}': {:#}", name, err);
                                                current_device_name = None;
                                                stream_data = rodio::OutputStreamBuilder::open_default_stream().ok();
                                            }
                                        },
                                        Err(err) => {
                                            error!("failed to create output stream builder from device '{}': {:#}", name, err);
                                            current_device_name = None;
                                            stream_data =
                                                rodio::OutputStreamBuilder::open_default_stream()
                                                    .ok();
                                        }
                                    }
                                } else {
                                    error!(
                                        "audio device '{}' not found, falling back to default",
                                        name
                                    );
                                    current_device_name = None;
                                    stream_data =
                                        rodio::OutputStreamBuilder::open_default_stream().ok();
                                }
                            } else {
                                current_device_name = None;
                                stream_data =
                                    rodio::OutputStreamBuilder::open_default_stream().ok();
                            }
                        }
                    }
                    AudioSignal::Terminate => break,
                },
                Err(err) => {
                    error!("failed to receive audio signals: {:#}", err);
                    break;
                }
            }
        }
    }
}

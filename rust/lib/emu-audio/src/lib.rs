pub mod channels;
pub mod manager;
pub mod messages;
use channels::{ChannelID, ChannelType};
use manager::ChannelManager;
use messages::Message;

use std::thread;
use std::sync::mpsc;

pub struct EmuAudio {
  channels: Vec<ChannelType>,
  tx: Option<mpsc::Sender<Message>>,
}

impl EmuAudio {
  pub fn new() -> EmuAudio {
    EmuAudio {
      channels: Vec::new(),
      tx: None,
    }
  }

  pub fn add_channel(&mut self, channel_type: ChannelType) -> ChannelID {
    self.channels.push(channel_type);
    if let Some(tx) = &self.tx {
      tx.send(Message::AddChannel(channel_type)).unwrap();
    }
    self.channels.len() as ChannelID - 1
  }

  pub fn enable_channel(&self, id: ChannelID) {
    if let Some(tx) = &self.tx {
      tx.send(Message::EnableChannel(id)).unwrap();
    }
  }

  pub fn disable_channel(&self, id: ChannelID) {
    if let Some(tx) = &self.tx {
      tx.send(Message::DisableChannel(id)).unwrap();
    }
  }

  pub fn set_frequency(&self, id: ChannelID, freq: f32) {
    //println!("SEND FREQ {}", freq);
    if let Some(tx) = &self.tx {
      tx.send(Message::SetFrequency(id, freq)).unwrap();
    }
  }

  pub fn set_volume(&self, id: ChannelID, vol: f32) {
    if let Some(tx) = &self.tx {
      tx.send(Message::SetVolume(id, vol)).unwrap();
    }
  }

  pub fn set_duty(&self, id: ChannelID, duty: f32) {
    if let Some(tx) = &self.tx {
      tx.send(Message::SetVolume(id, duty)).unwrap();
    }
  }

  pub fn enable_envelope(&self, id: ChannelID) {
    if let Some(tx) = &self.tx {
      tx.send(Message::EnableEnvelope(id)).unwrap();
    }
  }

  pub fn disable_envelope(&self, id: ChannelID) {
    if let Some(tx) = &self.tx {
      tx.send(Message::DisableEnvelope(id)).unwrap();
    }
  }

  pub fn set_attack_time(&self, id: ChannelID, time: f32) {
    if let Some(tx) = &self.tx {
      tx.send(Message::SetAttackTime(id, time)).unwrap();
    }
  }

  pub fn set_decay_time(&self, id: ChannelID, time: f32) {
    if let Some(tx) = &self.tx {
      tx.send(Message::SetDecayTime(id, time)).unwrap();
    }
  }

  pub fn set_sustain_level(&self, id: ChannelID, level: f32) {
    if let Some(tx) = &self.tx {
      tx.send(Message::SetSustainLevel(id, level)).unwrap();
    }
  }

  pub fn set_release_time(&self, id: ChannelID, time: f32) {
    if let Some(tx) = &self.tx {
      tx.send(Message::SetReleaseTime(id, time)).unwrap();
    }
  }

  pub fn press_note(&self, id: ChannelID) {
    if let Some(tx) = &self.tx {
      tx.send(Message::PressNote(id)).unwrap();
    }
  }

  pub fn release_note(&self, id: ChannelID) {
    if let Some(tx) = &self.tx {
      tx.send(Message::ReleaseNote(id)).unwrap();
    }
  }

  pub fn play_note_for_time(&self, id: ChannelID, time: f32) {
    if let Some(tx) = &self.tx {
      tx.send(Message::PlayNoteForTime(id, time)).unwrap();
    }
  }

  pub fn start(&mut self) {
    let (tx, rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();

    thread::spawn(move || {
      let device = cpal::default_output_device().unwrap();
      let format = device.default_output_format().unwrap();
      let event_loop = cpal::EventLoop::new();
      let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
      event_loop.play_stream(stream_id.clone());
      let sample_rate = format.sample_rate.0 as u32;

      let mut manager = ChannelManager::new(sample_rate);

      // Blocking loop, calls back into our next_value lambda
      event_loop.run(move |_, data| {
        match data {
          cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
            for sample in buffer.chunks_mut(format.channels as usize) {
              let mut should_read = true;
              while should_read {
                match rx.try_recv() {
                  Ok(msg) => manager.handle_message(msg),
                  Err(_) => should_read = false,
                }
              }
              let value = manager.get_next_sample();
              for out in sample.iter_mut() {
                *out = value;
              }
            }
          },
          _ => (),
        }
      });
    });

    self.tx = Some(tx);
  }
}
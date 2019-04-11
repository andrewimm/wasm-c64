use crate::channels::{Channel, ChannelType};
use crate::messages::Message;

pub struct ChannelManager {
  channels: Vec<Channel>,
  sample_rate: u32,
}

impl ChannelManager {
  pub fn new(sample_rate: u32) -> ChannelManager {
    ChannelManager {
      channels: Vec::new(),
      sample_rate: sample_rate,
    }
  }

  pub fn get_next_sample(&mut self) -> f32 {
    let mut total = 0.0;
    for chan in self.channels.iter_mut() {
      total += chan.get_next_sample();
    }
    total / self.channels.len() as f32
  }

  pub fn handle_message(&mut self, msg: Message) {
    match msg {
      Message::AddChannel(channel_type) => {
        let mut chan = Channel::new(channel_type, self.sample_rate);
        self.channels.push(chan);
      },
      Message::EnableChannel(id) => {
        match self.channels.get_mut(id as usize) {
          Some(chan) => {
            chan.enable();
          },
          None => (),
        }
      },
      Message::DisableChannel(id) => {
        match self.channels.get_mut(id as usize) {
          Some(chan) => {
            chan.disable();
          },
          None => (),
        }
      },
      Message::SetVolume(id, vol) => {
        match self.channels.get_mut(id as usize) {
          Some(chan) => {
            chan.set_volume(vol);
          },
          None => (),
        }
      },
      Message::SetFrequency(id, freq) => {
        //println!("FREQ {}", freq);
        match self.channels.get_mut(id as usize) {
          Some(chan) => {
            chan.set_frequency(freq);
          },
          None => (),
        }
      },
      Message::SetDuty(id, duty) => {
        println!("DUTY");
        match self.channels.get_mut(id as usize) {
          Some(chan) => {
            chan.set_duty(duty);
          },
          None => (),
        }
      },
      Message::EnableEnvelope(id) => {
        match self.channels.get_mut(id as usize) {
          Some(chan) => {
            chan.enable_envelope();
          },
          None => (),
        }
      }
      Message::DisableEnvelope(id) => {
        match self.channels.get_mut(id as usize) {
          Some(chan) => {
            chan.disable_envelope();
          },
          None => (),
        }
      },
      Message::SetAttackTime(id, time) => {
        match self.channels.get_mut(id as usize) {
          Some(chan) => {
            chan.set_attack_time(time);
          },
          None => (),
        }
      },
      Message::SetDecayTime(id, time) => {
        match self.channels.get_mut(id as usize) {
          Some(chan) => {
            chan.set_decay_time(time);
          },
          None => (),
        }
      },
      Message::SetSustainLevel(id, level) => {
        match self.channels.get_mut(id as usize) {
          Some(chan) => {
            chan.set_sustain_level(level);
          },
          None => (),
        }
      },
      Message::SetReleaseTime(id, time) => {
        match self.channels.get_mut(id as usize) {
          Some(chan) => {
            chan.set_release_time(time);
          },
          None => (),
        }
      },
      Message::PressNote(id) => {
        match self.channels.get_mut(id as usize) {
          Some(chan) => {
            chan.press_note();
          },
          None => (),
        }
      },
      Message::ReleaseNote(id) => {
        match self.channels.get_mut(id as usize) {
          Some(chan) => {
            chan.release_note();
          },
          None => (),
        }
      },
      Message::PlayNoteForTime(id, time) => {
        match self.channels.get_mut(id as usize) {
          Some(chan) => {
            chan.play_note_for_time(time);
          },
          None => (),
        }
      },
    }
  }
}
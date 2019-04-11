pub type ChannelID = u8;

#[derive(Copy, Clone, Debug)]
pub enum ChannelType {
  Square,
  Triangle,
  Sawtooth,
  Noise,
}

pub struct Channel {
  channel_type: ChannelType,
  sample_rate: u32, // Samples per second
  sample_count: u32,
  volume: f32,
  enabled: bool,
  freq: f32,
  duty: f32,

  // ADSR Envelope
  envelope_enabled: bool,
  attack_time: f32,
  decay_time: f32,
  sustain_level: f32,
  release_time: f32,
  note_pressed: bool,
  note_duration: f32,
  note_press_timer: f32, // Time since the note was pressed (A/D/S)
  note_release_timer: f32, // Time since the note was released (R)
}

impl Channel {
  pub fn new(channel_type: ChannelType, sample_rate: u32) -> Channel {
    Channel {
      channel_type: channel_type,
      sample_rate: sample_rate,
      sample_count: 0,
      volume: 0.0,
      enabled: false,
      freq: 440.0,
      duty: 0.5,

      envelope_enabled: false,
      attack_time: 0.0,
      decay_time: 0.0,
      sustain_level: 1.0,
      release_time: 0.0,
      note_pressed: false,
      note_duration: -1.0,
      note_press_timer: 0.0,
      note_release_timer: 0.0,
    }
  }

  // Get the amplitude at a point in time relative to the wave's period,
  // measured on a range from 0.0 to 1.0.
  // This will be the same regardless of frequency.
  pub fn get_amplitude_at_time(&self, time: f32) -> f32 {
    match self.channel_type {
      ChannelType::Square => if time < self.duty { 1.0 } else { 0.0 },
      ChannelType::Triangle => if time < 0.5 { 1.0 - (1.0 - 4.0 * time).abs() } else { (1.0 - 4.0 * (time - 0.5)).abs() - 1.0 },
      ChannelType::Sawtooth => 1.0 - 2.0 * time,
      ChannelType::Noise => 0.0, // TODO: implement
    }
  }

  // For accuracy, this needs to be called at the internal sample rate
  pub fn get_next_sample(&mut self) -> f32 {
    if self.freq < 1.0 {
      return 0.0;
    }
    self.sample_count = (self.sample_count + 1) % self.sample_rate;
    let time = ((self.sample_count as f32) * self.freq / (self.sample_rate as f32)).fract();

    let amp = self.get_amplitude_at_time(time);
    self.increment_envelope_timers();
    let envelope = self.get_envelope_level();
    let volume = self.get_current_volume();
    amp * volume * envelope
  }

  pub fn enable(&mut self) {
    self.enabled = true;
  }

  pub fn disable(&mut self) {
    self.enabled = false;
  }

  pub fn set_volume(&mut self, volume: f32) {
    let mut v = volume;
    if v < 0.0 {
      v = 0.0;
    }
    if v > 1.0 {
      v = 1.0;
    }
    self.volume = v;
  }

  pub fn set_frequency(&mut self, freq: f32) {
    self.freq = freq;
  }

  pub fn set_duty(&mut self, duty: f32) {
    let mut d = duty;
    if d < 0.0 {
      d = 0.0;
    }
    if d > 1.0 {
      d = 1.0;
    }
    self.duty = d;
  }

  pub fn get_current_volume(&self) -> f32 {
    if !self.enabled {
      return 0.0;
    }
    return self.volume;
  }

  /* Envelope functionality */

  pub fn enable_envelope(&mut self) {
    self.envelope_enabled = true;
  }

  pub fn disable_envelope(&mut self) {
    self.envelope_enabled = false;
  }

  pub fn set_attack_time(&mut self, time: f32) {
    self.attack_time = time;
  }

  pub fn set_decay_time(&mut self, time: f32) {
    self.decay_time = time;
  }

  pub fn set_sustain_level(&mut self, level: f32) {
    let mut sus = level;
    if sus > 1.0 {
      sus = 1.0;
    }
    if sus < 0.0 {
      sus = 0.0;
    }
    self.sustain_level = sus;
  }

  pub fn set_release_time(&mut self, time: f32) {
    if self.note_release_timer >= self.release_time {
      // Ensure we don't trigger an unintended note
      self.note_release_timer = time + 1.0;
    }
    self.release_time = time;
  }

  pub fn increment_envelope_timers(&mut self) {
    if !self.envelope_enabled {
      return;
    }
    let increment = 1.0 / (self.sample_rate as f32);
    if self.note_pressed {
      self.note_press_timer += increment;
    } else if self.note_release_timer < self.release_time {
      self.note_release_timer += increment;
    }
    if self.note_duration > 0.0 {
      self.note_duration -= increment;
      if self.note_duration <= 0.0 {
        self.note_pressed = false;
      }
    }
  }

  pub fn press_note(&mut self) {
    self.note_pressed = true;
    self.note_press_timer = 0.0;
    self.note_release_timer = 0.0;
  }

  pub fn release_note(&mut self) {
    self.note_pressed = false;
  }

  pub fn play_note_for_time(&mut self, time: f32) {
    self.press_note();
    self.note_duration = time;
  }

  pub fn get_envelope_level(&self) -> f32 {
    if !self.envelope_enabled {
      // if envelope is disabled, we play a continuous tone
      return 1.0;
    }
    if self.note_pressed {
      if self.note_press_timer < self.attack_time {
        // in the attack phase
        return self.note_press_timer / self.attack_time;
      }
      if self.note_press_timer < self.attack_time + self.decay_time {
        // in the decay phase
        let progress = (self.note_press_timer - self.attack_time) / self.decay_time;
        return 1.0 - progress * (1.0 - self.sustain_level);
      }
      // in the sustain phase
      return self.sustain_level;
    }
    if self.note_release_timer > self.release_time || self.release_time == 0.0 {
      // note has ended
      return 0.0;
    }
    return (1.0 - self.note_release_timer / self.release_time) * self.sustain_level;
  }
}
use emuaudio::EmuAudio;
use emuaudio::channels::{ChannelID, ChannelType};
use nesmemmap::apu::APU;

const NTSC_CLOCK: f32 = 1789773.0;

const LENGTH_LOOKUP: [f32; 32] = [
  10.0, 254.0, 20.0, 2.0, 40.0, 4.0, 80.0, 6.0, 160.0, 8.0, 60.0, 10.0, 14.0, 12.0, 26.0, 14.0,
  12.0, 16.0, 24.0, 18.0, 48.0, 20.0, 96.0, 22.0, 192.0, 24.0, 72.0, 26.0, 16.0, 28.0, 32.0, 30.0,
];

pub struct APUImpl {
  audio: EmuAudio,
  square_channel_0: ChannelID,
  square_channel_1: ChannelID,
  triangle_channel: ChannelID,

  test_channel: ChannelID,

  square_0_timer: u16,
  square_1_timer: u16,
  triangle_timer: u16,

  square_0_length: f32,
  square_1_length: f32,
  triangle_length: f32,
}

impl APUImpl {
  pub fn new() -> APUImpl {
    let mut audio = EmuAudio::new();
    audio.start();
    let square_channel_0 = audio.add_channel(ChannelType::Square);
    let square_channel_1 = audio.add_channel(ChannelType::Square);
    let triangle_channel = audio.add_channel(ChannelType::Triangle);
    audio.enable_envelope(triangle_channel);
    audio.set_volume(triangle_channel, 1.0);

    let test_channel = audio.add_channel(ChannelType::Square);
    audio.enable_channel(test_channel);
    audio.enable_envelope(test_channel);
    audio.set_attack_time(test_channel, 0.1);
    audio.set_decay_time(test_channel, 0.05);
    audio.set_sustain_level(test_channel, 0.8);
    audio.set_release_time(test_channel, 0.1);
    audio.set_volume(test_channel, 1.0);

    APUImpl {
      audio: audio,
      square_channel_0: square_channel_0,
      square_channel_1: square_channel_1,
      triangle_channel: triangle_channel,

      test_channel: test_channel,

      square_0_timer: 0,
      square_1_timer: 0,
      triangle_timer: 0,

      square_0_length: 0.0,
      square_1_length: 0.0,
      triangle_length: 0.0,
    }
  }

  fn set_square_properties(&mut self, id: ChannelID, duty: u8, loop_control: bool, constant_vol: bool, volume_or_envelope: u8, length: f32) {
    let duty_percent = match duty {
      0 => 0.125,
      1 => 0.25,
      2 => 0.5,
      3 => 0.75,
      _ => 0.5,
    };
    self.audio.set_duty(id, duty_percent);
    if constant_vol {
      if loop_control {
        // play a single tone at a constant volume
        let vol = volume_or_envelope as f32 / 15.0;
        self.audio.set_volume(id, vol);
        self.audio.disable_envelope(id);
      } else {
        // play a tone that ends after the length
        let vol = volume_or_envelope as f32 / 15.0;
        self.audio.set_volume(id, vol);
        self.audio.set_release_time(id, 0.0);
        self.audio.enable_envelope(id);
        self.audio.play_note_for_time(id, length);
      }
    } else {
      if loop_control {
        // loop the sound

      } else {
        self.audio.enable_envelope(id);
        let length = (volume_or_envelope as f32 + 1.0) / 15.0;
        self.audio.set_release_time(id, length);
        self.audio.set_volume(id, 1.0);
        self.audio.press_note(id);
        self.audio.release_note(id);
      }
    }
  }
}

impl APU for APUImpl {
  fn toggle_square_0(&mut self, enabled: bool) {
    if enabled {
      self.audio.enable_channel(self.square_channel_0);
    } else {
      self.audio.disable_channel(self.square_channel_0);
      self.square_0_length = 0.0;
    }
  }

  fn toggle_square_1(&mut self, enabled: bool) {
    if enabled {
      self.audio.enable_channel(self.square_channel_1);
    } else {
      self.audio.disable_channel(self.square_channel_1);
      self.square_1_length = 0.0;
    }
  }

  fn toggle_triangle(&mut self, enabled: bool) {
    if enabled {
      self.audio.enable_channel(self.triangle_channel);
    } else {
      self.audio.disable_channel(self.triangle_channel);
      self.triangle_length = 0.0;
    }
  }

  fn set_square_0_properties(&mut self, duty: u8, loop_control: bool, constant_vol: bool, volume_or_envelope: u8) {
    self.set_square_properties(self.square_channel_0, duty, loop_control, constant_vol, volume_or_envelope, self.square_0_length);
  }

  fn set_square_1_properties(&mut self, duty: u8, loop_control: bool, constant_vol: bool, volume_or_envelope: u8) {
    self.set_square_properties(self.square_channel_1, duty, loop_control, constant_vol, volume_or_envelope, self.square_1_length);
  }

  fn set_triangle_properties(&mut self, control: bool, counter_reload: u8) {
    if control {

    }
  }

  fn set_square_0_timer_low(&mut self, low: u8) {
    let high = self.square_0_timer & 0xf0;
    self.square_0_timer = high | (low as u16);
    let t = self.square_0_timer as f32;
    let mut f = NTSC_CLOCK / (16.0 * (t + 1.0));
    if t < 8.0 {
      f = 0.0;
    }
    self.audio.set_frequency(self.square_channel_0, f);
  }

  fn set_square_0_timer_high(&mut self, high: u8) {
    let low = self.square_0_timer & 0x0f;
    self.square_0_timer = ((high as u16) << 8) | low;
    let t = self.square_0_timer as f32;
    let mut f = NTSC_CLOCK / (16.0 * (t + 1.0));
    if t < 8.0 {
      f = 0.0;
    }
    self.audio.set_frequency(self.square_channel_0, f);
  }

  fn set_square_1_timer_low(&mut self, low: u8) {
    let high = self.square_1_timer & 0xf0;
    self.square_1_timer = high | (low as u16);
    let t = self.square_1_timer as f32;
    let mut f = NTSC_CLOCK / (16.0 * (t + 1.0));
    if t < 8.0 {
      f = 0.0;
    }
    self.audio.set_frequency(self.square_channel_1, f);
  }

  fn set_square_1_timer_high(&mut self, high: u8) {
    let low = self.square_1_timer & 0x0f;
    self.square_1_timer = ((high as u16) << 8) | low;
    let t = self.square_1_timer as f32;
    let mut f = NTSC_CLOCK / (16.0 * (t + 1.0));
    if t < 8.0 {
      f = 0.0;
    }
    self.audio.set_frequency(self.square_channel_1, f);
  }

  fn set_triangle_timer_low(&mut self, low: u8) {
    let high = self.triangle_timer & 0xf0;
    self.triangle_timer = high | (low as u16);
    let t = self.triangle_timer as f32;
    let f = NTSC_CLOCK / (32.0 * (t + 1.0));
    self.audio.set_frequency(self.triangle_channel, f);
  }

  fn set_triangle_timer_high(&mut self, high: u8) {
    let low = self.triangle_timer & 0x0f;
    self.triangle_timer = ((high as u16) << 8) | low;
    let t = self.triangle_timer as f32;
    let f = NTSC_CLOCK / (32.0 * (t + 1.0));
    self.audio.set_frequency(self.triangle_channel, f);
  }

  fn set_square_0_length(&mut self, length: u8) {
    self.square_0_length = LENGTH_LOOKUP[length as usize] / 60.0;
  }

  fn set_square_1_length(&mut self, length: u8) {
    self.square_1_length = LENGTH_LOOKUP[length as usize] / 60.0;
  }

  fn set_triangle_length(&mut self, length: u8) {
    self.triangle_length = LENGTH_LOOKUP[length as usize] / 60.0;
  }

  fn test_note(&mut self) {
    self.audio.play_note_for_time(self.test_channel, 1.0);
  }
}
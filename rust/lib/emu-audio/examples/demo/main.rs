use emuaudio::EmuAudio;
use emuaudio::channels::ChannelType;

fn main() {
  let mut audio = EmuAudio::new();
  audio.start();
  let channel = audio.add_channel(ChannelType::Square);
  audio.set_volume(channel, 1.0);
  audio.enable_channel(channel);
  loop {}
}
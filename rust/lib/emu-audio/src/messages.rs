use crate::channels::{ChannelID, ChannelType};

#[derive(Debug)]
pub enum Message {
  AddChannel(ChannelType),
  EnableChannel(ChannelID),
  DisableChannel(ChannelID),
  SetVolume(ChannelID, f32), // volume from 0.0 to 1.0
  SetFrequency(ChannelID, f32), // frequency
  SetDuty(ChannelID, f32), // duty from 0.0 to 1.0, only works for square waves
  EnableEnvelope(ChannelID),
  DisableEnvelope(ChannelID),
  SetAttackTime(ChannelID, f32),
  SetDecayTime(ChannelID, f32),
  SetSustainLevel(ChannelID, f32),
  SetReleaseTime(ChannelID, f32),
  PressNote(ChannelID),
  ReleaseNote(ChannelID),
  PlayNoteForTime(ChannelID, f32),
}
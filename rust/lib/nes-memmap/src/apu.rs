pub trait APU {
  fn toggle_square_0(&mut self, enabled: bool);
  fn set_square_0_properties(&mut self, duty: u8, loop_control: bool, constant_vol: bool, volume_or_envelop: u8);
  fn set_square_0_timer_low(&mut self, low: u8);
  fn set_square_0_timer_high(&mut self, high: u8);
  fn toggle_square_1(&mut self, enabled: bool);
  fn set_square_1_properties(&mut self, duty: u8, loop_control: bool, constant_vol: bool, volume_or_envelop: u8);
  fn set_square_1_timer_low(&mut self, low: u8);
  fn set_square_1_timer_high(&mut self, high: u8);
  fn toggle_triangle(&mut self, enabled: bool);
  fn set_triangle_properties(&mut self, control: bool, counter_reload: u8);
  fn set_triangle_timer_low(&mut self, low: u8);
  fn set_triangle_timer_high(&mut self, high: u8);
  fn set_square_0_length(&mut self, length: u8);
  fn set_square_1_length(&mut self, length: u8);
  fn set_triangle_length(&mut self, length: u8);

  fn test_note(&mut self);
}
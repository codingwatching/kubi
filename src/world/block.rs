use strum::EnumIter;

#[derive(Clone, Copy, Debug, EnumIter)]
#[repr(u8)]
pub enum Block {
  Air,
  Stone,
  Dirt,
  Grass,
  Sand,
}

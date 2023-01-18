use strum::{EnumIter, IntoEnumIterator};
use crate::game::{
  items::Item,
  assets::textures::BlockTexture,
};


#[derive(Clone, Copy, Debug)]
pub enum CollisionType {
  Solid,
  Liquid,
  Ladder,
}

#[derive(Clone, Copy, Debug)]
pub enum RenderType {
  OpaqueBlock,
  TranslucentBlock,
  TranslucentLiquid,
  CrossShape
}

#[derive(Clone, Copy, Debug)]
pub struct BlockTextures {
  pub top: BlockTexture,
  pub bottom: BlockTexture,
  pub left: BlockTexture,
  pub right: BlockTexture,
  pub back: BlockTexture,
  pub front: BlockTexture,
}
impl BlockTextures {
  pub const fn all(tex: BlockTexture) -> Self {
    Self {
      top: tex,
      bottom: tex,
      left: tex,
      right: tex,
      back: tex,
      front: tex,
    }
  }
  pub const fn top_sides_bottom(top: BlockTexture, sides: BlockTexture, bottom: BlockTexture) -> Self {
    Self {
      top,
      bottom,
      left: sides,
      right: sides,
      back: sides,
      front: sides,
    }
  }
}

#[derive(Clone, Copy, Debug)]
pub struct BlockDescriptor {
  pub name: &'static str,
  pub id: &'static str,
  pub collision: Option<CollisionType>,
  pub raycast_collision: bool,
  pub render: Option<(RenderType, BlockTextures)>,
  pub item: Option<Item>,
}
impl BlockDescriptor {
  //Not using the Default trait because this function has to be const!
  pub const fn default() -> Self {
    Self {
      name: "default",
      id: "default",
      collision: Some(CollisionType::Solid),
      raycast_collision: true,
      render: Some((RenderType::OpaqueBlock, BlockTextures::all(BlockTexture::Stone))),
      item: None
    }
  }
}

#[derive(Clone, Copy, Debug, EnumIter)]
pub enum Block {
  Air,
  Stone,
  Dirt,
  Grass,
  Sand,
}
impl Block {
  //TODO make this O(1) with compile-time computed maps
  pub fn get_by_id(id: &str) -> Option<Self> {
    for block in Self::iter() {
      if block.descriptor().id == id {
        return Some(block)
      }
    }
    None
  }

  pub const fn descriptor(self) -> BlockDescriptor {
    match self {
      Self::Air => BlockDescriptor {
        name: "Air",
        id: "air",
        collision: None,
        raycast_collision: false,
        render: None,
        item: None,
      },
      Self::Stone => BlockDescriptor {
        name: "Stone",
        id: "stone",
        collision: Some(CollisionType::Solid),
        raycast_collision: true,
        render: Some((RenderType::OpaqueBlock, BlockTextures::all(BlockTexture::Stone))),
        item: Some(Item::StoneBlock)
      },
      Self::Dirt => BlockDescriptor {
        name: "Dirt",
        id: "dirt",
        collision: Some(CollisionType::Solid),
        raycast_collision: true,
        render: Some((RenderType::OpaqueBlock, BlockTextures::all(BlockTexture::Dirt))),
        item: Some(Item::DirtBlock)
      },
      Self::Grass => BlockDescriptor {
        name: "Grass",
        id: "grass",
        collision: Some(CollisionType::Solid),
        raycast_collision: true,
        render: Some((RenderType::OpaqueBlock, BlockTextures::top_sides_bottom(BlockTexture::GrassTop, BlockTexture::GrassSide, BlockTexture::Dirt))),
        item: Some(Item::DirtBlock)
      },
      Self::Sand => BlockDescriptor { 
        name: "Sand",
        id: "sand",
        collision: Some(CollisionType::Solid),
        raycast_collision: true,
        render: Some((RenderType::OpaqueBlock, BlockTextures::all(BlockTexture::Sand))), //this is not a sand tex
        item: Some(Item::StoneBlock)
      }
    }
  }
}

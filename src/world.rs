use nohash_hasher::BuildNoHashHasher;
use shipyard::{Unique, AllStoragesView};
use glam::IVec3;
use hashbrown::HashMap;
use anyhow::{Result, Context};

pub mod chunk;
pub mod block;
pub mod render;
pub mod tasks;
pub mod loading;
pub mod mesh;
pub mod neighbors;
pub mod worldgen;

use chunk::{Chunk, ChunkMesh};
use tasks::ChunkTaskManager;

//TODO separate world struct for render data
// because this is not send-sync

#[derive(Default, Unique)]
#[track(Modification)]
pub struct ChunkStorage {
  pub chunks: HashMap<IVec3, Chunk>
}
impl ChunkStorage {
  pub fn new() -> Self {
    Self::default()
  }
}

#[derive(Unique)]
pub struct WorldInfo {
  pub seed: u32,
}

#[derive(Default, Unique)]
pub struct ChunkMeshStorage {
  meshes: HashMap<usize, ChunkMesh, BuildNoHashHasher<usize>>,
  index: usize,
}
impl ChunkMeshStorage {
  pub fn new() -> Self {
    Self {
      meshes: HashMap::with_capacity_and_hasher(250, BuildNoHashHasher::default()),
      index: 0,
    }
  }
  pub fn insert(&mut self, mesh: ChunkMesh) -> usize {
    let index = self.index;
    self.meshes.insert_unique_unchecked(index, mesh);
    self.index += 1;
    index
  }
  pub fn update(&mut self, key: usize, mesh: ChunkMesh) -> Result<()> {
    *self.meshes.get_mut(&key).context("Chunk doesn't exist")? = mesh;
    Ok(())
  }
  pub fn remove(&mut self, key: usize) -> Result<()> {
    self.meshes.remove(&key).context("Chunk doesn't exist")?;
    Ok(())
  }
  pub fn get(&self, key: usize) -> Option<&ChunkMesh> {
    self.meshes.get(&key)
  }
}

pub fn init_game_world(
  storages: AllStoragesView,
) {
  storages.add_unique_non_send_sync(ChunkMeshStorage::new());
  storages.add_unique(ChunkStorage::new());
  storages.add_unique(ChunkTaskManager::new());
}

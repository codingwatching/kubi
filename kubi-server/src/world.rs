use shipyard::{Unique, UniqueView, UniqueViewMut, Workload, IntoWorkload, AllStoragesView};
use glam::IVec3;
use hashbrown::HashMap;
use kubi_shared::networking::messages::{ClientToServerMessage, ServerToClientMessage};
use kubi_udp::server::ServerEvent;
use crate::{
  server::{UdpServer, ServerEvents}, 
  config::ConfigTable,
  util::log_error, 
};

pub mod chunk;
pub mod tasks;

use chunk::Chunk;

use self::{tasks::{ChunkTaskManager, ChunkTask, ChunkTaskResponse}, chunk::ChunkState};

#[derive(Unique, Default)]
pub struct ChunkManager {
  pub chunks: HashMap<IVec3, Chunk>
}
impl ChunkManager {
  pub fn new() -> Self {
    Self::default()
  }
}

fn process_chunk_requests(
  mut server: UniqueViewMut<UdpServer>,
  events: UniqueView<ServerEvents>,
  mut chunk_manager: UniqueViewMut<ChunkManager>,
  task_manager: UniqueView<ChunkTaskManager>,
  config: UniqueView<ConfigTable>
) {
  for event in &events.0 {
    if let ServerEvent::MessageReceived { 
      from: client_id, 
      message: ClientToServerMessage::ChunkSubRequest { 
        chunk: chunk_position 
      } 
    } = event {
      let chunk_position = IVec3::from_array(*chunk_position);
      if let Some(chunk) = chunk_manager.chunks.get_mut(&chunk_position) {
        chunk.subscriptions.insert(*client_id);
        //TODO Start task here if status is "Nothing"
        if let Some(blocks) = &chunk.blocks {
          server.0.send_message(*client_id, kubi_shared::networking::messages::ServerToClientMessage::ChunkResponse {
            chunk: chunk_position.to_array(),
            data: blocks.clone(),
            queued: Vec::with_capacity(0)
          }).map_err(log_error).ok();
        }
      } else {
        let mut chunk = Chunk::new(chunk_position);
        chunk.state = ChunkState::Loading;
        chunk_manager.chunks.insert(chunk_position, chunk);
        task_manager.spawn_task(ChunkTask::LoadChunk {
          position: chunk_position,
          seed: config.world.seed,
        });
      }
    }
  }
}

fn process_finished_tasks(
  mut server: UniqueViewMut<UdpServer>,
  task_manager: UniqueView<ChunkTaskManager>,
  mut chunk_manager: UniqueViewMut<ChunkManager>,
) {
  while let Some(res) = task_manager.receive() {
    let ChunkTaskResponse::ChunkLoaded { chunk_position, blocks, queue } = res;
    let Some(chunk) = chunk_manager.chunks.get_mut(&chunk_position) else {
      log::warn!("Chunk discarded: Doesn't exist");
      continue
    };
    if chunk.state != ChunkState::Loading {
      log::warn!("Chunk discarded: Not Loading");
      continue
    }
    chunk.state = ChunkState::Loaded;
    chunk.blocks = Some(blocks.clone());
    for &subscriber in &chunk.subscriptions {
      server.0.send_message(subscriber, ServerToClientMessage::ChunkResponse {
        chunk: chunk_position.to_array(),
        data: blocks.clone(),
        queued: queue.iter().map(|item| (item.position.to_array(), item.block_type)).collect()
      }).map_err(log_error).ok();
    }
  }
}

fn init_chunk_manager(
  storages: AllStoragesView
) {
  storages.add_unique(ChunkManager::new());
}

pub fn init_world() -> Workload {
  (
    init_chunk_manager
  ).into_workload()
}

pub fn update_world() -> Workload {
  (
    process_chunk_requests,
    process_finished_tasks,
  ).into_workload()
}

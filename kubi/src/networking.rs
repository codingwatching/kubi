use shipyard::{Unique, AllStoragesView, UniqueView, UniqueViewMut, Workload, IntoWorkload, EntitiesViewMut, Component, ViewMut, SystemModificator, View, IntoIter, WorkloadModificator};
use glium::glutin::event_loop::ControlFlow;
use std::net::SocketAddr;
use uflow::{client::{Client, Config as ClientConfig, Event as ClientEvent}, SendMode};
use lz4_flex::decompress_size_prepended;
use anyhow::{Result, Context};
use kubi_shared::{
  networking::{
    messages::{ClientToServerMessage, ServerToClientMessage, S_SERVER_HELLO, S_CHUNK_RESPONSE, S_QUEUE_BLOCK},
    state::ClientJoinState, 
    channels::{CHANNEL_AUTH, CHANNEL_BLOCK},
  }, 
  queue::QueuedBlock
};
use crate::{
  events::{EventComponent, player_actions::PlayerActionEvent}, 
  control_flow::SetControlFlow, 
  world::{tasks::{ChunkTaskResponse, ChunkTaskManager}, queue::BlockUpdateQueue}, 
  state::is_ingame_or_loading
};

#[derive(Unique, Clone, Copy, PartialEq, Eq)]
pub enum GameType {
  Singleplayer,
  Muliplayer
}

#[derive(Unique, Clone, Copy, PartialEq, Eq)]
pub struct ServerAddress(pub SocketAddr);

#[derive(Unique)]
pub struct UdpClient(pub Client);

#[derive(Component)]
pub struct NetworkEvent(pub ClientEvent);

impl NetworkEvent {
  ///Checks if postcard-encoded message has a type
  pub fn is_message_of_type<const T: u8>(&self) -> bool {
    let ClientEvent::Receive(data) = &self.0 else { return false };
    if data.len() == 0 { return false }
    data[0] == T
  }
}

#[derive(Component)]
pub struct NetworkMessageEvent(pub ServerToClientMessage);

fn connect_client(
  storages: AllStoragesView
) {
  log::info!("Creating client");
  let address = storages.borrow::<UniqueView<ServerAddress>>().unwrap();
  let client = Client::connect(address.0, ClientConfig::default()).expect("Client connection failed");
  storages.add_unique(UdpClient(client));
  storages.add_unique(ClientJoinState::Disconnected);
}

fn poll_client(
  mut client: UniqueViewMut<UdpClient>,
  mut entities: EntitiesViewMut,
  mut events: ViewMut<EventComponent>,
  mut network_events: ViewMut<NetworkEvent>,
) {
  entities.bulk_add_entity((
    &mut events,
    &mut network_events,
  ), client.0.step().map(|event| {
    (EventComponent, NetworkEvent(event))
  }));
}

fn flush_client(
  mut client: UniqueViewMut<UdpClient>,
) {
  client.0.flush();
}

fn set_client_join_state_to_connected(
  mut join_state: UniqueViewMut<ClientJoinState>
) {
  log::info!("Setting ClientJoinState");
  *join_state = ClientJoinState::Connected;
}

fn say_hello(
  mut client: UniqueViewMut<UdpClient>,
) {
  log::info!("Authenticating");
  client.0.send(
    postcard::to_allocvec(
      &ClientToServerMessage::ClientHello {
        username: "Sbeve".into(),
        password: None
      }
    ).unwrap().into_boxed_slice(),
    CHANNEL_AUTH,
    uflow::SendMode::Reliable
  );
}

fn check_server_hello_response(
  network_events: View<NetworkEvent>,
  mut join_state: UniqueViewMut<ClientJoinState>
) {
  for event in network_events.iter() {
    let ClientEvent::Receive(data) = &event.0 else {
      continue
    };
    if !event.is_message_of_type::<S_SERVER_HELLO>() {
      continue
    }
    let Ok(parsed_message) = postcard::from_bytes(data) else {
      log::error!("Malformed message");
      continue
    };
    let ServerToClientMessage::ServerHello { init } = parsed_message else {
      unreachable!()
    };
    //TODO handle init data
    *join_state = ClientJoinState::Joined;
    log::info!("Joined the server!");
    return;
  }
}

//TODO multithreaded decompression
fn decompress_chunk_packet(data: &Box<[u8]>) -> Result<ServerToClientMessage> {
  let mut decompressed = decompress_size_prepended(&data[1..])?;
  decompressed.insert(0, data[0]);
  Ok(postcard::from_bytes(&decompressed).ok().context("Deserialization failed")?)
}

//TODO get rid of this, this is awfulll
fn inject_network_responses_into_manager_queue(
  manager: UniqueView<ChunkTaskManager>,
  events: View<NetworkEvent>
) {
  for event in events.iter() {
    if event.is_message_of_type::<S_CHUNK_RESPONSE>() {
      let NetworkEvent(ClientEvent::Receive(data)) = &event else { unreachable!() };
      let packet = decompress_chunk_packet(data).expect("Chunk decode failed");
      let ServerToClientMessage::ChunkResponse {
        chunk, data, queued
      } = packet else { unreachable!() };
      manager.add_sussy_response(ChunkTaskResponse::LoadedChunk {
        position: chunk, 
        chunk_data: data,
        queued
      });
    }
  }
}

fn send_block_place_events(
  action_events: View<PlayerActionEvent>,
  mut client: UniqueViewMut<UdpClient>,
) {
  for event in action_events.iter() {
    let PlayerActionEvent::UpdatedBlock { position, block } = event else {
      continue
    };
    client.0.send(
      postcard::to_allocvec(&ClientToServerMessage::QueueBlock {
        item: QueuedBlock { 
          position: *position, 
          block_type: *block, 
          soft: false
        }
      }).unwrap().into_boxed_slice(), 
      CHANNEL_BLOCK, 
      SendMode::Reliable,
    );
  }
}

fn recv_block_place_events(
  mut queue: UniqueViewMut<BlockUpdateQueue>,
  network_events: View<NetworkEvent>,
) {
  for event in network_events.iter() {
    let ClientEvent::Receive(data) = &event.0 else {
      continue
    };
    if !event.is_message_of_type::<S_QUEUE_BLOCK>() {
      continue
    }
    let Ok(parsed_message) = postcard::from_bytes(data) else {
      log::error!("Malformed message");
      continue
    };
    let ServerToClientMessage::QueueBlock { item } = parsed_message else {
      unreachable!()
    };
    queue.push(item);
  }
}

pub fn update_networking() -> Workload {
  (
    connect_client.run_if_missing_unique::<UdpClient>(),
    poll_client,
    (
      set_client_join_state_to_connected,
      say_hello,
    ).into_sequential_workload().run_if(if_just_connected),
    (
      check_server_hello_response
    ).run_if(is_join_state::<{ClientJoinState::Connected as u8}>),
    (
      recv_block_place_events
    ).run_if(is_join_state::<{ClientJoinState::Joined as u8}>).run_if(is_ingame_or_loading),
    inject_network_responses_into_manager_queue.run_if(is_ingame_or_loading).skip_if_missing_unique::<ChunkTaskManager>(),
  ).into_sequential_workload()
}

pub fn update_networking_late() -> Workload {
  (
    send_block_place_events.run_if(is_join_state::<{ClientJoinState::Joined as u8}>),
    flush_client,
  ).into_sequential_workload()
}

pub fn disconnect_on_exit(
  control_flow: UniqueView<SetControlFlow>,
  mut client: UniqueViewMut<UdpClient>,
) {
  if let Some(ControlFlow::ExitWithCode(_)) = control_flow.0 {
    if client.0.is_active() {
      client.0.flush();
      client.0.disconnect();
      while client.0.is_active() { client.0.step().for_each(|_|()); }
      log::info!("Client disconnected");
    } else {
      log::info!("Client inactive")
    }
    // if let Err(error) = client.0. {
    //   log::error!("failed to disconnect: {}", error);
    // } else {
    //   log::info!("Client disconnected");
    // }
  }
}

// conditions

fn if_just_connected(
  network_events: View<NetworkEvent>,
) -> bool {
  network_events.iter().any(|event| matches!(&event.0, ClientEvent::Connect))
}

fn is_join_state<const STATE: u8>(
  join_state: UniqueView<ClientJoinState>
) -> bool {
  (*join_state as u8) == STATE
}

pub fn is_multiplayer(
  game_type: UniqueView<GameType>
) -> bool {
  *game_type == GameType::Muliplayer
}

pub fn is_singleplayer(
  game_type: UniqueView<GameType>
) -> bool {
  *game_type == GameType::Singleplayer
}

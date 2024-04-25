use shipyard::{AllStoragesView, IntoIter, NonSendSync, Unique, UniqueView, UniqueViewMut, View};
use crate::{events::InputDeviceEvent, rendering::Renderer};
use winit::{
  event::{DeviceEvent, ElementState, RawKeyEvent},
  keyboard::{KeyCode, PhysicalKey},
  window::CursorGrabMode
};

#[derive(Unique)]
pub struct CursorLock(pub bool);

pub fn update_cursor_lock_state(
  lock: UniqueView<CursorLock>,
  display: NonSendSync<UniqueView<Renderer>>
) {
  if cfg!(target_os = "android") {
    return
  }
  if lock.is_inserted_or_modified() {
    //TODO MIGRATION
    let window = &display.window;
    window.set_cursor_grab(match lock.0 {
      true  => CursorGrabMode::Confined,
      false => CursorGrabMode::None,
    }).expect("Failed to change cursor grab state");
    window.set_cursor_visible(!lock.0);
  }
}

pub fn insert_lock_state(
  storages: AllStoragesView
) {
  storages.add_unique(CursorLock(false))
}

pub fn lock_cursor_now(
  mut lock: UniqueViewMut<CursorLock>
) {
  lock.0 = true
}

/// XXX: this is a huge hack
pub fn debug_toggle_lock(
  mut lock: UniqueViewMut<CursorLock>,
  device_events: View<InputDeviceEvent>,
) {
  for evt in device_events.iter() {
    if let DeviceEvent::Key(RawKeyEvent {
      physical_key: PhysicalKey::Code(KeyCode::F3),
      state: ElementState::Pressed,
    }) = evt.event {
      lock.0 = !lock.0;
    }
  }
}

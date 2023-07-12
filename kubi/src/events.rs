use glam::UVec2;
use shipyard::{World, Component, AllStoragesViewMut, SparseSet, NonSendSync, UniqueView};
use winit::event::{Event, DeviceEvent, DeviceId, WindowEvent, Touch};
use crate::rendering::Renderer;

pub mod player_actions;

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct EventComponent;

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct OnBeforeExitEvent;

#[derive(Component, Clone, Debug)]
pub struct InputDeviceEvent{
  pub device_id: DeviceId,
  pub event: DeviceEvent
}

#[derive(Component, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct TouchEvent(pub Touch);

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct WindowResizedEvent(pub UVec2);

pub fn process_glutin_events(world: &mut World, event: &Event<'_, ()>) {
  #[allow(clippy::collapsible_match, clippy::single_match)]
  match event {
    Event::WindowEvent { window_id: _, event } => match event {
      WindowEvent::Resized(size) => {
        world.add_entity((
          EventComponent,
          WindowResizedEvent(UVec2::new(size.width as _, size.height as _))
        ));
      },

      #[cfg(not(feature = "raw-evt"))]
      WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {
        world.add_entity((
          EventComponent,
          InputDeviceEvent {
            device_id: *device_id,
            event: DeviceEvent::Key(*input)
          }
        ));
      }

      WindowEvent::Touch(touch) => {
        world.add_entity((
          EventComponent,
          TouchEvent(*touch)
        ));
      }

      _ => ()
    },

    #[cfg(feature = "raw-evt")]
    Event::DeviceEvent { device_id, event } => {
      world.add_entity((
        EventComponent,
        InputDeviceEvent {
          device_id: *device_id,
          event: event.clone()
        }
      ));
    },

    Event::LoopDestroyed => {
      world.add_entity((
        EventComponent,
        OnBeforeExitEvent
      ));
    },
    
    _ => (),
  }
}

pub fn initial_resize_event(
  mut storages: AllStoragesViewMut,
) {
  let renderer = storages.borrow::<NonSendSync<UniqueView<Renderer>>>().unwrap();
  storages.add_entity((
    EventComponent,
    WindowResizedEvent(UVec2::new(renderer.size.width, renderer.size.height))
  ));
}

pub fn clear_events(
  mut all_storages: AllStoragesViewMut,
) {
  all_storages.delete_any::<SparseSet<EventComponent>>();
}

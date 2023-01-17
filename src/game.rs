use glam::Vec2;
use glium::{Surface, uniform};
use glium::uniforms::{Sampler, MinifySamplerFilter, MagnifySamplerFilter};
use glium::glutin::{
  event::{Event, WindowEvent, DeviceEvent},
  event_loop::{EventLoop, ControlFlow},
};
use std::time::Instant;

mod assets;
mod display;
mod shaders;
mod camera;
mod controller;
mod world;
mod blocks;
mod items;
mod options;

use assets::Assets;
use display::init_display;
use shaders::{Programs, chunk::Vertex as ChunkVertex};
use camera::Camera;
use controller::Controls;
use world::World;
use options::GameOptions;

struct State {
  pub camera: Camera,
  pub controls: Controls, 
  pub world: World
}
impl State {
  pub fn init() -> Self {
    Self {
      camera: Camera::default(),
      controls: Controls::default(),
      world: World::new(),
    }
  }
}

pub fn run() {
  log::info!("starting up");
  let event_loop = EventLoop::new();
  log::info!("initializing display");
  let display = init_display(&event_loop);
  log::info!("compiling shaders");
  let programs = Programs::compile_all(&display);
  log::info!("loading assets");
  let assets = Assets::load_all_sync(&display);
  log::info!("init game options");
  let options = GameOptions::default();
  log::info!("init game state");
  let mut state = State::init();
  state.camera.position = [0., 0., -1.];
  log::info!("game loaded");

  //=======================
  let vertex1 = ChunkVertex { position: [-0.5, -0.5, 0.], uv: [0., 0.], normal: [0., 1., 0.] };
  let vertex2 = ChunkVertex { position: [ 0.0,  0.5, 0.], uv: [0., 1.], normal: [0., 1., 0.] };
  let vertex3 = ChunkVertex { position: [ 0.5, -0.5, 0.], uv: [1., 1.], normal: [0., 1., 0.] };
  let shape = vec![vertex1, vertex2, vertex3];
  let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
  //=======================

  let mut last_render = Instant::now();
  
  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Poll;
    match event {
      // Mouse motion
      Event::DeviceEvent {
        event: DeviceEvent::MouseMotion{ delta, }, ..
      } => {
        state.controls.process_mouse_input(delta.0, delta.1);
        return
      }
      // Keyboard input
      Event::DeviceEvent { event: DeviceEvent::Key(input), .. } => {
        if let Some(key) = input.virtual_keycode {
          state.controls.process_keyboard_input(key, input.state);
        }
        return
      }
      // Window events
      Event::WindowEvent { event, .. } => {
        match event {
          WindowEvent::CloseRequested => {
            log::info!("exit requested");
            *control_flow = ControlFlow::Exit;
            return
          },
          _ => return
        }
      },
      Event::MainEventsCleared => (),
      _ => return
    }
    
    //Calculate delta time
    let now = Instant::now();
    let dt = (now - last_render).as_secs_f32();
    last_render = now;

    //Update controls
    state.controls.calculate(dt).apply_to_camera(&mut state.camera);

    //Load new chunks
    
    state.world.update_loaded_chunks(
      Vec2::new(state.camera.position[0], state.camera.position[2]), 
      &options, 
      &display
    );

    //Start drawing
    let mut target = display.draw();
    target.clear_color_and_depth((0.5, 0.5, 1., 1.), 1.);

    //Compute camera
    let target_dimensions = target.get_dimensions();
    let perspective = state.camera.perspective_matrix(target_dimensions);
    let view = state.camera.view_matrix();
    
    //Draw example triangle

    //Draw chunks
    state.world.render(&mut target, &programs, &assets, perspective, view);

    // target.draw(
    //   &vertex_buffer,
    //   glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList), 
    //   &programs.chunk, 
    //   &uniform! { 
    //     model: [
    //       [1., 0., 0., 0.],
    //       [0., 1., 0., 0.],
    //       [0., 0., 1., 0.],
    //       [0., 0., 0., 1.0_f32]
    //     ],
    //     view: view,
    //     perspective: perspective,
    //     tex: Sampler(&assets.textures.block_atlas, sampler_nearest)
    //   }, 
    //   &Default::default()
    // ).unwrap();

    //Finish drawing
    target.finish().unwrap();
  });
}

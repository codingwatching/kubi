use glium::implement_vertex;

#[derive(Clone, Copy)]
pub struct Vertex {
  pub position: [f32; 2]
}
implement_vertex!(Vertex, position);

pub const VERTEX_SHADER: &str = r#"
  #version 150 core

  in vec2 position;

  void main() {
    gl_Position = vec4(position, 0., 1.);
  }
"#;
pub const FRAGMENT_SHADER: &str = r#"
  #version 150 core

  out vec4 color;
  uniform vec4 u_color;

  void main() {
    color = u_color;
  }
"#;

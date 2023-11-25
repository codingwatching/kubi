use glam::vec2;
use crate::{
  LayoutInfo,
  UiDirection,
  element::UiElement,
  state::StateRepo,
  measure::Response,
  draw::UiDrawCommand
};

pub struct Spacer(f32);

impl Default for Spacer {
  fn default() -> Self {
    Self(5.)
  }
}

impl UiElement for Spacer {
  fn measure(&self, state: &StateRepo, layout: &LayoutInfo) -> Response {
    Response {
      desired_size: match layout.direction {
        UiDirection::Horizontal => vec2(self.0, 0.),
        UiDirection::Vertical => vec2(0., self.0),
      }
    }
  }

  fn process(&self, _measure: &Response, _state: &mut StateRepo, _layout: &LayoutInfo, _draw: &mut Vec<UiDrawCommand>) {}
}
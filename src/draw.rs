// custom Draw to intercept drawing calls when the scissor rect has a width or height of 0
// nannou::Draw does not intercept these calls so they get passed to wgpu, which will crash
pub(crate) struct Draw {
    scissor: Option<nannou::geom::Rect>,
    cur_draw: Option<nannou::Draw>,

    original_draw: nannou::Draw,
    dummy_draw: nannou::Draw, // this is where draw calls go if the scissor rect has a width or height of 0

}
impl Draw {
    pub(crate) fn new(draw: nannou::Draw, window_rect: nannou::geom::Rect) -> Draw {
        Draw { scissor: Some(window_rect), cur_draw: Some(draw.clone()), original_draw: draw, dummy_draw: nannou::Draw::new(), }
    }

    pub(crate) fn scissor(&self, scissor: nannou::geom::Rect) -> Draw {
        let new_scissor = self.scissor.and_then(|s| s.overlap(scissor)).and_then(|s| if s.w() < 1.0 || s.h() < 1.0 { None } else { Some(s) });
        // not sure if this is a bug in nannou or if this is intentional but the y coordinate of the scissor rect needs to be flipped
        let new_scissor_flipped_y = new_scissor.map(|s| nannou::geom::Rect::from_x_y_w_h(s.x(), -s.y(), s.w(), s.h()));

        Draw {
            scissor: new_scissor,
            cur_draw: new_scissor_flipped_y.map(|s| self.original_draw.scissor(s)),
            original_draw: self.original_draw.clone(),
            dummy_draw: self.dummy_draw.clone(),
        }
    }
}

impl std::ops::Deref for Draw {
    type Target = nannou::Draw;

    fn deref(&self) -> &Self::Target {
        self.cur_draw.as_ref().unwrap_or(&self.dummy_draw)
    }
}

// graphics utilities

#[allow(unused_imports)]
pub(crate) use sfml::{
    graphics::{CircleShape, Color, FloatRect, Font, Rect, RectangleShape, RenderTarget, RenderTexture, RenderWindow, Sprite, Text, Transformable},
    system::{Vector2, Vector2f, Vector2i, Vector2u},
};

pub(crate) trait RectCenter<T> {
    fn center(&self) -> Vector2<T>;
}
impl<T: std::ops::Div + std::ops::Add<<T as std::ops::Div>::Output, Output = T> + From<i8> + Copy> RectCenter<T> for Rect<T> {
    fn center(&self) -> Vector2<T> {
        Vector2::new(self.left + self.width / 2.into(), self.top + self.height / 2.into())
    }
}
pub(crate) trait CenterText {
    fn center(&mut self);
    fn center_horizontally(&mut self);
    fn center_vertically(&mut self);
}
impl CenterText for Text<'_> {
    fn center(&mut self) {
        let bounds = self.local_bounds();
        self.set_origin((bounds.width / 2.0, bounds.height / 2.0));
    }

    fn center_horizontally(&mut self) {
        let bounds = self.local_bounds();
        self.set_origin((bounds.width / 2.0, self.origin().y));
    }

    fn center_vertically(&mut self) {
        let boudns = self.local_bounds();
        self.set_origin((self.origin().x, boudns.height / 2.0));
    }
}

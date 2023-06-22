// graphics utilities

use sfml::graphics::Transformable;

pub(crate) trait RectCenter<T> {
    fn center(&self) -> sfml::system::Vector2<T>;
}
impl<T: std::ops::Div + std::ops::Add<<T as std::ops::Div>::Output, Output = T> + From<i8> + Copy> RectCenter<T> for sfml::graphics::Rect<T> {
    fn center(&self) -> sfml::system::Vector2<T> {
        sfml::system::Vector2::new(self.left + self.width / 2.into(), self.top + self.height / 2.into())
    }
}
pub(crate) trait CenterText {
    fn center(&mut self);
    fn center_horizontally(&mut self);
    fn center_vertically(&mut self);
}
impl CenterText for sfml::graphics::Text<'_> {
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

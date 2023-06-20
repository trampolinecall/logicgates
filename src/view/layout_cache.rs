use std::cell::RefCell;

pub(crate) struct LayoutCache<Layout>(RefCell<Option<(nannou::geom::Rect, Layout)>>);

impl<Layout> LayoutCache<Layout> {
    pub(crate) fn new() -> LayoutCache<Layout> {
        LayoutCache(RefCell::new(None))
    }

    pub(crate) fn with_layout<ComputeLayout: FnOnce(nannou::geom::Rect) -> Layout, R, F: FnOnce(&Layout) -> R>(&self, given_rect: nannou::geom::Rect, compute_layout: ComputeLayout, f: F) -> R {
        // layouts should really never need to be computed more than once because the view tree is supposed to only be used for one frame so any layouts should really never change
        // but conceptually the layout is supposed to be computed for whatever rect is passed into draw() or find_hover() so it has to be recomputed if the cache stores one for a different rect
        let mut layout_borrow = self.0.borrow_mut();
        let layout_field = &mut *layout_borrow;
        let needs_recompute = match layout_field {
            None => true,
            Some((old_given_rect, _)) if *old_given_rect != given_rect => true,
            _ => false,
        };

        if needs_recompute {
            let layout = compute_layout(given_rect);
            *layout_field = Some((given_rect, layout));
        }

        let layout = &layout_field.as_ref().expect("layout was either already computed or just computed").1;
        f(layout)
    }
}

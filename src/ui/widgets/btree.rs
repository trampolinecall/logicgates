use std::marker::PhantomData;

use crate::{
    graphics::{RenderTarget, Vector2f},
    theme::Theme,
    ui::widgets::button::ButtonState,
    view::{
        id::{ViewId, ViewIdMaker},
        lens::{self, Lens},
        GeneralEvent, SizeConstraints, TargetedEvent, View, ViewWithoutLayout,
    },
    App,
};

// TODO: clean all this up
// TODO: add way to convert split back into single

pub(crate) enum BTree<Child> {
    Single { child: Child, splith_button: ButtonState },
    HSplit { left: Box<BTree<Child>>, right: Box<BTree<Child>> }, // TODO: split amount
    VSplit { top: Box<BTree<Child>>, bottom: Box<BTree<Child>> }, // TODO: also split amount
}

impl<Child> BTree<Child> {
    pub(crate) fn new_single(c: Child) -> BTree<Child> {
        BTree::Single { child: c, splith_button: ButtonState::new() }
    }
}

enum BTreeView<Data, ChildView: ViewWithoutLayout<Data>, SplitHButtonView: ViewWithoutLayout<Data>> {
    Single { splith_button: SplitHButtonView, child_view: ChildView, _phantom: PhantomData<fn(&Data)> },
    HSplit { left: Box<BTreeView<Data, ChildView, SplitHButtonView>>, right: Box<BTreeView<Data, ChildView, SplitHButtonView>> },
    VSplit {},
}
enum BTreeLayout<'original, Data, ChildView: ViewWithoutLayout<Data> + 'original, SplitHButtonView: ViewWithoutLayout<Data> + 'original> {
    Single { splith_button_offset: Vector2f, splith_button: SplitHButtonView::WithLayout<'original>, child_view: ChildView::WithLayout<'original> },
    HSplit { left: Box<BTreeLayout<'original, Data, ChildView, SplitHButtonView>>, right: Box<BTreeLayout<'original, Data, ChildView, SplitHButtonView>> },
    VSplit {},
}

impl<Data, ChildView: ViewWithoutLayout<Data>, SplitHButtonView: ViewWithoutLayout<Data>> ViewWithoutLayout<Data> for BTreeView<Data, ChildView, SplitHButtonView> {
    type WithLayout<'without_layout> = BTreeLayout<'without_layout, Data, ChildView, SplitHButtonView> where Self: 'without_layout;

    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_> {
        match self {
            BTreeView::Single { splith_button, _phantom, child_view } => {
                // TODO: clean this up
                let child_layout = child_view.layout(sc);
                let splith_button_layout = splith_button.layout(SizeConstraints { min: Theme::DEFAULT.modify_ui_button_size.into(), max: Theme::DEFAULT.modify_ui_button_size.into() });
                let splith_button_offset = Vector2f::new(0.0, child_layout.size().y / 2.0 - Theme::DEFAULT.modify_ui_button_size.1 / 2.0);
                BTreeLayout::Single { splith_button: splith_button_layout, child_view: child_layout, splith_button_offset }
            }
            BTreeView::HSplit { left, right } => {
                let half_width_sc = SizeConstraints { min: Vector2f::new(0.0, 0.0), max: Vector2f::new(sc.max.x / 2.0, sc.max.y) };
                let left_layout = left.layout(half_width_sc);
                let right_layout = right.layout(half_width_sc);
                BTreeLayout::HSplit { left: Box::new(left_layout), right: Box::new(right_layout) }
            }
            BTreeView::VSplit {} => BTreeLayout::VSplit {},
        }
    }
}

impl<Data, ChildView: ViewWithoutLayout<Data>, SplitHButtonView: ViewWithoutLayout<Data>> View<Data> for BTreeLayout<'_, Data, ChildView, SplitHButtonView> {
    fn draw_inner(&self, app: &App, target: &mut dyn RenderTarget, top_left: Vector2f, hover: Option<ViewId>) {
        match self {
            BTreeLayout::Single { splith_button, child_view, splith_button_offset } => {
                child_view.draw(app, target, top_left, hover);
                // splith_button.draw(app, target, top_left + *splith_button_offset, hover); TODO: figure this out
            }
            BTreeLayout::HSplit { left, right } => todo!(),
            BTreeLayout::VSplit {} => todo!(),
        }
    }

    fn find_hover(&self, top_left: Vector2f, mouse: Vector2f) -> Option<ViewId> {
        match self {
            BTreeLayout::Single { splith_button, child_view, splith_button_offset } => {
                // splith_button.find_hover(top_left + *splith_button_offset, mouse).or(child_view.find_hover(top_left, mouse)) TODO: figure this out
                child_view.find_hover(top_left, mouse)
            }
            BTreeLayout::HSplit { left, right } => todo!(),
            BTreeLayout::VSplit {} => todo!(),
        }
    }

    fn size(&self) -> Vector2f {
        match self {
            BTreeLayout::Single { splith_button_offset: _, splith_button: _, child_view } => child_view.size(),
            BTreeLayout::HSplit { left, right } => todo!(),
            BTreeLayout::VSplit {} => todo!(),
        }
    }

    fn send_targeted_event(&self, app: &App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        match self {
            BTreeLayout::Single { splith_button, child_view, splith_button_offset: _ } => {
                splith_button.send_targeted_event(app, data, target, event);
                child_view.send_targeted_event(app, data, target, event);
            }
            BTreeLayout::HSplit { left, right } => todo!(),
            BTreeLayout::VSplit {} => todo!(),
        }
    }

    fn targeted_event(&self, _: &App, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&self, app: &App, data: &mut Data, event: GeneralEvent) {
        match self {
            BTreeLayout::Single { splith_button_offset: _, splith_button, child_view } => {
                splith_button.general_event(app, data, event);
                child_view.general_event(app, data, event);
            }
            BTreeLayout::HSplit { left, right } => todo!(),
            BTreeLayout::VSplit {} => todo!(),
        }
    }
}

pub(crate) struct BTreeChildLens<Data, Child, BTreeLens: Lens<Data, BTree<Child>>> {
    btree_lens: BTreeLens,
    get_child: fn(&BTree<Child>) -> &Child,
    get_child_mut: fn(&mut BTree<Child>) -> &mut Child,
    _phantom: PhantomData<fn(&Data) -> &Child>,
}
impl<Data, Child, BTreeLens: Lens<Data, BTree<Child>> + Clone> Clone for BTreeChildLens<Data, Child, BTreeLens> {
    fn clone(&self) -> BTreeChildLens<Data, Child, BTreeLens> {
        BTreeChildLens { btree_lens: self.btree_lens.clone(), get_child: self.get_child, get_child_mut: self.get_child_mut, _phantom: PhantomData }
    }
}
impl<Data, Child, BTreeLens: Lens<Data, BTree<Child>> + Copy> Copy for BTreeChildLens<Data, Child, BTreeLens> {}
impl<Data, Child, BTreeLens: Lens<Data, BTree<Child>>> Lens<Data, Child> for BTreeChildLens<Data, Child, BTreeLens> {
    fn with<'a, R: 'a, F: FnOnce(&Child) -> R>(&self, a: &Data, f: F) -> R {
        self.btree_lens.with(a, |btree| f((self.get_child)(btree)))
    }

    fn with_mut<'a, R: 'a, F: FnOnce(&mut Child) -> R>(&self, a: &mut Data, f: F) -> R {
        self.btree_lens.with_mut(a, |btree| f((self.get_child_mut)(btree)))
    }
}

pub(crate) fn btree<Child: Clone, BTreeLens: Lens<Data, BTree<Child>> + Copy, ChildView: ViewWithoutLayout<Data>, Data>(
    app: &App,
    id_maker: &mut ViewIdMaker,
    data: &Data,
    btree_lens: BTreeLens,
    view_child: impl Fn(&mut ViewIdMaker, BTreeChildLens<Data, Child, BTreeLens>, &Data) -> ChildView + Copy,
) -> impl ViewWithoutLayout<Data> {
    btree_lens.with(data, move |btree| match btree {
        BTree::Single { child: _, splith_button: _ } => {
            let splith_button = crate::ui::widgets::button::button(
                id_maker,
                data,
                lens::Compose::new(
                    btree_lens,
                    lens::Closures::new(
                        |btree: &_| match btree {
                            BTree::Single { child: _, splith_button } => splith_button,
                            BTree::HSplit { left: _, right: _ } => panic!("btree single lens used with hsplit btree"),
                            BTree::VSplit { top: _, bottom: _ } => panic!("btree single lens used with vsplit btree"),
                        },
                        |btree| match btree {
                            BTree::Single { child: _, splith_button } => splith_button,
                            BTree::HSplit { left: _, right: _ } => panic!("btree single lens used with hsplit btree"),
                            BTree::VSplit { top: _, bottom: _ } => panic!("btree single lens used with vsplit btree"),
                        },
                    ),
                ),
                move |_, data| {
                    btree_lens.with_mut(data, |btree| match btree {
                        BTree::Single { child, splith_button: _ } => {
                            *btree = BTree::HSplit { left: Box::new(BTree::new_single(child.clone())), right: Box::new(BTree::new_single(child.clone())) };
                        }

                        BTree::HSplit { left: _, right: _ } => panic!("splith button made for btree that is hsplit"),
                        BTree::VSplit { top: _, bottom: _ } => panic!("splith button made for btree that is vsplit"),
                    });
                },
            );

            fn get_child<Child>(bt: &BTree<Child>) -> &Child {
                match bt {
                    BTree::Single { child, splith_button: _ } => child,
                    BTree::HSplit { left: _, right: _ } => panic!("get_child called on hsplit btree"),
                    BTree::VSplit { top: _, bottom: _ } => panic!("get_child called on vsplit btree"),
                }
            }
            fn get_child_mut<Child>(bt: &mut BTree<Child>) -> &mut Child {
                match bt {
                    BTree::Single { child, splith_button: _ } => child,
                    BTree::HSplit { left: _, right: _ } => panic!("get_child_mut called on hsplit btree"),
                    BTree::VSplit { top: _, bottom: _ } => panic!("get_child_mut called on vsplit btree"),
                }
            }
            BTreeView::Single { splith_button, child_view: view_child(id_maker, BTreeChildLens { btree_lens, _phantom: PhantomData, get_child, get_child_mut }, data), _phantom: PhantomData }
        }
        BTree::HSplit { left, right } => {
            /* TODO
            let left_lens = lens::Closures::new(todo!(), todo!());
            let right_lens = lens::Closures::new(todo!(), todo!());
            BTreeView::HSplit {
                left: Box::new(self::btree(app, id_maker, data, lens::Compose::new(btree_lens, left_lens), view_child)),
                right: Box::new(self::btree(app, id_maker, data, lens::Compose::new(btree_lens, right_lens), view_child)),
            }
            */
            todo!()
        }
        BTree::VSplit { top, bottom } => BTreeView::VSplit {},
    })
}

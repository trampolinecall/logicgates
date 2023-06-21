macro_rules! flow {
    (horizontal $($rest:tt)*) => {
        flow!($crate::ui::widgets::flow::Direction::Horizontal; $($rest)*)
    };
    (vertical $($rest:tt)*) => {
        flow!($crate::ui::widgets::flow::Direction::Vertical; $($rest)*)
    };
    ($direction:expr; $( $name:ident : $e:expr ),* $(,)?) => {
        {
            #[allow(non_camel_case_types)]
            struct Container<Data, $($name: $crate::view::ViewWithoutLayout<Data>),*> {
                $(
                    $name: $name,
                )*

                _phantom: ::std::marker::PhantomData<fn(Data)>,
            }
            #[allow(non_camel_case_types)]
            struct ContainerWithLayout<'original, Data, $($name: $crate::view::ViewWithoutLayout<Data> + 'original),*> {
                own_size: ::nannou::geom::Vec2,
                $(
                    $name: (::nannou::geom::Vec2, $name::WithLayout<'original>),
                )*
            }

            #[allow(non_camel_case_types)]
            impl<Data, $($name: $crate::view::ViewWithoutLayout<Data>),*> $crate::view::ViewWithoutLayout<Data> for Container<Data, $($name),*> {
                type WithLayout<'s> = ContainerWithLayout<'s, Data, $($name),*> where $($name: 's,)* Data: 's;

                fn layout(&self, sc: view::SizeConstraints) -> ContainerWithLayout<'_, Data, $($name),*> {
                    $(
                        let $name = self.$name.layout($crate::ui::widgets::flow::layout::child_sc(sc));
                    )*
                    let own_size = $crate::ui::widgets::flow::layout::find_own_size($direction, sc, [$(&$name as &dyn $crate::view::View<_>),*]);

                    let mut cur_pos = $crate::ui::widgets::flow::layout::find_start_pos($direction, own_size);
                    $(
                        let $name = ($crate::ui::widgets::flow::layout::layout_step($direction, &mut cur_pos, &$name), $name);
                    )*

                    ContainerWithLayout { own_size, $($name),* }
                }
            }
            #[allow(non_camel_case_types)]
            impl<Data, $($name: $crate::view::ViewWithoutLayout<Data>),*> $crate::view::View<Data> for ContainerWithLayout<'_, Data, $($name),*> {
                fn draw_inner(&self, app: &::nannou::App, draw: &$crate::draw::Draw, center: ::nannou::geom::Vec2, hover: ::std::option::Option<$crate::view::id::ViewId>) {
                    $(
                        {
                            let (offset, child) = &self.$name;
                            child.draw(app, draw, center + *offset, hover);
                        }
                    )*
                }

                fn find_hover(&self, center: ::nannou::geom::Vec2, mouse: ::nannou::geom::Vec2) -> ::std::option::Option<$crate::view::id::ViewId> {
                    $(
                        {
                            let (offset, child) = &self.$name;
                            if let x @ Some(_) = child.find_hover(center + *offset, mouse) {
                                return x;
                            }
                        }
                    )*
                    None
                }

                fn size(&self) -> ::nannou::geom::Vec2 {
                    self.own_size
                }

                fn send_targeted_event(&self, app: &::nannou::App, data: &mut Data, target: $crate::view::id::ViewId, event: $crate::view::TargetedEvent) {
                    $(
                        self.$name.1.send_targeted_event(app, data, target, event);
                    )*
                }

                fn targeted_event(&self, _: &::nannou::App, _: &mut Data, _: $crate::view::TargetedEvent) {}
                fn general_event(&self, app: &::nannou::App, data: &mut Data, event: $crate::view::GeneralEvent) {
                    $(
                        self.$name.1.general_event(app, data, event);
                    )*
                }
            }

            Container {
                $(
                    $name: $e,
                )*
                _phantom: ::std::marker::PhantomData,
            }
        }
    };
}

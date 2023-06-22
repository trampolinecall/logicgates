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
                own_size: ::sfml::system::Vector2f,
                $(
                    $name: (::sfml::system::Vector2f, $name::WithLayout<'original>),
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

                    let mut cur_pos = 0.0;
                    $(
                        let $name = ($crate::ui::widgets::flow::layout::layout_step($direction, &mut cur_pos, &$name), $name);
                    )*

                    ContainerWithLayout { own_size, $($name),* }
                }
            }
            #[allow(non_camel_case_types)]
            impl<Data, $($name: $crate::view::ViewWithoutLayout<Data>),*> $crate::view::View<Data> for ContainerWithLayout<'_, Data, $($name),*> {
                fn draw_inner(&self, app: &$crate::App, target: &mut dyn sfml::graphics::RenderTarget, top_left: ::sfml::system::Vector2f, hover: ::std::option::Option<$crate::view::id::ViewId>) {
                    $(
                        {
                            let (offset, child) = &self.$name;
                            child.draw(app, target, top_left + *offset, hover);
                        }
                    )*
                }

                fn find_hover(&self, top_left: ::sfml::system::Vector2f, mouse: ::sfml::system::Vector2f) -> ::std::option::Option<$crate::view::id::ViewId> {
                    None
                    $(
                        .or({
                            let (offset, child) = &self.$name;
                            child.find_hover(top_left + *offset, mouse)
                        })
                    )*
                }

                fn size(&self) -> ::sfml::system::Vector2f {
                    self.own_size
                }

                fn send_targeted_event(&self, app: &$crate::App, data: &mut Data, target: $crate::view::id::ViewId, event: $crate::view::TargetedEvent) {
                    $(
                        self.$name.1.send_targeted_event(app, data, target, event);
                    )*
                }

                fn targeted_event(&self, _: &$crate::App, _: &mut Data, _: $crate::view::TargetedEvent) {}
                fn general_event(&self, app: &$crate::App, data: &mut Data, event: $crate::view::GeneralEvent) {
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

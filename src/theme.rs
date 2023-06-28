use sfml::graphics::Color;

pub(crate) struct Theme {
    pub(crate) animation_time: f32,
    pub(crate) animation_ease: fn(f32) -> f32,

    pub(crate) slide_out_toggle_y_offset: f32,
    pub(crate) modify_ui_button_size: (f32, f32), // things like the toggle button on slide over views or hsplit and vsplit buttons on btree views

    pub(crate) button_normal_bg: Color,
    pub(crate) button_normal_fg: Color,
    pub(crate) button_hover_bg: Color,
    pub(crate) button_hover_fg: Color,
    pub(crate) button_pressed_bg: Color,
    pub(crate) button_pressed_fg: Color,

    pub(crate) simulation_bg_color: Color,

    pub(crate) gate_color: Color,
    pub(crate) gate_hover_color: Color,
    pub(crate) gate_text_color: Color,
    pub(crate) gate_hover_dist: f32,

    pub(crate) on_color: Color,
    pub(crate) off_color: Color,
    pub(crate) high_impedance_color: Color,
    pub(crate) err_color: Color,
    pub(crate) node_hover_color: Color,
    pub(crate) node_rad: f32,
    pub(crate) node_hover_dist: f32,

    pub(crate) connection_width: f32,
    pub(crate) connection_hover_dist: f32,
}

impl Theme {
    pub(crate) const DEFAULT: Theme = Theme {
        animation_time: 0.2,
        animation_ease: Theme::cubic_ease_out,

        slide_out_toggle_y_offset: 30.0,
        modify_ui_button_size: (10.0, 30.0),

        button_normal_bg: Color::rgb(200, 200, 200),
        button_normal_fg: Color::rgb(0, 0, 0),
        button_hover_bg: Color::rgb(255, 255, 255),
        button_hover_fg: Color::rgb(0, 0, 0),
        button_pressed_bg: Color::rgb(100, 100, 100),
        button_pressed_fg: Color::rgb(0, 0, 0),

        simulation_bg_color: Color::rgb(180, 180, 180),

        gate_color: Color::rgb(100, 100, 100),
        gate_hover_color: Color::rgba(255, 255, 255, 50),
        gate_text_color: Color::rgb(255, 255, 255),
        gate_hover_dist: 5.0,
        on_color: Color::rgb(0, 255, 0),
        off_color: Color::rgb(50, 50, 50),
        high_impedance_color: Color::rgb(0, 0, 255),
        err_color: Color::rgb(255, 0, 0),
        node_hover_color: Color::rgba(255, 255, 255, 50),
        node_rad: 5.0,
        node_hover_dist: 4.0,
        connection_width: 2.5,
        connection_hover_dist: 4.0,
    };

    fn linear_ease(x: f32) -> f32 {
        x
    }
    fn cubic_ease_out(x: f32) -> f32 {
        (x - 1.0).powf(3.0) + 1.0
    }
}

type Rgb = nannou::color::rgb::Rgb<nannou::color::encoding::Srgb, u8>;
type Rgba = nannou::color::rgb::Rgba<nannou::color::encoding::Srgb, u8>;

const fn rgb(red: u8, green: u8, blue: u8) -> Rgb {
    Rgb { red, green, blue, standard: std::marker::PhantomData }
}
const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Rgba {
    Rgba { color: rgb(r, g, b), alpha: a }
}

pub(crate) struct Theme {
    pub(crate) animation_time: f32,
    pub(crate) animation_ease: fn(f32) -> f32,

    pub(crate) slide_out_toggle_y_offset: f32,
    pub(crate) slide_out_size: (f32, f32),

    pub(crate) button_normal_bg: Rgb,
    pub(crate) button_hover_bg: Rgb,
    pub(crate) button_pressed_bg: Rgb,

    pub(crate) simulation_bg_color: Rgb,

    pub(crate) gate_color: Rgb,
    pub(crate) gate_hover_color: Rgba,
    pub(crate) gate_text_color: Rgb,
    pub(crate) gate_hover_dist: f32,

    pub(crate) on_color: Rgb,
    pub(crate) off_color: Rgb,
    pub(crate) high_impedance_color: Rgb,
    pub(crate) err_color: Rgb,
    pub(crate) node_hover_color: Rgba,
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
        slide_out_size: (10.0, 30.0),

        button_normal_bg: rgb(200, 200, 200),
        button_hover_bg: rgb(255, 255, 255),
        button_pressed_bg: rgb(100, 100, 100),

        simulation_bg_color: rgb(180, 180, 180),
        gate_color: rgb(100, 100, 100),
        gate_hover_color: rgba(255, 255, 255, 50),
        gate_text_color: rgb(255, 255, 255),

        gate_hover_dist: 5.0,
        on_color: rgb(0, 255, 0),
        off_color: rgb(50, 50, 50),
        high_impedance_color: rgb(0, 0, 255),
        err_color: rgb(255, 0, 0),
        node_hover_color: rgba(255, 255, 255, 50),
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

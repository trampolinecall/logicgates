type Rgb = nannou::color::rgb::Rgb<nannou::color::encoding::Srgb, u8>;
type Rgba = nannou::color::rgb::Rgba<nannou::color::encoding::Srgb, u8>;

const fn rgb(red: u8, green: u8, blue: u8) -> Rgb {
    Rgb { red, green, blue, standard: std::marker::PhantomData }
}
const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Rgba {
    Rgba { color: rgb(r, g, b), alpha: a }
}

pub(crate) struct Theme {
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

pub(crate) const THEME: Theme = Theme {
    simulation_bg_color: rgb(0, 0, 0),

    gate_color: rgb(255, 255, 255),
    gate_hover_color: rgba(255, 255, 255, 50),
    gate_text_color: rgb(0, 0, 0),

    gate_hover_dist: 5.0,
    on_color: rgb(46, 204, 133),
    off_color: rgb(127, 140, 141),
    high_impedance_color: rgb(52, 152, 219),
    err_color: rgb(231, 76, 60),
    node_hover_color: rgba(255, 255, 255, 50),
    node_rad: 5.0,

    node_hover_dist: 4.0,
    connection_width: 2.5,
    connection_hover_dist: 4.0,
};

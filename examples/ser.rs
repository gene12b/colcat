use colcat::color::palettes::ColorPalette;

pub fn main() {
    let color_params = ColorParams {
        palette: ColorPalette::Uniform(csscolorparser::parse("#000000").unwrap()),
        background_color: None,
    };
    let text = serde_json::to_string(&color_params).unwrap();
    println!("{text}");
}

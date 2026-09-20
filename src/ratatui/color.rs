use crate::types::IntoExt;

//impl TryIntoExt for csscolorparser::Color {
//    type Output = ratatui::style::Color;
//    type Error = ColcatError;
//    fn try_into(self) -> Result<Self::Output, Self::Error> {
//        let color_components = self.to_hsla();
//        ratatui::style::Color::from_hsl(palette::Hsl::new_srgb(
//            color_components[0],
//            color_components[1],
//            color_components[2],
//        ))
//    }
//}

impl IntoExt for csscolorparser::Color {
    type Output = ratatui::style::Color;
    fn into_ext(self) -> Self::Output
    where
        Self: Sized,
    {
        let color_components = self.to_hsla();
        ratatui::style::Color::from_hsl(palette::Hsl::new_srgb(
            color_components[0],
            color_components[1],
            color_components[2],
        ))
    }
}

impl IntoExt for &csscolorparser::Color {
    type Output = ratatui::style::Color;
    fn into_ext(self) -> Self::Output
    where
        Self: Sized,
    {
        let color_components = self.to_hsla();
        ratatui::style::Color::from_hsl(palette::Hsl::new_srgb(
            color_components[0],
            color_components[1],
            color_components[2],
        ))
    }
}

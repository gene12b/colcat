use std::path::Path;

use colcat::{
    color::ColoredCharacter,
    errors::ColcatError,
    types::{
        params::{ColcatParams, ColcatParamsSerde},
        state::ColcatState,
    },
};
use docx_rust::{Docx, DocxFile, document::BodyContent};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.json");
    let config: ColcatParams = ColcatParamsSerde::try_from_path(config_path)?.try_into()?;
    let mut state = ColcatState::new(config, "".to_string())?;
    state.set_text("asdf".to_string());
    //println!("loaded config: {state:?}");
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("test.docx");
    //let file = File::open(path)?;
    //DocxFile must live as long as possible
    let docx_file = DocxFile::from_file(path)?;
    let mut docx = docx_file.parse()?;
    let docx_text = docx.extract_text()?;
    println!("docx_text: \n{docx_text:?}");
    state.set_text(docx_text);
    println!("Processing text...");
    state.process_text()?;
    let processed = &state.text_state.processed_text;
    let processed_text = String::from_iter(processed.iter().map(|val| val.character));
    println!("processed text: \n{processed_text:?}");

    //docx.colorize(processed);
    //println!("new docx: {docx:#?}");
    //println!("docx: {docx:#?}");

    Ok(())
}

//docx compresses different sequences of characters with properties called "Runs" into
//"Paragraphs". If I want to color each character differently, I need to convert Paragraphs
//with single Runs for all text within paragraph to a Run for every character in that text with
//its own coloring scheme. Then I can compress afterwards by combining Runs with equivalent
//coloring/properties.

trait ExtractText {
    fn extract_text(&self) -> Result<String, ColcatError>;
}

impl ExtractText for Docx<'_> {
    fn extract_text(&self) -> Result<String, ColcatError> {
        self.document.body.content.extract_text()
    }
}

impl ExtractText for Vec<BodyContent<'_>> {
    fn extract_text(&self) -> Result<String, ColcatError> {
        let mut ret_string = String::new();
        for content in self {
            match content {
                BodyContent::Sdt(val) => {
                    //"Structured Domain Tag"
                    ret_string.push_str(&val.text());
                    ret_string.push('\n');
                }
                BodyContent::Run(val) => {
                    ret_string.push_str(&val.text());
                    ret_string.push('\n');
                }
                BodyContent::Table(val) => {
                    //ret_string.push_str(&val.text());
                }
                BodyContent::TableCell(val) => {}
                BodyContent::Paragraph(val) => {
                    ret_string.push_str(&val.text());
                    ret_string.push('\n');
                }
                BodyContent::SectionProperty(val) => {}
            }
        }
        ret_string.pop();
        Ok(ret_string)
    }
}

trait ColorizeExt {
    fn colorize(&mut self, colors: &Vec<ColoredCharacter>) -> Result<(), ColcatError>
    where
        Self: Sized;
    fn colorize_iter(&mut self, colors: impl Iterator<Item = ColoredCharacter>) {}
}

//fg_color -> Run(Run.property<Option<CharacterProperty>>.color<Option<Color>>.value)
//bg_color -> Run(Run.property<Option<CharacterProperty>>.shading<Option<Shading>>.fill<Option<&str>>)
//impl ColorizeExt for Docx<'_> {
//    fn colorize(&mut self, colors: &Vec<ColoredCharacter>) -> Result<(), ColcatError>
//    where
//        Self: Sized,
//    {
//        for component in &mut self.document.body.content {
//            match component {
//                BodyContent::Paragraph(val) => {
//                    let para_content = val.content;
//                    for content in para_content {
//                        match content {
//                            docx_rust::document::ParagraphContent::Run(val) => val.colorize(colors),
//                        }
//                    }
//                }
//                _other => {}
//            }
//        }
//        Ok(())
//    }
//}

//asdf, basdf lasdfjk.
// aksdfj
//
//asdf, basdf lasdfjk aksdfj
//
//
//

//impl ColorizeExt for Run<'_> {
//    fn colorize(&mut self, colors: &Vec<ColoredCharacter>) -> Result<(), ColcatError>
//    where
//        Self: Sized,
//    {
//        match self.content {
//            RunContent::Text(val) => {
//                while let val.text.chars().next() == colors.iter().next() {
//                    val.text.set_style(style)
//                }
//            }
//        }
//        Ok(())
//    }
//    fn colorize_iter(&mut self, colors: impl Iterator<Item = ColoredCharacter>) {
//        match self.content {
//            RunContent::Text(val) => val.te,
//        }
//    }
//}

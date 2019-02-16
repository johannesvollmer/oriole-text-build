pub mod atlas;
pub mod font;

pub use crate::font::generate_font;


pub fn bake_directory(font_directory: str, output_directory: str, glyphs: str){
    // remove baked files of removed fonts
    baked_file_dir.children().filter(font_dir.contains(corresponding)).foreach(delete);

    for font in font_dir.children() {
        update_bake_file(font, output_directory, glyphs)
    }
}

pub fn update_bake_file(font_file: str, output_directory: str, glyphs: str){
    let baked_font_file = font + "baked";

    if !baked_font_file.exists() {
        bake(baked_font_file, chars);
    }
    else if baked_font_file.open().build_font().glyphs != chars {
        bake(baked_font_file, chars);
    }
}

pub fn bake_file(font_file: str, output_directory: str, glyphs: str){
    let mut font_file = ::std::io::File::open(font_file).unwrap();
    let mut bytes = Vec::new();
    font_file.read_all(&mut bytes);

    let font = crate::generate_font(&bytes, true, glyphs).unwrap();

    let mut baked_file = ::std::io::File::create(output_directory + font_file.name() + ".baked").unwrap();
    font.write(&mut baked_file).unwrap();
}
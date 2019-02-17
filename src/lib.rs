pub mod atlas;
pub mod font;

pub use crate::font::generate_font;
use std::{fs};
use std::io::{ Result, Read, Write };
use std::path::{ Path };
use std::fs::File;
use crate::font::BuildCongifuration;
use oriole_text::font::Font;


pub fn bake_font_directory(
    ttf_directory: &Path,
    bake_directory: &Path,
    configuration: BuildCongifuration,
    glyphs: impl Iterator<Item=char> + Clone,
) -> Result<()>
{
    if !bake_directory.exists() {
        fs::create_dir(bake_directory)?;
    }

    // remove all baked files whoose source files have been removed
    for baked in fs::read_dir(bake_directory)? {
        let baked = baked?.path();
        let ttf = ttf_directory.join(baked.file_name().unwrap());
        if !ttf.exists() { fs::remove_file(baked)?; }
    }

    // generate baked files or update existing, also do that for all subdirectories
    for ttf in fs::read_dir(ttf_directory)? {
        let entry = ttf?.path();

        if entry.ends_with(".ttf") || entry.ends_with(".otf") {
            /*let metadata = fs::metadata(&path)?;
            let last_modified = metadata.modified()?.elapsed()?.as_secs();
            if last_modified < 24 * 3600 && metadata.is_file() {}*/
            update_bake_file(&entry, bake_directory, configuration, glyphs)?;
        }
        else if entry.is_dir() {
            bake_font_directory(
                &entry, &bake_directory.join(entry.file_name().unwrap()),
                configuration, glyphs
            )?;
        }
    }

    Ok(())
}

pub fn update_bake_file(
    font_file: &Path,
    output_directory: &Path,
    configuration: BuildCongifuration,
    glyphs: impl Iterator<Item=char> + Clone,
) -> Result<()>
{
    let baked_font_file = output_directory
        .join(font_file.file_name().unwrap()).join(".baked_font");

//    if !baked_font_file.exists() {

    // unconditionally bake, for now.
    bake_file(font_file, &baked_font_file, configuration, glyphs)

//    }
//    else if baked_font_file.open().build_font().glyphs != chars {
//        bake(baked_font_file, chars);
//    }
}

pub fn bake_file(
    ttf_file: &Path,
    bake_file: &Path,
    configuration: BuildCongifuration,
    glyphs: impl Iterator<Item=char> + Clone,
) -> Result<()>
{
//    let font_file = File::open(ttf_file)?;
//    let mut bytes = Vec::new();
//    font_file.read_all(&mut bytes)?;
    let bytes: Vec<u8> = File::open(ttf_file)?.bytes().collect();

    let font = crate::generate_font(&bytes, configuration, glyphs).unwrap();

    let mut baked_file = File::create(bake_file).unwrap();
    Font::deserialized(font).write(&mut baked_file).unwrap();
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn glium() -> Result<()> {
        let bytes: Vec<u8> = File::open("fonts/Roboto-Regular.ttf")?.bytes().collect();

        let config = BuildCongifuration {
            char_resolution_y: 64,
            sdf_multisampling: 8,
            skip_unknown_chars: false
        };

        let font = crate::generate_font(
            &bytes, config,
            (0 .. 65000 ).map(|i| i as char)
        ).unwrap();

        oriole_text::test::glium(font);
        Ok(())
    }

}

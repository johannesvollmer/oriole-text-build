pub mod atlas;
pub mod font;




pub mod prelude {
    pub use crate::bake::bake_file;
    pub use crate::bake::bake_font_directory;
    pub use crate::font::generate_font;
    pub use crate::font::BuildConfiguration;
}

pub mod bake {
    use super::prelude::*;
    use std::{fs};
    use std::io::{ Read };
    use std::path::{ Path };
    use std::fs::File;

    #[derive(Debug)]
    pub enum Error {
        File(::std::io::Error),
        Bake((crate::font::Error)),
        Serialize(oriole_text::font::Error),
    }

    pub fn bake_font_directory(
        ttf_directory: &Path,
        bake_directory: &Path,
        configuration: BuildConfiguration,
        glyphs: impl Iterator<Item=char> + Clone,
    ) -> Result<(), Error>
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
                update_bake_file(&entry, bake_directory, configuration, glyphs.clone())?;
            }
            else if entry.is_dir() {
                bake_font_directory(
                    &entry, &bake_directory.join(entry.file_name().unwrap()),
                    configuration, glyphs.clone()
                )?;
            }
        }

        Ok(())
    }

    pub fn update_bake_file(
        font_file: &Path,
        output_directory: &Path,
        configuration: BuildConfiguration,
        glyphs: impl Iterator<Item=char> + Clone,
    ) -> Result<(), Error>
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
        configuration: BuildConfiguration,
        glyphs: impl Iterator<Item=char> + Clone,
    ) -> Result<(), Error>
    {
        let mut font_file = File::open(ttf_file)?;
        let mut bytes = Vec::new();
        font_file.read_to_end(&mut bytes)?;

        let font = generate_font(&bytes, configuration, glyphs)?;

        let mut baked_file = File::create(bake_file)?;
        font.write(&mut baked_file)?;
        Ok(())
    }

    impl From<::std::io::Error> for Error {
        fn from(error: ::std::io::Error) -> Self {
            Error::File(error)
        }
    }

    impl From<crate::font::Error> for Error {
        fn from(error: crate::font::Error) -> Self {
            Error::Bake(error)
        }
    }

    impl From<oriole_text::font::Error> for Error {
        fn from(error: oriole_text::font::Error) -> Self {
            Error::Serialize(error)
        }
    }
}


#[cfg(test)]
pub mod test {

    #[test]
    fn main2(){
        println!("testing!!!!!!");
    }

    #[test]
    fn main(){
        println!("testing!!!!!!");

        use crate::prelude::*;
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open("fonts/Roboto-Regular.ttf").unwrap();
        println!("opened file");

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();

        println!("finished reading file bytes");

        let configuration = BuildConfiguration {
            char_resolution_y: 64, // 128
            sdf_multisampling: 4,
            skip_unknown_chars: true,
            sdf_max_distance: 20
        };

        let font = generate_font(
            &bytes, configuration, (0..=191_u8).map(|u| u as char)
        ).unwrap();


        use std::path::Path;
        use std::io::BufWriter;
        use png::HasParameters;

        let path = Path::new("generated_images/sdf.png");
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);
        let mut encoder = png::Encoder::new(
            w, font.atlas.resolution.0 as u32, font.atlas.resolution.1 as u32
        );
        encoder.set(png::ColorType::Grayscale).set(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&font.atlas.distance_field).unwrap();



        let path = Path::new("generated_images/reconstructed.png");
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);
        let mut encoder = png::Encoder::new(
            w, font.atlas.resolution.0 as u32, font.atlas.resolution.1 as u32
        );
        encoder.set(png::ColorType::Grayscale).set(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        let reconstructed: Vec<u8> = font.atlas.distance_field.iter()
            .map(|u| if *u < 128 { 255 } else { 0 }).collect();
        writer.write_image_data(&reconstructed).unwrap();

        let path = Path::new("generated_images/font_uncompressed.baked");
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);
        font.write_uncompressed(w).unwrap();

        let path = Path::new("generated_images/font.baked");
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);
        font.write(w).unwrap();

    }
}
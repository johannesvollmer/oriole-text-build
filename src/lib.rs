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

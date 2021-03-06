use oriole_text::font::{ FontLayout, GlyphLayout, SerializedFont };
use oriole_text::rectangle::Rectangle;
use crate::atlas::Segment;


// hash all parameters to this font, for cached builds
/*pub fn hash_baking_parameters(glyphs: &[char], glyph_res) -> usize {
    let mut hasher = DefaultHasher::new();
    contained_chars.hash(&mut hasher);
    sdf_multisampling.hash(&mut hasher);
    char_resolution_y.hash(&mut hasher);
    let hash = hasher.finish();
}*/

#[derive(Clone, Copy)]
pub struct BuildConfiguration {
    pub char_resolution_y: usize,
    pub sdf_multisampling: usize,
    pub skip_unknown_chars: bool,
    pub sdf_max_distance: usize,
}

#[derive(Debug)]
pub enum Error {
    FontDecode(rusttype::Error),
    UnsupportedGlyph(char)
}

pub fn generate_font(
    ttf_bytes: &[u8],
    configuration: BuildConfiguration,
    chars: impl Iterator<Item=char> + Clone,
)
    -> Result<SerializedFont, Error>
{
    let rusttype_font = rusttype::Font::from_bytes(ttf_bytes)?;

    let mut contained_chars = Vec::new();

    let mut kerning = Vec::new();
    let mut glyphs = Vec::new();

    let proxy_scale_factor = (configuration.char_resolution_y * configuration.sdf_multisampling) as f32;
    let proxy_scale = rusttype::Scale::uniform(proxy_scale_factor);
    let proxy_position = rusttype::point(0.0, 0.0);

    let atlas = crate::atlas::generate_atlas(chars.clone().flat_map(|character|{

        println!("starting character '{}'", character);
        let glyph = rusttype_font.glyph(character);

        // skip unknown glyphs, except for the zero-glyph itself
        if glyph.id().0 == 0 && character as usize != 0 {
            println!("char not in font!");

            if configuration.skip_unknown_chars {
                return None;
            }
            else {
                // TODO return Err(Error::UnsupportedGlyph(character));
                panic!("unknown glyph");
            }
        }


        let glyph = glyph.scaled(proxy_scale)
            .positioned(proxy_position);

        if let Some(bounds) = glyph.pixel_bounding_box() {
            println!("bounding box: {:?}, dimensions: {:?}", bounds, (bounds.width(), bounds.height()));
            if bounds.width() == 0 || bounds.height() == 0 {
                panic!("dimension is 0 for `{}`", character);
            }

            contained_chars.push(character);

            let padding = configuration.sdf_max_distance;
            let width = bounds.width() as usize + padding * 2;
            let height = bounds.height() as usize + padding * 2;
            let x = bounds.min.x + padding as i32;
            let y = bounds.min.y + padding as i32;

            // collect glyph layout
            glyphs.push((character, GlyphLayout {
                advance_x: glyph.unpositioned().h_metrics().advance_width,
                bounds: Rectangle {
                    position: (x as f32 / proxy_scale_factor, y as f32 / proxy_scale_factor),
                    dimensions: (width as f32 / proxy_scale_factor, height as f32 / proxy_scale_factor),
                },
            }));

            // collect pair kerning
            for follower in chars.clone() {
                let pair_kerning = rusttype_font.pair_kerning(proxy_scale, character, follower);
                if pair_kerning.abs() > 0.00001 {
                    kerning.push(((character, follower), pair_kerning/*TODO / proxy_scale_factor*/));
                }
            }

            let mut binary_image = vec![0_u8; width * height];
            glyph.draw(|x, y, value|{
                binary_image[(y as usize + padding) * width + (x as usize + padding)] = {
                    if value > 0.5 { 255 } else { 0 }
                };
            });

            let distance_image = signed_distance_field::compute_f32_distance_field(
                &signed_distance_field::binary_image::of_byte_slice(&binary_image, width as u16, height as u16)
            );

            if let Some(distance_image) = distance_image
                .normalize_clamped_distances(-(configuration.sdf_max_distance as f32), configuration.sdf_max_distance as f32)
            {
                let distance_image = distance_image.to_u8();

                let char_width = (width as f32 / configuration.sdf_multisampling as f32).ceil() as usize;
                let char_height = (height as f32 / configuration.sdf_multisampling as f32).ceil() as usize;

                let mut resizer = resize::new(
                    width, height, char_width, char_height,
                    resize::Pixel::Gray8, resize::Type::Lanczos3
                );

                let mut downsampled_distances = vec![0_u8; char_width * char_height];
                resizer.resize(&distance_image, &mut downsampled_distances);

                let segment = Segment {
                    size: (char_width, char_height),
                    distance_field: downsampled_distances
                };

                // FIXME resize to a height of 64 instead of to a width of 64
                Some((character, segment))
            }

            else {
                println!("glyph did contain any shape (distance field is infinite) for `{}`", character);
                None
            }

        }
        else { // glyph did not contain any shape, e.g. ' ' or '\t'
            None
        }
    }));

    let metrics = rusttype_font.v_metrics_unscaled();


    Ok(SerializedFont {
        atlas,
        glyphs,
        kerning,
        layout: FontLayout {
            advance_y: metrics.line_gap + metrics.ascent,
//                space_advance_x: 0.0,
//                tab_advance_x: 0.0,
            descent: metrics.descent,
            ascent: metrics.ascent
        }
    })
}

impl From<rusttype::Error> for Error {
    fn from(error: rusttype::Error) -> Self {
        Error::FontDecode(error)
    }
}


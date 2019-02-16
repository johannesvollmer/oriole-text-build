use crate::atlas::{ Atlas, SerializedAtlas };
use hashbrown::HashMap;
use crate::rectangle::Rectangle;
use std::io::{ Read, Write };
use serde_derive::*;
use crate::atlas::Segment;
use crate::text::LayoutGlyphs;


pub struct Font {
    pub atlas: Atlas,
    pub glyphs: HashMap<char, GlyphLayout>,
    pub kerning: HashMap<(char, char), f32>,
    pub layout: FontLayout
}

#[derive(Serialize, Deserialize)]
pub struct SerializedFont {
    pub atlas: SerializedAtlas,
    pub glyphs: Vec<(char, GlyphLayout)>,
    pub kerning: Vec<((char, char), f32)>,
    pub layout: FontLayout
}

#[derive(Serialize, Deserialize)]
pub struct FontLayout {
    pub advance_y: f32,
//    pub space_advance_x: f32,
//    pub tab_advance_x: f32,
    pub ascent: f32,
    pub descent: f32,
}

#[derive(Serialize, Deserialize)]
pub struct GlyphLayout {
    pub bounds: Rectangle,
    pub advance_x: f32,
}


pub fn downsample_u8_image_to_width(width: usize, height: usize, bytes: &[u8], new_width: usize)
    -> (Vec<u8>, usize)
{
    let factor = new_width as f32 / width as f32;
    let new_height = (height as f32 * factor) as usize;

    let mut result = vec![0_u8; new_width * new_height];

    unimplemented!()

    (result, new_height)
}

impl Font {

    pub fn generate(path: &str, mut chars: impl Iterator<Item=char>) -> Self {
        let rusttype_font = rusttype::Font::from_bytes(path).unwrap();

        let mut kerning = HashMap::new();
        let mut glyphs = HashMap::new();

        let proxy_scale_factor = 30.0;
        let proxy_scale = rusttype::Scale::uniform(proxy_scale_factor);
        let proxy_position = rusttype::point(0.0, 0.0);

        let atlas = Atlas::generate(chars.and_then(|character|{
            let glyph = rusttype_font.glyph(character);

            // skip unknown glyphs, except for the zero-glyph itself
            if glyph.id().0 == 0 && character as usize != 0 {
                return None;
            }

            let glyph = glyph.glyph.scaled(proxy_scale)
                .positioned(proxy_position);

            if let Some(bounds) = glyph.bounding_box() {
                let width = bounds.width();
                let height = bounds.height();

                let binary_image = vec![0_u8; width * height];
                let distance_image = signed_distance_field::compute_f16_distance_field(
                    signed_distance_field::binary_image::BinaryByteImage::from_slice(width, height, &binary_image)
                );

                let downsampled_width = 64;
                if let Some(distance_image) = distance_image.normalized_clamped(-30.0, 30.0) {
                    let distance_image = distance_image.to_u8_image();
                    let (downsampled_distances, downsampled_height) = downsample_u8_image_to_width(
                        width, height, &distance_image, downsampled_width
                    );


                    let segment = Segment {
                        size: (downsampled_width, downsampled_height),
                        distance_field: downsampled_distances
                    };

                    // collect pair kerning
                    for follower in chars.clone() {
                        let pair_kerning = rusttype_font.pair_kerning(proxy_scale, character, follower);
                        if pair_kerning.abs() > 0.00001 {
                            kerning.insert((character, follower), kerning);
                        }
                    }

                    // collect glyph layout
                    glyphs.push(character, GlyphLayout {
                        advance_x: glyph.h_metrics().advance_width,
                        bounds: Rectangle {
                            position: (bounds.min.0 / proxy_scale_factor, bounds.min.1 / proxy_scale_factor),
                            dimensions: (width as f32 / proxy_scale_factor, height as f32 / proxy_scale_factor),
                        },
                    });

                    // FIXME resize to a height of 64 instead of to a width of 64
                    Some((character, segment))
                }

                // glyph did not contain any shape, e.g. ' ' or '\t'
                else {
                    None
                }

            }
            else {
                panic!("No bounding box???");
            }
        }));

        let metrics = rusttype_font.v_metrics_unscaled();

        Font {
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
        }
    }

    pub fn layout_glyphs<S>(&self, chars: S) -> LayoutGlyphs<S> where S: Iterator<Item=char> {
        LayoutGlyphs::new(self, chars)
    }

    pub fn read(reader: &mut impl Read) -> Option<Self> {
        let mut uncompressed = Vec::with_capacity(2048);
        compress::lz4::Decoder::new(reader).read_to_end(uncompressed).ok()?;
        Self::read_uncompressed(&uncompressed).ok()
    }

    pub fn write(self, writer: &mut impl Write) -> Option<()> {
        let mut compressed = Vec::with_capacity(2048);
        self.write_uncompressed(&mut compressed).ok()?;
        compress::lz4::Encoder::new(writer).write_all(&compressed).ok()
    }

    pub fn read_uncompressed(reader: &mut impl Read) -> bincode::Result<Self> {
        bincode::deserialize_from(&bytes).map(|s| Self::deserialized(s))
    }

    pub fn write_uncompressed(self, writer: impl Write) -> bincode::Result<()> {
        bincode::serialize_into(writer, self.serialized())
    }

    pub fn deserialized(serialized: SerializedFont) -> Self {
        Font {
            atlas: Atlas::deserialized(serialized.atlas),
            glyphs: serialized.glyphs.into_iter().collect(),
            kerning: serialized.kerning.into_iter().collect(),
            layout: serialized.layout
        }
    }

    pub fn serialized(self) -> SerializedFont {
        SerializedFont {
            atlas: self.atlas.serialized(),
            glyphs: self.glyphs.into_iter().collect(),
            kerning: self.kerning.into_iter().collect(),
            layout: self.layout
        }
    }

}


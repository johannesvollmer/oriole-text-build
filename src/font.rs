use oriole_text::font::{ HashMap, Font, FontLayout, GlyphLayout };
use oriole_text::rectangle::Rectangle;
use crate::atlas::Segment;

pub fn generate_font(
    bytes: &[u8], skip_unknown_chars: bool,
    mut chars: impl Iterator<Item=char>
)
    -> Result<Font, ()>
{
    let rusttype_font = rusttype::Font::from_bytes(bytes).unwrap();

    let mut kerning = HashMap::new();
    let mut glyphs = HashMap::new();

    let proxy_scale_factor = 30.0;
    let proxy_scale = rusttype::Scale::uniform(proxy_scale_factor);
    let proxy_position = rusttype::point(0.0, 0.0);

    let atlas = Atlas::generate(chars.and_then(|character|{
        let glyph = rusttype_font.glyph(character);

        // skip unknown glyphs, except for the zero-glyph itself
        if glyph.id().0 == 0 && character as usize != 0 {
            if skip_unknown_chars {
                return None;
            }
            else {
                panic!("Glyph not supported by font: {}", character);
            }
        }

        let glyph = glyph.glyph.scaled(proxy_scale)
            .positioned(proxy_position);

        if let Some(bounds) = glyph.bounding_box() {
            let width = bounds.width();
            let height = bounds.height();

            let mut binary_image = vec![0_u8; width * height];
            glyph.draw(|x, y, value|{
                binary_image[y * width + x] = if value > 0.5 { 255 } else { 0 };
            });

            let distance_image = signed_distance_field::compute_f32_distance_field(
                signed_distance_field::binary_image::from_byte_slice(&binary_image, width, height)
            );

            if let Some(distance_image) = distance_image.normalize_clamped_distances(-30.0, 30.0) {
                let distance_image = distance_image.to_u8();

                let char_height = 64;
                let scale_factor = height as f32 / char_height as f32;
                let char_width = (width as f32 * scale_factor) as usize;

                let mut resizer = resize::new(
                    width, height, char_width, char_height,
                    resize::Pixel::Gray8, resize::Type::Lanczos3
                );

                let mut downsampled_distances = vec![0_u8; char_width * char_height];
                resizer.resize(&distance_image, &mut downsampled_distances);

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

    Ok(Font {
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


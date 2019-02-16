use oriole_text::atlas::{ Atlas,  SerializedAtlas };
use oriole_text::rectangle::Rectangle;

pub struct Segment {
    size: (usize, usize),
    distance_field: Vec<u8>,
}


pub fn generate_atlas(mut glyphs: impl Iterator<Item=(char, Segment)>) -> Self {
    let mut packed_size = (128, 128);

    struct PackedSegment {
        character: char,
        image_data: Segment,
        packed_position: (usize, usize),
    };

    let mut packed: Vec<PackedSegment> = glyphs
        .map(|(character, segment)| PackedSegment {
            character, image_data: segment,
            packed_position: (0, 0)
        })
        .collect();

    // write into each packed segment the rectangle where it was placed.
    'pack_larger: loop {
        let packer_config = rect_packer::Config {
            width: packed_size.0 as i32,
            height: packed_size.1 as i32,
            border_padding: 3,
            rectangle_padding: 3
        };

        let mut packer = rect_packer::Packer::new(packer_config);

        for packed_segment in &mut packed {
            let (w, h) = packed_segment.image_data.size;
            if let Some(packed) = packer.pack(w as i32, h as i32, false){
                packed_segment.packed_position = (packed.x as usize, packed.y as usize)
            }

            // if it does not fit, try packing into a larger texture
            // TODO are there researched algorithms for this?
            else {
                packed_size.0 += 128;
                packed_size.1 += 128;
                continue 'pack_larger;
            }
        }

        // shrink packed dimensions if there is unused space
        packed_size = (0, 0);
        for packed in &packed {
            packed_size.0 = packed_size.0.max(packed.packed_position.0 + packed.image_data.size.0);
            packed_size.1 = packed_size.1.max(packed.packed_position.1 + packed.image_data.size.1);
        }


        // copy all distance fields into the actual atlas
        let (atlas_w, atlas_h) = packed_size;

        // fill atlas with 255 per default, which is the largest distance possible
        let mut atlas = vec![::std::u8::MAX; atlas_w * atlas_h];

        for packed in packed {
            let (atlas_x, atlas_y) = packed.packed_position;
            let (segment_w, segment_h) = packed.image_data.size;
            let pixels = packed.image_data.distance_field;

            // copy row by row
            for segment_y in 0..segment_h {
                let segment_index = y * segment_w;
                let segment_row = &pixels[segment_index .. segment_index + segment_w];

                let atlas_index = (y + atlas_y) * atlas_w + atlas_x;
                let atlas_row = &mut atlas[atlas_index .. atlas_index + segment_w];

                atlas_row.copy_from_slice(segment_row).unwrap();
            }
        }

        return Atlas {
            glyphs: packed.into_iter()
                .map(|seg| (
                    seg.character,
                    Rectangle {
                        position: (
                            seg.packed_position.0 as f32 / atlas_w as f32,
                            seg.packed_position.1 as f32 / atlas_h as f32,
                        ),
                        dimensions: (
                            seg.image_data.size.0 as f32 / atlas_w as f32,
                            seg.image_data.size.1 as f32 / atlas_h as f32,
                        )
                    }
                ))
                .collect(),

            resolution: packed_size,
            distance_field: atlas
        };
    }
}



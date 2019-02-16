use hashbrown::HashMap;
use serde::{ Serialize, Deserialize };
use serde_derive::*;
use std::io::{ Read, Write };
use crate::rectangle::Rectangle;

pub struct Atlas {
    pub glyphs: HashMap<char, Rectangle>,
    pub resolution: (usize, usize),
    pub distance_field: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedAtlas {
    glyphs: Vec<(char, Rectangle)>,
    distance_field: Vec<u8>,
    resolution: (usize, usize)
}

pub struct Segment {
    size: (usize, usize),
    distance_field: Vec<u8>,
}


impl Atlas {

    pub fn generate(mut glyphs: impl Iterator<Item=(char, Segment)>) -> Self {
        let packed_size = (0, 0);

        struct PackedSegment {
            character: char,
            image_data: Segment,
            packed: Rectangle,
        };

        let mut packed: Vec<_> = glyphs
            .map(|(character, segment)| PackedSegment {
                character, image_data: segment,
                packed: Rectangle::zero()
            })
            .collect();

        'pack_larger: loop {
            for packed_segment in &mut packed {
                if let Some(packed) = rect_packer::pack(){

                }
                else {
                    continue 'pack_larger;
                }
            }

            return Atlas {
                glyphs: packed.into_iter().collect(),
                resolution: packed_size,
                distance_field: render_atlas_image(packed_size, &packed)
            };
        }
    }

    pub fn deserialized(serialized: SerializedAtlas) -> Self {
        Atlas {
            glyphs: serialized.glyphs.into_iter().collect(),
            resolution: serialized.resolution,
            distance_field: serialized.distance_field,
        }
    }

    pub fn serialized(self) -> SerializedAtlas {
        SerializedAtlas {
            glyphs: self.glyphs.into_iter().collect(),
            resolution: self.resolution,
            distance_field: self.distance_field,
        }
    }
}


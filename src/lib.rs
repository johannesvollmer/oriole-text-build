pub mod atlas;
pub mod font;
pub mod text;
pub mod rectangle;

#[cfg(feature = "glium_text")]
pub mod glium;

pub mod prelude {
    pub use font::Font;
    pub use text::TextCaretPositions;
}



#[cfg(text)]
mod tests {
    use crate::text::LayoutGlyphs;
    use crate::font::Font;
    use crate::text::layout_glyphs;

    #[test]
    fn api(){
        let font = Font::generate("fonts/Roboto.ttf", (0..65000)).unwrap();

        let mut text_mesh: Vec<Vertex> = Vec::new();
        let mut caret_positions: Vec<(f32, f32)> = Vec::new();

        struct Vertex {
            position: (f32, f32),
            texture: (f32, f32),
        };

        for glyph in font.layout_glyphs("Hello World!") {
            caret_positions.push(glyph.layout.in_mesh.position);

            let quad_positions = glyph.layout.in_mesh.vertices();
            let quad_texture_coords = glyph.layout.in_atlas.vertices();

            for quad_vertex_index in 0..4 {
                text_mesh.push(Vertex {
                    position: quad_positions[quad_vertex_index],
                    texture: quad_texture_coords[quad_vertex_index]
                })
            }
        }


    }

}
// Copyright © SixtyFPS GmbH <info@slint-ui.com>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-commercial

#[cfg(not(target_arch = "wasm32"))]
pub use tiny_skia::IntRect as Rect;

#[derive(Debug, Clone, Copy, Default)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum PixelFormat {
    // 24 bit RGB
    Rgb,
    // 32 bit RGBA
    Rgba,
    // 8bit alpha map with a given color
    AlphaMap([u8; 3]),
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct Texture {
    pub total_size: Size,
    pub rect: Rect,
    pub data: Vec<u8>,
    pub format: PixelFormat,
}

#[cfg(not(target_arch = "wasm32"))]
impl Texture {
    pub fn new_empty() -> Self {
        Self {
            total_size: Size::default(),
            rect: Rect::from_xywh(0, 0, 1, 1).unwrap(),
            data: vec![0, 0, 0, 0],
            format: PixelFormat::Rgba,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Default)]
pub struct BitmapGlyph {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub x_advance: i16,
    pub data: Vec<u8>, // 8bit alpha map
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct BitmapGlyphs {
    pub pixel_size: u16,
    pub glyph_data: Vec<BitmapGlyph>,
}

#[derive(Debug, Clone)]
pub struct CharacterMapEntry {
    pub code_point: char,
    pub glyph_index: u16,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct BitmapFont {
    pub family_name: String,
    pub character_map: Vec<CharacterMapEntry>, // map of available glyphs, sorted by char
    pub units_per_em: f32,
    pub ascent: f32,
    pub descent: f32,
    pub glyphs: Vec<BitmapGlyphs>,
}

#[derive(Debug, Clone)]
pub enum EmbeddedResourcesKind {
    /// Just put the file content as a resource
    RawData,
    /// The data has been processed in a texture
    TextureData(#[cfg(not(target_arch = "wasm32"))] Texture),
    /// A set of pre-rendered glyphs of a TrueType font
    BitmapFontData(#[cfg(not(target_arch = "wasm32"))] BitmapFont),
}

#[derive(Debug, Clone)]
pub struct EmbeddedResources {
    /// unique integer id, that can be used by the generator for symbol generation.
    pub id: usize,

    pub kind: EmbeddedResourcesKind,
}

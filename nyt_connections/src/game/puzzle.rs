use crate::wasm_bindgen;

use super::color::AsColor;
use super::color::Blue;
use super::color::Color;
use super::color::Green;
use super::color::Purple;
use super::color::Yellow;
use super::Board;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use flate2::write::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::array;
use std::io::Write;
use std::marker::PhantomData;
use std::ops::Deref;
use thiserror::Error;

#[allow(unused_imports)]
use web_sys::console;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionPuzzle {
    yellow: YellowSet,
    blue: BlueSet,
    purple: PurpleSet,
    green: GreenSet,
}

#[repr(transparent)]
#[derive(Serialize, Deserialize, Debug)]
struct BlueSet(ConnectionSet);

#[repr(transparent)]
#[derive(Serialize, Deserialize, Debug)]
struct YellowSet(ConnectionSet);

#[repr(transparent)]
#[derive(Serialize, Deserialize, Debug)]
struct PurpleSet(ConnectionSet);

#[repr(transparent)]
#[derive(Serialize, Deserialize, Debug)]
struct GreenSet(ConnectionSet);

impl Deref for BlueSet {
    type Target = ConnectionSet;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Deref for PurpleSet {
    type Target = ConnectionSet;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Deref for GreenSet {
    type Target = ConnectionSet;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Deref for YellowSet {
    type Target = ConnectionSet;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ConnectionSet {
    pub theme: String,
    pub words: [String; 4],
}

impl ConnectionSet {
    fn new(theme: &str, words: [&str; 4]) -> Self {
        let words: [String; 4] = array::from_fn(|index| String::from(words[index]));
        Self {
            theme: theme.into(),
            words,
        }
    }

    const fn empty_set() -> Self {
        Self {
            theme: String::new(),
            words: [const { String::new() }; 4],
        }
    }

    pub fn theme(&self) -> &str {
        &self.theme
    }

    pub fn words(&self) -> String {
        format!(
            "{}, {}, {}, {}",
            self.words[0], self.words[1], self.words[2], self.words[3]
        )
    }
}

impl std::ops::Index<usize> for ConnectionSet {
    type Output = str;

    fn index(&self, index: usize) -> &str {
        self.words[index].as_str()
    }
}

impl std::ops::Index<PuzzleRef> for ConnectionPuzzle {
    type Output = str;

    fn index(&self, index: PuzzleRef) -> &str {
        let set = self.by_color(index.color());
        index.word(set)
    }
}

const fn empty_pair() -> (String, [String; 4]) {
    (String::new(), [const { String::new() }; 4])
}

impl ConnectionPuzzle {
    pub const fn empty() -> Self {
        let yellow = YellowSet(ConnectionSet::empty_set());
        let blue = BlueSet(ConnectionSet::empty_set());
        let purple = PurpleSet(ConnectionSet::empty_set());
        let green = GreenSet(ConnectionSet::empty_set());

        Self {
            yellow,
            blue,
            purple,
            green,
        }
    }

    pub fn new(
        yellow: (&str, [&str; 4]),
        blue: (&str, [&str; 4]),
        purple: (&str, [&str; 4]),
        green: (&str, [&str; 4]),
    ) -> Self {
        let yellow = YellowSet(ConnectionSet::new(yellow.0, yellow.1));
        let blue = BlueSet(ConnectionSet::new(blue.0, blue.1));
        let purple = PurpleSet(ConnectionSet::new(purple.0, purple.1));
        let green = GreenSet(ConnectionSet::new(green.0, green.1));

        Self {
            yellow,
            blue,
            purple,
            green,
        }
    }

    pub fn encode(&self) -> String {
        let postcard_bytes: Vec<u8> = postcard::to_allocvec(&self).expect("error serializing");
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&postcard_bytes).unwrap();
        let compressed_bytes = encoder.finish().unwrap();
        URL_SAFE.encode(&compressed_bytes)
    }

    pub fn decode(code: &str) -> Result<Self, TranscodingError> {
        let compressed_bytes = URL_SAFE
            .decode(code)
            .map_err(|_| TranscodingError::Base64)?;

        let mut decoder = GzDecoder::new(Vec::new());
        decoder
            .write_all(&compressed_bytes[..])
            .map_err(|_| TranscodingError::Gzip)?;

        let postcard_bytes = decoder.finish().unwrap();
        postcard::from_bytes(&postcard_bytes[..]).map_err(|_| TranscodingError::Postcard)
    }

    pub fn theme(&self, reference: PuzzleRef) -> &str {
        let set = self.by_color(reference.color());
        &set[reference.word_index]
    }

    pub fn by_color(&self, color: Color) -> &ConnectionSet {
        match color {
            Color::Blue => &self.blue,
            Color::Green => &self.green,
            Color::Purple => &self.purple,
            Color::Yellow => &self.yellow,
        }
    }

    pub fn yellow(&self) -> &ConnectionSet {
        &self.yellow
    }

    pub fn blue(&self) -> &ConnectionSet {
        &self.blue
    }
    pub fn green(&self) -> &ConnectionSet {
        &self.green
    }
    pub fn purple(&self) -> &ConnectionSet {
        &self.purple
    }
}

#[wasm_bindgen]
#[derive(Debug, Error)]
pub enum TranscodingError {
    #[error("couldn't decode")]
    Base64,
    #[error("couldn't decompress")]
    Gzip,
    #[error("couldn't deserialize")]
    Postcard,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct PuzzleRef {
    color: Color,
    word_index: usize,
}

impl PuzzleRef {
    const fn new(color: Color, word_index: usize) -> Self {
        Self { color, word_index }
    }

    pub const fn color(&self) -> Color {
        self.color
    }

    pub fn word<'a>(&self, set: &'a ConnectionSet) -> &'a str {
        set.words[self.word_index].as_ref()
    }

    pub const fn new_set() -> [Self; 16] {
        use Color::*;
        [
            Self::new(Yellow, 0),
            Self::new(Yellow, 1),
            Self::new(Yellow, 2),
            Self::new(Yellow, 3),
            Self::new(Blue, 0),
            Self::new(Blue, 1),
            Self::new(Blue, 2),
            Self::new(Blue, 3),
            Self::new(Green, 0),
            Self::new(Green, 1),
            Self::new(Green, 2),
            Self::new(Green, 3),
            Self::new(Purple, 0),
            Self::new(Purple, 1),
            Self::new(Purple, 2),
            Self::new(Purple, 3),
        ]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct PuzzleKey {
    pub color: Color,
    pub word_index: usize,
}

impl PuzzleKey {
    fn new(color: Color, word_index: usize) -> Self {
        Self { color, word_index }
    }
}

impl Default for PuzzleKey {
    fn default() -> Self {
        Self {
            color: Color::Yellow,
            word_index: 0,
        }
    }
}

impl Default for ConnectionPuzzle {
    fn default() -> Self {
        let purple = ("___Room", ["war", "bed", "situation", "clean"]);
        let green = (
            "Domains of Greek Gods",
            ["victory", "ocean", "thunder", "music"],
        );
        let yellow = ("Minecraft Cake Recipe", ["wheat", "milk", "eggs", "sugar"]);
        let blue = ("noble gasses", ["helium", "argon", "krypton", "neon"]);

        Self::new(yellow, blue, purple, green)
    }
}

use brs::Color;
use brs::ColorMode;
use brs::Direction;
use brs::Rotation;
use brs::User;
use brs::WriteData;
use brs::{chrono::DateTime, uuid::Uuid};
use std::ffi::OsStr;
use std::path::Path;
use std::str::FromStr;

type Pos = (i32, i32, i32);
type Col = [u8; 3];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum BrickType {
    // value = asset index
    Basic = 0,
    Tile = 1,
    Micro = 2,
    Stud = 3,
}

impl FromStr for BrickType {
    type Err = std::io::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use std::io::ErrorKind;
        match s.to_lowercase().as_str() {
            "basic" => Ok(Self::Basic),
            "tile" => Ok(Self::Tile),
            "micro" => Ok(Self::Micro),
            "stud" => Ok(Self::Stud),
            _ => Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "unrecognized brick type",
            )),
        }
    }
}

impl BrickType {
    pub fn min_height(self) -> u32 {
        match self {
            Self::Basic => 2,
            Self::Tile => 2,
            Self::Micro => 2,
            Self::Stud => 5,
        }
    }

    pub fn asset_index(self) -> u32 {
        self as u32
    }
}

// convert gamma to linear gamma
pub fn to_linear_gamma(c: u8) -> u8 {
    let cf = (c as f64) / 255.0;
    (if cf > 0.04045 {
        (cf / 1.055 + 0.0521327).powf(2.4) * 255.0
    } else {
        cf / 12.192 * 255.0
    }) as u8
}

// convert sRGB to linear rgb
pub fn to_linear_rgb(rgb: [u8; 4]) -> [u8; 4] {
    [
        to_linear_gamma(rgb[0]),
        to_linear_gamma(rgb[1]),
        to_linear_gamma(rgb[2]),
        rgb[3],
    ]
}

// Brick creation helper
pub fn ez_brick(size: u32, position: Pos, height: u32, color: Col, tile: bool) -> brs::Brick {
    // require brick height to be even (gen doesn't allow odd height bricks)
    let height = height + height % 2;

    brs::Brick {
        asset_name_index: tile.into(),
        size: (size, size, height),
        position: (
            position.0 * size as i32 * 2 + 5,
            position.1 * size as i32 * 2 + 5,
            position.2 - height as i32 + 2,
        ),
        direction: Direction::ZPositive,
        rotation: Rotation::Deg0,
        collision: true,
        visibility: true,
        material_index: 0,
        color: ColorMode::Custom(Color::from_rgba(color[0], color[1], color[2], 255)),
        owner_index: Some(0),
    }
}

// given an array of bricks, create a save
#[allow(unused)]
pub fn bricks_to_save(
    bricks: Vec<brs::Brick>,
    owner_id: String,
    owner_name: String,
) -> brs::WriteData {
    let default_id = Uuid::parse_str("a1b16aca-9627-4a16-a160-67fa9adbb7b6").unwrap();

    let author = User {
        id: Uuid::parse_str(&owner_id).unwrap_or(default_id),
        name: owner_name.clone(),
    };

    let brick_owners = vec![User {
        id: Uuid::parse_str(&owner_id).unwrap_or(default_id),
        name: owner_name,
    }];

    WriteData {
        map: String::from("Plate"),
        author,
        description: String::from("Save generated from heightmap file"),
        save_time: DateTime::from(std::time::SystemTime::now()),
        mods: vec![],
        brick_assets: vec![
            String::from("PB_DefaultBrick"),
            String::from("PB_DefaultTile"),
            String::from("PB_DefaultMicroBrick"),
            String::from("PB_DefaultStudded"),
        ],
        colors: vec![],
        materials: vec![String::from("BMC_Plastic")],
        brick_owners,
        bricks,
    }
}

// get extension from filename
#[allow(unused)]
pub fn file_ext(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

use image::{imageops, RgbaImage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleFactor {
    OneX,
    TwoX,
    ThreeX,
    FourX,
}

impl ScaleFactor {
    pub fn factor(&self) -> u32 {
        match self {
            Self::OneX => 1,
            Self::TwoX => 2,
            Self::ThreeX => 3,
            Self::FourX => 4,
        }
    }

    pub fn from_u32(val: u32) -> Self {
        match val {
            1 => Self::OneX,
            2 => Self::TwoX,
            3 => Self::ThreeX,
            4 => Self::FourX,
            _ => Self::TwoX,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let clean = s.trim().trim_end_matches('x').trim_end_matches('X');
        match clean {
            "1" => Some(Self::OneX),
            "2" => Some(Self::TwoX),
            "3" => Some(Self::ThreeX),
            "4" => Some(Self::FourX),
            _ => None,
        }
    }
}

pub fn apply_scale(img: &RgbaImage, factor: ScaleFactor) -> RgbaImage {
    let f = factor.factor();
    if f == 1 {
        return img.clone();
    }

    let (w, h) = img.dimensions();
    let target_w = w * f;
    let target_h = h * f;

    // Nearest-Neighbor estrito para preservar quinas duras de pixel art
    imageops::resize(img, target_w, target_h, imageops::FilterType::Nearest)
}

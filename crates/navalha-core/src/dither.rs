use crate::palette::{Palette, RgbColor};
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DitherAlgorithm {
    #[default]
    FloydSteinberg,
    Atkinson,
    Bayer4x4,
    Bayer8x8,
    Halftone {
        angle_deg: u16,
        cell_size: u8,
    },
    AsciiArt,
    Threshold,
    AtariGaming,
    Mimeografo,
    SauvolaWoodcut,
    GlitchCrt,
}

impl DitherAlgorithm {
    pub fn parse(s: &str) -> Option<Self> {
        let norm = s.to_lowercase().replace(['-', '_', ' '], "");
        match norm.as_str() {
            "floydsteinberg" | "floyd" | "fs" => Some(Self::FloydSteinberg),
            "atkinson" | "atk" => Some(Self::Atkinson),
            "bayer4x4" | "bayer4" => Some(Self::Bayer4x4),
            "bayer8x8" | "bayer8" | "bayer" => Some(Self::Bayer8x8),
            "halftone" | "halftone45" | "reticula" => Some(Self::Halftone {
                angle_deg: 45,
                cell_size: 6,
            }),
            "halftone0" => Some(Self::Halftone {
                angle_deg: 0,
                cell_size: 6,
            }),
            "halftone90" => Some(Self::Halftone {
                angle_deg: 90,
                cell_size: 6,
            }),
            "asciiart" | "ascii" | "cordeiro" => Some(Self::AsciiArt),
            "threshold" | "limiar" | "brutalismo" => Some(Self::Threshold),
            "atarigaming" | "atari" | "tia" | "stella" => Some(Self::AtariGaming),
            "mimeografo" | "mimeo" | "anilina" => Some(Self::Mimeografo),
            "sauvolawoodcut" | "sauvola" | "xilogravura" | "xilo" => Some(Self::SauvolaWoodcut),
            "glitchcrt" | "glitch" | "crt" => Some(Self::GlitchCrt),
            _ => None,
        }
    }
}

pub fn apply_dither(img: &RgbaImage, algorithm: DitherAlgorithm, palette: &Palette) -> RgbaImage {
    match algorithm {
        DitherAlgorithm::FloydSteinberg => dither_floyd_steinberg(img, palette),
        DitherAlgorithm::Atkinson => dither_atkinson(img, palette),
        DitherAlgorithm::Bayer4x4 => dither_bayer_4x4(img, palette),
        DitherAlgorithm::Bayer8x8 => dither_bayer_8x8(img, palette),
        DitherAlgorithm::Halftone {
            angle_deg,
            cell_size,
        } => dither_halftone(img, palette, angle_deg, cell_size),
        DitherAlgorithm::AsciiArt => crate::ascii::render_ascii_plate(img, palette, 9, 12, false),
        DitherAlgorithm::Threshold => dither_threshold(img, palette),
        DitherAlgorithm::AtariGaming => dither_atari_gaming(img, palette),
        DitherAlgorithm::Mimeografo => dither_mimeografo(img, palette),
        DitherAlgorithm::SauvolaWoodcut => dither_sauvola_woodcut(img, palette),
        DitherAlgorithm::GlitchCrt => dither_glitch_crt(img, palette),
    }
}

/// Floyd-Steinberg com varredura em serpentina alternada
fn dither_floyd_steinberg(img: &RgbaImage, palette: &Palette) -> RgbaImage {
    let (width, height) = img.dimensions();
    let mut out = RgbaImage::new(width, height);

    // Buffer de erro em ponto flutuante [r, g, b]
    let mut error_buf: Vec<[f32; 3]> = img
        .pixels()
        .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
        .collect();

    for y in 0..height {
        let left_to_right = y % 2 == 0;
        let x_indices: Box<dyn Iterator<Item = u32>> = if left_to_right {
            Box::new(0..width)
        } else {
            Box::new((0..width).rev())
        };

        for x in x_indices {
            let idx = (y * width + x) as usize;
            let orig_alpha = img.get_pixel(x, y)[3];

            if orig_alpha == 0 {
                out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                continue;
            }

            let cur = error_buf[idx];
            let r_clamped = cur[0].clamp(0.0, 255.0).round() as u8;
            let g_clamped = cur[1].clamp(0.0, 255.0).round() as u8;
            let b_clamped = cur[2].clamp(0.0, 255.0).round() as u8;

            let current_color = RgbColor::new(r_clamped, g_clamped, b_clamped);
            let target_color = palette.nearest_color(current_color);

            out.put_pixel(
                x,
                y,
                Rgba([target_color.r, target_color.g, target_color.b, orig_alpha]),
            );

            let err_r = cur[0] - target_color.r as f32;
            let err_g = cur[1] - target_color.g as f32;
            let err_b = cur[2] - target_color.b as f32;

            let distribute = |buf: &mut Vec<[f32; 3]>, nx: i32, ny: i32, weight: f32| {
                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    let nidx = (ny as u32 * width + nx as u32) as usize;
                    buf[nidx][0] += err_r * weight;
                    buf[nidx][1] += err_g * weight;
                    buf[nidx][2] += err_b * weight;
                }
            };

            if left_to_right {
                distribute(&mut error_buf, x as i32 + 1, y as i32, 7.0 / 16.0);
                distribute(&mut error_buf, x as i32 - 1, y as i32 + 1, 3.0 / 16.0);
                distribute(&mut error_buf, x as i32, y as i32 + 1, 5.0 / 16.0);
                distribute(&mut error_buf, x as i32 + 1, y as i32 + 1, 1.0 / 16.0);
            } else {
                distribute(&mut error_buf, x as i32 - 1, y as i32, 7.0 / 16.0);
                distribute(&mut error_buf, x as i32 + 1, y as i32 + 1, 3.0 / 16.0);
                distribute(&mut error_buf, x as i32, y as i32 + 1, 5.0 / 16.0);
                distribute(&mut error_buf, x as i32 - 1, y as i32 + 1, 1.0 / 16.0);
            }
        }
    }

    out
}

/// Dither de Atkinson (dissipa apenas 75% do erro para contraste acentuado)
fn dither_atkinson(img: &RgbaImage, palette: &Palette) -> RgbaImage {
    let (width, height) = img.dimensions();
    let mut out = RgbaImage::new(width, height);

    let mut error_buf: Vec<[f32; 3]> = img
        .pixels()
        .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
        .collect();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let orig_alpha = img.get_pixel(x, y)[3];

            if orig_alpha == 0 {
                out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                continue;
            }

            let cur = error_buf[idx];
            let r_clamped = cur[0].clamp(0.0, 255.0).round() as u8;
            let g_clamped = cur[1].clamp(0.0, 255.0).round() as u8;
            let b_clamped = cur[2].clamp(0.0, 255.0).round() as u8;

            let current_color = RgbColor::new(r_clamped, g_clamped, b_clamped);
            let target_color = palette.nearest_color(current_color);

            out.put_pixel(
                x,
                y,
                Rgba([target_color.r, target_color.g, target_color.b, orig_alpha]),
            );

            let err_r = cur[0] - target_color.r as f32;
            let err_g = cur[1] - target_color.g as f32;
            let err_b = cur[2] - target_color.b as f32;

            // Cada vizinho recebe 1/8 do erro. São 6 vizinhos = 6/8 = 75% de dispersão
            let distribute = |buf: &mut Vec<[f32; 3]>, nx: i32, ny: i32| {
                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    let nidx = (ny as u32 * width + nx as u32) as usize;
                    buf[nidx][0] += err_r * (1.0 / 8.0);
                    buf[nidx][1] += err_g * (1.0 / 8.0);
                    buf[nidx][2] += err_b * (1.0 / 8.0);
                }
            };

            let xi = x as i32;
            let yi = y as i32;
            distribute(&mut error_buf, xi + 1, yi);
            distribute(&mut error_buf, xi + 2, yi);
            distribute(&mut error_buf, xi - 1, yi + 1);
            distribute(&mut error_buf, xi, yi + 1);
            distribute(&mut error_buf, xi + 1, yi + 1);
            distribute(&mut error_buf, xi, yi + 2);
        }
    }

    out
}

/// Bayer Ordered Dithering 4x4
fn dither_bayer_4x4(img: &RgbaImage, palette: &Palette) -> RgbaImage {
    #[rustfmt::skip]
    const BAYER_4X4: [f32; 16] = [
         0.0 / 16.0,  8.0 / 16.0,  2.0 / 16.0, 10.0 / 16.0,
        12.0 / 16.0,  4.0 / 16.0, 14.0 / 16.0,  6.0 / 16.0,
         3.0 / 16.0, 11.0 / 16.0,  1.0 / 16.0,  9.0 / 16.0,
        15.0 / 16.0,  7.0 / 16.0, 13.0 / 16.0,  5.0 / 16.0,
    ];

    apply_bayer(img, palette, &BAYER_4X4, 4)
}

/// Bayer Ordered Dithering 8x8
fn dither_bayer_8x8(img: &RgbaImage, palette: &Palette) -> RgbaImage {
    #[rustfmt::skip]
    const BAYER_8X8: [f32; 64] = [
         0.0 / 64.0, 32.0 / 64.0,  8.0 / 64.0, 40.0 / 64.0,  2.0 / 64.0, 34.0 / 64.0, 10.0 / 64.0, 42.0 / 64.0,
        48.0 / 64.0, 16.0 / 64.0, 56.0 / 64.0, 24.0 / 64.0, 50.0 / 64.0, 18.0 / 64.0, 58.0 / 64.0, 26.0 / 64.0,
        12.0 / 64.0, 44.0 / 64.0,  4.0 / 64.0, 36.0 / 64.0, 14.0 / 64.0, 46.0 / 64.0,  6.0 / 64.0, 38.0 / 64.0,
        60.0 / 64.0, 28.0 / 64.0, 52.0 / 64.0, 20.0 / 64.0, 62.0 / 64.0, 30.0 / 64.0, 54.0 / 64.0, 22.0 / 64.0,
         3.0 / 64.0, 35.0 / 64.0, 11.0 / 64.0, 43.0 / 64.0,  1.0 / 64.0, 33.0 / 64.0,  9.0 / 64.0, 41.0 / 64.0,
        51.0 / 64.0, 19.0 / 64.0, 59.0 / 64.0, 27.0 / 64.0, 49.0 / 64.0, 17.0 / 64.0, 57.0 / 64.0, 25.0 / 64.0,
        15.0 / 64.0, 47.0 / 64.0,  7.0 / 64.0, 39.0 / 64.0, 13.0 / 64.0, 45.0 / 64.0,  5.0 / 64.0, 37.0 / 64.0,
        63.0 / 64.0, 31.0 / 64.0, 55.0 / 64.0, 23.0 / 64.0, 61.0 / 64.0, 29.0 / 64.0, 53.0 / 64.0, 21.0 / 64.0,
    ];

    apply_bayer(img, palette, &BAYER_8X8, 8)
}

fn apply_bayer(img: &RgbaImage, palette: &Palette, matrix: &[f32], dim: u32) -> RgbaImage {
    let (width, height) = img.dimensions();
    let mut out = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let p = img.get_pixel(x, y);
            if p[3] == 0 {
                out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                continue;
            }

            let matrix_val = matrix[((y % dim) * dim + (x % dim)) as usize];
            let offset = (matrix_val - 0.5) * 64.0;

            let r = (p[0] as f32 + offset).clamp(0.0, 255.0).round() as u8;
            let g = (p[1] as f32 + offset).clamp(0.0, 255.0).round() as u8;
            let b = (p[2] as f32 + offset).clamp(0.0, 255.0).round() as u8;

            let target = palette.nearest_color(RgbColor::new(r, g, b));
            out.put_pixel(x, y, Rgba([target.r, target.g, target.b, p[3]]));
        }
    }

    out
}

/// Halftone analógico em ângulos configuráveis (0, 45, 90 graus)
fn dither_halftone(img: &RgbaImage, palette: &Palette, angle_deg: u16, cell_size: u8) -> RgbaImage {
    let (width, height) = img.dimensions();
    let mut out = RgbaImage::new(width, height);

    let rad = (angle_deg as f32).to_radians();
    let cos_a = rad.cos();
    let sin_a = rad.sin();
    let size = cell_size.max(2) as f32;

    for y in 0..height {
        for x in 0..width {
            let p = img.get_pixel(x, y);
            if p[3] == 0 {
                out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                continue;
            }

            let xf = x as f32;
            let yf = y as f32;

            // Rotação de coordenadas pelo ângulo de retícula
            let rx = xf * cos_a - yf * sin_a;
            let ry = xf * sin_a + yf * cos_a;

            let mod_x = ((rx % size) + size) % size;
            let mod_y = ((ry % size) + size) % size;

            let center = size / 2.0;
            let dist_center = ((mod_x - center).powi(2) + (mod_y - center).powi(2)).sqrt();
            let max_dist = size * std::f32::consts::FRAC_1_SQRT_2; // diagonal/2

            // Normaliza distância ao centro da célula de meio-tom (0.0 no centro, 1.0 na borda)
            let cell_factor = dist_center / max_dist;

            let lum = 0.299 * (p[0] as f32 / 255.0)
                + 0.587 * (p[1] as f32 / 255.0)
                + 0.114 * (p[2] as f32 / 255.0);

            // Se o fator da célula for menor que a escuridão (1 - lum), vira escuro
            let darkness = 1.0 - lum;
            let is_dot = cell_factor <= darkness;

            let chosen_color = if is_dot {
                // Cor mais escura da paleta
                palette
                    .colors
                    .iter()
                    .min_by(|a, b| a.luminance().partial_cmp(&b.luminance()).unwrap())
                    .copied()
                    .unwrap_or(RgbColor::new(0, 0, 0))
            } else {
                // Cor mais clara da paleta
                palette
                    .colors
                    .iter()
                    .max_by(|a, b| a.luminance().partial_cmp(&b.luminance()).unwrap())
                    .copied()
                    .unwrap_or(RgbColor::new(255, 255, 255))
            };

            out.put_pixel(
                x,
                y,
                Rgba([chosen_color.r, chosen_color.g, chosen_color.b, p[3]]),
            );
        }
    }

    out
}

/// Binarização global estrita por média ponderada de luminância (Brutalismo)
fn dither_threshold(img: &RgbaImage, palette: &Palette) -> RgbaImage {
    let (width, height) = img.dimensions();
    let mut out = RgbaImage::new(width, height);

    let mut sum_lum = 0.0f64;
    let mut count = 0usize;

    for p in img.pixels() {
        if p[3] > 0 {
            let lum = 0.299 * p[0] as f64 + 0.587 * p[1] as f64 + 0.114 * p[2] as f64;
            sum_lum += lum;
            count += 1;
        }
    }

    let avg_lum = if count > 0 {
        (sum_lum / count as f64) as f32
    } else {
        128.0
    };

    let dark_color = palette
        .colors
        .iter()
        .min_by(|a, b| a.luminance().partial_cmp(&b.luminance()).unwrap())
        .copied()
        .unwrap_or(RgbColor::new(0, 0, 0));

    let light_color = palette
        .colors
        .iter()
        .max_by(|a, b| a.luminance().partial_cmp(&b.luminance()).unwrap())
        .copied()
        .unwrap_or(RgbColor::new(255, 255, 255));

    for y in 0..height {
        for x in 0..width {
            let p = img.get_pixel(x, y);
            if p[3] == 0 {
                out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                continue;
            }

            let lum = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
            let chosen = if lum >= avg_lum {
                light_color
            } else {
                dark_color
            };
            out.put_pixel(x, y, Rgba([chosen.r, chosen.g, chosen.b, p[3]]));
        }
    }

    out
}

/// Downsampling assimétrico 2:1 (160x192), quantização TIA Stella e scanlines CRT
fn dither_atari_gaming(img: &RgbaImage, palette: &Palette) -> RgbaImage {
    let (width, height) = img.dimensions();
    let mut out = RgbaImage::new(width, height);

    for y in 0..height {
        let is_scanline_blank = y % 2 == 0;
        let sample_y = (y * 192 / height.max(1)) * height / 192;

        for x in 0..width {
            let p = img.get_pixel(x, y);
            if p[3] == 0 {
                out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                continue;
            }

            if is_scanline_blank {
                out.put_pixel(x, y, Rgba([0, 0, 0, p[3]]));
            } else {
                let sample_x = (x * 160 / width.max(1)) * width / 160;
                let sampled_p = img.get_pixel(sample_x.min(width - 1), sample_y.min(height - 1));
                let target =
                    palette.nearest_color(RgbColor::new(sampled_p[0], sampled_p[1], sampled_p[2]));
                out.put_pixel(x, y, Rgba([target.r, target.g, target.b, p[3]]));
            }
        }
    }

    out
}

/// Mimeógrafo violeta com densidade de tinta invertida, Bayer 4x4 e sangramento de matriz 1px
fn dither_mimeografo(img: &RgbaImage, palette: &Palette) -> RgbaImage {
    let (width, height) = img.dimensions();
    let mut out = RgbaImage::new(width, height);

    let mut density = vec![0.0f32; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            let p = img.get_pixel(x, y);
            if p[3] > 0 {
                let lum = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0;
                density[(y * width + x) as usize] = (1.0 - lum).clamp(0.0, 1.0);
            }
        }
    }

    // Sangramento morfológico de 1px em áreas com densidade > 0.70
    let mut bleed = vec![false; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if density[idx] > 0.70 {
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            bleed[(ny as u32 * width + nx as u32) as usize] = true;
                        }
                    }
                }
            }
        }
    }

    let bayer = [
        0.0 / 16.0,
        8.0 / 16.0,
        2.0 / 16.0,
        10.0 / 16.0,
        12.0 / 16.0,
        4.0 / 16.0,
        14.0 / 16.0,
        6.0 / 16.0,
        3.0 / 16.0,
        11.0 / 16.0,
        1.0 / 16.0,
        9.0 / 16.0,
        15.0 / 16.0,
        7.0 / 16.0,
        13.0 / 16.0,
        5.0 / 16.0,
    ];

    let papel = palette
        .colors
        .first()
        .copied()
        .unwrap_or(RgbColor::new(244, 239, 230));
    let violeta = palette
        .colors
        .get(1)
        .copied()
        .unwrap_or(RgbColor::new(67, 44, 122));
    let saturacao = palette
        .colors
        .get(2)
        .copied()
        .unwrap_or(RgbColor::new(42, 25, 84));

    for y in 0..height {
        for x in 0..width {
            let p = img.get_pixel(x, y);
            if p[3] == 0 {
                out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                continue;
            }

            let idx = (y * width + x) as usize;
            let d = density[idx];
            let is_bleed = bleed[idx];

            let b_val = bayer[((y % 4) * 4 + (x % 4)) as usize];
            let threshold = b_val * 0.85 + 0.05;

            let color = if d > threshold || (is_bleed && d > 0.35) {
                if d > 0.65 || is_bleed {
                    saturacao
                } else {
                    violeta
                }
            } else {
                papel
            };

            out.put_pixel(x, y, Rgba([color.r, color.g, color.b, p[3]]));
        }
    }

    out
}

/// Segmentação adaptativa local de Sauvola (15x15, k=0.35, R=128) + estrias de corte em madeira
fn dither_sauvola_woodcut(img: &RgbaImage, palette: &Palette) -> RgbaImage {
    let (width, height) = img.dimensions();
    let mut out = RgbaImage::new(width, height);

    let k = 0.35f32;
    let r_dyn = 128.0f32;
    let window_radius = 7i32; // 15x15 = raio 7

    let paper_color = palette
        .colors
        .first()
        .copied()
        .unwrap_or(RgbColor::new(214, 196, 165));
    let ink_color = palette
        .colors
        .get(1)
        .copied()
        .unwrap_or(RgbColor::new(19, 17, 16));

    let mut lum_map = vec![0.0f32; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            let p = img.get_pixel(x, y);
            lum_map[(y * width + x) as usize] =
                0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
        }
    }

    for y in 0..height {
        for x in 0..width {
            let p = img.get_pixel(x, y);
            if p[3] == 0 {
                out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                continue;
            }

            let cur_lum = lum_map[(y * width + x) as usize];

            // Janela 15x15 local
            let min_x = (x as i32 - window_radius).max(0) as u32;
            let max_x = (x as i32 + window_radius).min(width as i32 - 1) as u32;
            let min_y = (y as i32 - window_radius).max(0) as u32;
            let max_y = (y as i32 + window_radius).min(height as i32 - 1) as u32;

            let mut sum = 0.0f32;
            let mut sum_sq = 0.0f32;
            let mut count = 0.0f32;

            for wy in min_y..=max_y {
                for wx in min_x..=max_x {
                    let val = lum_map[(wy * width + wx) as usize];
                    sum += val;
                    sum_sq += val * val;
                    count += 1.0;
                }
            }

            let mean = sum / count;
            let variance = (sum_sq / count - mean * mean).max(0.0);
            let std_dev = variance.sqrt();

            let t = mean * (1.0 + k * ((std_dev / r_dyn) - 1.0));
            let mut is_dark = cur_lum < t;

            // Interferência direcional de fibra de corte em madeira (descontinuidade vegetal)
            let wood_grain = ((x * 19 + (x / 5) * 11 + y * 2) % 31 == 0) && ((y + x * 3) % 7 != 0);
            if wood_grain && (cur_lum > t * 0.7 && cur_lum < t * 1.3) {
                is_dark = !is_dark;
            }

            let color = if is_dark { ink_color } else { paper_color };
            out.put_pixel(x, y, Rgba([color.r, color.g, color.b, p[3]]));
        }
    }

    out
}

/// Sync jitter senoidal + aberração cromática R/B (+-4px) + barra horizontal de V-Hold 12px
fn dither_glitch_crt(img: &RgbaImage, palette: &Palette) -> RgbaImage {
    let (width, height) = img.dimensions();
    let mut out = RgbaImage::new(width, height);

    let v_hold_start = (height * 3) / 4;
    let v_hold_end = (v_hold_start + 12).min(height);

    for y in 0..height {
        // Deslocamento de varredura senoidal
        let mut jitter = (6.0 * (0.08 * y as f32).sin()).floor() as i32;
        if y % 48 < 3 {
            let abrupt = if (y / 48) % 2 == 0 { 18 } else { -18 };
            jitter += abrupt;
        }

        let in_v_hold = y >= v_hold_start && y < v_hold_end;

        for x in 0..width {
            let p_orig = img.get_pixel(x, y);
            if p_orig[3] == 0 {
                out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                continue;
            }

            let base_x = x as i32 + jitter;
            let r_x = (base_x - 4).clamp(0, width as i32 - 1) as u32;
            let g_x = base_x.clamp(0, width as i32 - 1) as u32;
            let b_x = (base_x + 4).clamp(0, width as i32 - 1) as u32;

            let mut r = img.get_pixel(r_x, y)[0];
            let mut g = img.get_pixel(g_x, y)[1];
            let mut b = img.get_pixel(b_x, y)[2];

            // Inversão e ruído na barra de V-Hold
            if in_v_hold {
                r = 255 - r;
                g = 255 - g;
                b = 255 - b;
                if ((x * 13 + y * 29) % 5) == 0 {
                    r ^= 0x7F;
                    b ^= 0x7F;
                }
            }

            let mapped = palette.nearest_color(RgbColor::new(r, g, b));
            out.put_pixel(x, y, Rgba([mapped.r, mapped.g, mapped.b, p_orig[3]]));
        }
    }

    out
}

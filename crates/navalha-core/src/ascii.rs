//! Motor determinístico de ASCII Art e quantização por densidade tipográfica
//! Homenagem a Waldemar Cordeiro & Giorgio Moscati (USP, 1971, "Derivadas de uma Imagem")

#![allow(clippy::type_complexity)]

use crate::palette::RgbColor;
use crate::press::EMBEDDED_FONT_BYTES;
use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

/// Rampa de densidade óptica de caracteres (do mais escuro/vazio ao mais denso/brilhante)
pub const ASCII_DENSITY_RAMP: &[u8] = b" .:-=+*#%@";

/// Rampa estendida de 16 níveis para maior fidelidade de meios-tons
pub const ASCII_EXTENDED_RAMP: &[u8] = b" .'`,:;!~+=*#%@#";

/// Converte uma imagem qualquer para uma string de arte ASCII pura
pub fn image_to_ascii(img: &DynamicImage, cols: u32, invert: bool) -> String {
    let (orig_w, orig_h) = img.dimensions();
    if orig_w == 0 || orig_h == 0 {
        return String::new();
    }

    // Caracteres monoespaçados costumam ser ~2x mais altos que largos, compensamos com 0.55
    let aspect = orig_h as f32 / orig_w as f32;
    let rows = ((cols as f32 * aspect) * 0.55).max(1.0) as u32;

    let resized = img.resize_exact(cols, rows, image::imageops::FilterType::Nearest);
    let mut out = String::with_capacity((cols as usize + 1) * rows as usize);

    let ramp = ASCII_DENSITY_RAMP;
    let ramp_len = ramp.len();

    for y in 0..rows {
        for x in 0..cols {
            let p = resized.get_pixel(x, y);
            // ITU-R BT.601 luminance
            let lum = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0;

            let idx = if invert {
                ((1.0 - lum) * (ramp_len - 1) as f32).round() as usize
            } else {
                (lum * (ramp_len - 1) as f32).round() as usize
            };
            let idx = idx.clamp(0, ramp_len - 1);
            out.push(ramp[idx] as char);
        }
        out.push('\n');
    }

    out
}

/// Rasteriza uma string ASCII em uma imagem RgbaImage com fundo e fósforo monoespaçado
pub fn render_ascii_to_image(
    ascii_text: &str,
    cell_width: u32,
    cell_height: u32,
    fg: RgbColor,
    bg: RgbColor,
) -> RgbaImage {
    let lines: Vec<&str> = ascii_text.lines().collect();
    if lines.is_empty() {
        return RgbaImage::new(1, 1);
    }

    let max_cols = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1) as u32;
    let rows = lines.len() as u32;

    let img_w = (max_cols * cell_width).max(1);
    let img_h = (rows * cell_height).max(1);

    let mut img = RgbaImage::from_pixel(img_w, img_h, Rgba([bg.r, bg.g, bg.b, 255]));
    let font = FontRef::try_from_slice(EMBEDDED_FONT_BYTES).expect("Fonte embutida corrompida");
    let scale = PxScale::from(cell_height as f32 * 0.95);
    let scaled_font = font.as_scaled(scale);

    for (row_idx, line) in lines.iter().enumerate() {
        let base_y = row_idx as u32 * cell_height;
        for (col_idx, ch) in line.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let base_x = col_idx as u32 * cell_width;
            let glyph_id = font.glyph_id(ch);
            let glyph = glyph_id.with_scale_and_position(scale, ab_glyph::Point { x: 0.0, y: 0.0 });

            if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|gx, gy, c| {
                    if c > 0.2 {
                        let px = base_x as i32 + bounds.min.x as i32 + gx as i32;
                        let py = base_y as i32
                            + bounds.min.y as i32
                            + gy as i32
                            + scaled_font.ascent() as i32;
                        if px >= 0 && px < img_w as i32 && py >= 0 && py < img_h as i32 {
                            img.put_pixel(px as u32, py as u32, Rgba([fg.r, fg.g, fg.b, 255]));
                        }
                    }
                });
            }
        }
    }

    img
}

/// Rasteriza uma imagem de entrada como matriz de caracteres ASCII reais (Waldemar Cordeiro 1971)
pub fn render_ascii_plate(
    img: &RgbaImage,
    palette: &crate::palette::Palette,
    cell_width: u32,
    cell_height: u32,
    invert: bool,
) -> RgbaImage {
    let (src_w, src_h) = img.dimensions();
    if src_w == 0 || src_h == 0 {
        return RgbaImage::new(1, 1);
    }

    let out_w = src_w * cell_width;
    let out_h = src_h * cell_height;
    let mut out = RgbaImage::new(out_w, out_h);

    let font = FontRef::try_from_slice(EMBEDDED_FONT_BYTES).expect("Fonte embutida corrompida");
    let scale = PxScale::from(cell_height as f32 * 0.95);
    let scaled_font = font.as_scaled(scale);

    let ramp = ASCII_DENSITY_RAMP;
    let ramp_len = ramp.len();

    // Cache pré-rasterizado dos glifos da rampa (máscaras de alfa 0..255)
    let mut glyph_masks: Vec<Option<(ab_glyph::Rect, Vec<(i32, i32, u8)>)>> =
        Vec::with_capacity(ramp_len);
    for &byte in ramp {
        let ch = byte as char;
        if ch == ' ' {
            glyph_masks.push(None);
            continue;
        }
        let glyph_id = font.glyph_id(ch);
        let glyph = glyph_id.with_scale_and_position(scale, ab_glyph::Point { x: 0.0, y: 0.0 });
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            let mut pts = Vec::new();
            outlined.draw(|gx, gy, c| {
                if c > 0.15 {
                    let a = (c * 255.0).clamp(0.0, 255.0) as u8;
                    pts.push((gx as i32, gy as i32, a));
                }
            });
            glyph_masks.push(Some((bounds, pts)));
        } else {
            glyph_masks.push(None);
        }
    }

    // Cores úteis da paleta para gradiente de fósforo
    let pal_colors = &palette.colors;
    let fallback_fg = RgbColor::new(0, 255, 102);

    for sy in 0..src_h {
        let base_y = sy * cell_height;
        for sx in 0..src_w {
            let p = img.get_pixel(sx, sy);
            if p[3] < 128 {
                continue;
            }

            let base_x = sx * cell_width;
            let raw_lum = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0;
            if raw_lum < 0.14 {
                // Fundo preto CRT limpo (sem ruído de pontos)
                continue;
            }

            // Curva de resposta tonal para enriquecer meios-tons faciais
            let lum = raw_lum.powf(0.75);
            let idx = if invert {
                ((1.0 - lum) * (ramp_len - 1) as f32).round() as usize
            } else {
                (lum * (ramp_len - 1) as f32).round() as usize
            };
            let idx = idx.clamp(1, ramp_len - 1);

            if let Some((bounds, ref pts)) = glyph_masks[idx] {
                // Centraliza horizontalmente o glifo na célula
                let glyph_w = bounds.width();
                let pad_x = ((cell_width as f32 - glyph_w) / 2.0).max(0.0) as i32;

                // Mapeia cor de acordo com o nível da paleta
                let color = if pal_colors.len() >= 4 {
                    if idx < 3 {
                        pal_colors[1]
                    } else if idx < 7 {
                        pal_colors[2]
                    } else {
                        pal_colors[3]
                    }
                } else if !pal_colors.is_empty() {
                    pal_colors[pal_colors.len() - 1]
                } else {
                    fallback_fg
                };

                for &(gx, gy, a) in pts {
                    let px = base_x as i32 + pad_x + bounds.min.x as i32 + gx;
                    let py = base_y as i32 + bounds.min.y as i32 + gy + scaled_font.ascent() as i32;
                    if px >= 0 && px < out_w as i32 && py >= 0 && py < out_h as i32 {
                        let cur = out.get_pixel_mut(px as u32, py as u32);
                        if a == 255 || cur[3] == 0 {
                            *cur = Rgba([color.r, color.g, color.b, a]);
                        } else {
                            let old_a = cur[3] as f32 / 255.0;
                            let new_a = a as f32 / 255.0;
                            let out_a = new_a + old_a * (1.0 - new_a);
                            if out_a > 0.0 {
                                let r = ((color.r as f32 * new_a
                                    + cur[0] as f32 * old_a * (1.0 - new_a))
                                    / out_a) as u8;
                                let g = ((color.g as f32 * new_a
                                    + cur[1] as f32 * old_a * (1.0 - new_a))
                                    / out_a) as u8;
                                let b = ((color.b as f32 * new_a
                                    + cur[2] as f32 * old_a * (1.0 - new_a))
                                    / out_a) as u8;
                                *cur = Rgba([r, g, b, (out_a * 255.0) as u8]);
                            }
                        }
                    }
                }
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_conversion() {
        let img = DynamicImage::ImageRgba8(RgbaImage::new(40, 40));
        let ascii = image_to_ascii(&img, 20, false);
        assert!(!ascii.is_empty());
    }

    #[test]
    fn test_render_ascii_plate() {
        let mut test_img = RgbaImage::new(40, 40);
        for y in 0..40 {
            for x in 0..40 {
                let v = ((x + y) * 255 / 80) as u8;
                test_img.put_pixel(x, y, Rgba([v, v, v, 255]));
            }
        }
        let pal = crate::palette::Palette::terminal_green();
        let plate = render_ascii_plate(&test_img, &pal, 8, 12, false);
        assert_eq!(plate.width(), 40 * 8);
        assert_eq!(plate.height(), 40 * 12);
    }

    #[test]
    fn test_ascii_plate_rendering_density() {
        let mut test_img = RgbaImage::new(20, 20);
        // Preenche com gradiente para testar todas as faixas da rampa de caracteres
        for y in 0..20 {
            for x in 0..20 {
                let v = ((x + y) * 255 / 40) as u8;
                test_img.put_pixel(x, y, Rgba([v, v, v, 255]));
            }
        }
        let pal = crate::palette::Palette::terminal_green();
        let plate = render_ascii_plate(&test_img, &pal, 9, 12, false);
        assert_eq!(plate.width(), 20 * 9);
        assert_eq!(plate.height(), 20 * 12);

        // Deve conter pixels ativos desenhados com alfa opaco e cor da paleta
        let active_pixels = plate.pixels().filter(|p| p[3] > 0).count();
        assert!(
            active_pixels > 0,
            "Deveria ter caracteres renderizados na matriz"
        );
    }
}

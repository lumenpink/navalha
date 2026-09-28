use image::{imageops, Rgba, RgbaImage};

#[derive(Debug, Clone, PartialEq)]
pub struct PrepConfig {
    /// Rotação da imagem em graus (0, 90, 180, 270)
    pub rotation: u32,
    /// Fator de zoom granular na imagem original (0.1 a 10.0, padrão 1.0 = 100%)
    pub zoom: f32,
    /// Largura base forçada para a retícula (ex: 600 a 900)
    pub target_width: Option<u32>,
    /// Intensidade do Unsharp Masking agressivo (0.0 a 5.0, padrão 2.0)
    pub unsharp_strength: f32,
    /// Raio do blur para a máscara de nitidez (1 a 5, padrão 2)
    pub unsharp_radius: i32,
    /// Piso de queima de sombras (0.0 a 1.0, ex: 0.15 = tudo abaixo vira 0)
    pub levels_shadow_cutoff: f32,
    /// Teto de queima de altas luzes (0.0 a 1.0, ex: 0.85 = tudo acima vira 1)
    pub levels_highlight_cutoff: f32,
    /// Gamma / meio-tom (padrão 1.0)
    pub levels_gamma: f32,
}

impl Default for PrepConfig {
    fn default() -> Self {
        Self {
            rotation: 0,
            zoom: 1.0,
            target_width: Some(720),
            unsharp_strength: 2.2,
            unsharp_radius: 2,
            levels_shadow_cutoff: 0.18,
            levels_highlight_cutoff: 0.82,
            levels_gamma: 1.0,
        }
    }
}

/// Aplica rotação e zoom granular mantendo o enquadramento na chapa
pub fn apply_image_orientation_and_zoom(img: &RgbaImage, rotation: u32, zoom: f32) -> RgbaImage {
    let rotated = match rotation % 360 {
        90 => imageops::rotate90(img),
        180 => imageops::rotate180(img),
        270 => imageops::rotate270(img),
        _ => img.clone(),
    };

    apply_image_zoom(&rotated, zoom)
}

/// Aplica zoom granular sobre a imagem original:
/// - zoom == 1.0: retorna clone sem modificação
/// - zoom > 1.0: aproxima (teleobjetiva), recortando uma janela central e redimensionando para (w, h)
/// - zoom < 1.0: afasta (grande-angular), reduzindo a imagem e centralizando em um canvas transparente (w, h)
pub fn apply_image_zoom(img: &RgbaImage, zoom: f32) -> RgbaImage {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return img.clone();
    }

    let z = zoom.clamp(0.1, 10.0);
    if (z - 1.0).abs() < 0.001 {
        return img.clone();
    }

    if z > 1.0 {
        // Zoom in: recorta janela central de proporção w/z x h/z e amplia de volta para (w, h)
        let crop_w = ((w as f32 / z).round() as u32).max(1).min(w);
        let crop_h = ((h as f32 / z).round() as u32).max(1).min(h);
        let crop_x = (w - crop_w) / 2;
        let crop_y = (h - crop_h) / 2;

        let cropped = imageops::crop_imm(img, crop_x, crop_y, crop_w, crop_h).to_image();
        imageops::resize(&cropped, w, h, imageops::FilterType::Lanczos3)
    } else {
        // Zoom out: encolhe a imagem inteira e centraliza em um canvas transparente (w, h)
        let new_w = ((w as f32 * z).round() as u32).max(1);
        let new_h = ((h as f32 * z).round() as u32).max(1);
        let scaled = imageops::resize(img, new_w, new_h, imageops::FilterType::Lanczos3);

        let mut out = RgbaImage::new(w, h);
        let offset_x = ((w as i32 - new_w as i32) / 2).max(0) as u32;
        let offset_y = ((h as i32 - new_h as i32) / 2).max(0) as u32;

        imageops::overlay(&mut out, &scaled, offset_x as i64, offset_y as i64);
        out
    }
}

pub fn prepare_plate(img: &RgbaImage, config: &PrepConfig) -> RgbaImage {
    // 0. Rotação se configurada
    let rotated = match config.rotation % 360 {
        90 => imageops::rotate90(img),
        180 => imageops::rotate180(img),
        270 => imageops::rotate270(img),
        _ => img.clone(),
    };

    // 1. Downsampling forçado
    let resized = if let Some(target_w) = config.target_width {
        let (w, h) = rotated.dimensions();
        if w > target_w {
            let target_h = ((h as f32 * target_w as f32) / w as f32).round().max(1.0) as u32;
            imageops::resize(&rotated, target_w, target_h, imageops::FilterType::Lanczos3)
        } else {
            rotated
        }
    } else {
        rotated
    };

    // 2. Unsharp Masking Agressivo
    let sharpened = if config.unsharp_strength > 0.0 {
        apply_unsharp_mask(&resized, config.unsharp_radius, config.unsharp_strength)
    } else {
        resized
    };

    // 3. Queima de Níveis (Levels)
    apply_levels(
        &sharpened,
        config.levels_shadow_cutoff,
        config.levels_highlight_cutoff,
        config.levels_gamma,
    )
}

/// Calcula o threshold ótimo de separação de tons pelo algoritmo de Otsu
pub fn compute_otsu_threshold(img: &RgbaImage) -> f32 {
    let mut histogram = [0u32; 256];
    let mut total_pixels = 0u32;

    for p in img.pixels() {
        if p[3] > 10 {
            let lum = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32).round() as usize;
            let lum = lum.clamp(0, 255);
            histogram[lum] += 1;
            total_pixels += 1;
        }
    }

    if total_pixels == 0 {
        return 0.5;
    }

    let mut sum_all = 0.0f32;
    for (i, &count) in histogram.iter().enumerate() {
        sum_all += i as f32 * count as f32;
    }

    let mut sum_bg = 0.0f32;
    let mut weight_bg = 0u32;
    let mut max_variance = 0.0f32;
    let mut best_threshold = 128usize;

    for (t, &count) in histogram.iter().enumerate() {
        weight_bg += count;
        if weight_bg == 0 {
            continue;
        }
        let weight_fg = total_pixels - weight_bg;
        if weight_fg == 0 {
            break;
        }

        sum_bg += t as f32 * count as f32;
        let mean_bg = sum_bg / weight_bg as f32;
        let mean_fg = (sum_all - sum_bg) / weight_fg as f32;

        let diff = mean_bg - mean_fg;
        let variance = weight_bg as f32 * weight_fg as f32 * diff * diff;

        if variance > max_variance {
            max_variance = variance;
            best_threshold = t;
        }
    }

    (best_threshold as f32 / 255.0).clamp(0.1, 0.95)
}

fn apply_unsharp_mask(img: &RgbaImage, radius: i32, strength: f32) -> RgbaImage {
    let (width, height) = img.dimensions();
    let mut out = RgbaImage::new(width, height);

    // Box blur rápido de 2 passadas para aproximar gaussian blur
    let blurred = box_blur(img, radius);

    for y in 0..height {
        for x in 0..width {
            let orig = img.get_pixel(x, y);
            let blur = blurred.get_pixel(x, y);

            if orig[3] == 0 {
                out.put_pixel(x, y, *orig);
                continue;
            }

            let mut channels = [0u8; 4];
            for c in 0..3 {
                let orig_val = orig[c] as f32;
                let blur_val = blur[c] as f32;
                let diff = orig_val - blur_val;
                let sharp_val = orig_val + (diff * strength);
                channels[c] = sharp_val.clamp(0.0, 255.0).round() as u8;
            }
            channels[3] = orig[3];

            out.put_pixel(x, y, Rgba(channels));
        }
    }

    out
}

fn box_blur(img: &RgbaImage, radius: i32) -> RgbaImage {
    let (width, height) = img.dimensions();
    if radius <= 0 || width == 0 || height == 0 {
        return img.clone();
    }

    let mut temp = RgbaImage::new(width, height);
    let mut out = RgbaImage::new(width, height);

    // Passada horizontal
    for y in 0..height {
        for x in 0..width {
            let mut sum_r = 0f32;
            let mut sum_g = 0f32;
            let mut sum_b = 0f32;
            let mut count = 0f32;

            for dx in -radius..=radius {
                let nx = x as i32 + dx;
                if nx >= 0 && nx < width as i32 {
                    let p = img.get_pixel(nx as u32, y);
                    if p[3] > 0 {
                        sum_r += p[0] as f32;
                        sum_g += p[1] as f32;
                        sum_b += p[2] as f32;
                        count += 1.0;
                    }
                }
            }

            if count > 0.0 {
                let orig_a = img.get_pixel(x, y)[3];
                temp.put_pixel(
                    x,
                    y,
                    Rgba([
                        (sum_r / count).round() as u8,
                        (sum_g / count).round() as u8,
                        (sum_b / count).round() as u8,
                        orig_a,
                    ]),
                );
            } else {
                temp.put_pixel(x, y, *img.get_pixel(x, y));
            }
        }
    }

    // Passada vertical
    for y in 0..height {
        for x in 0..width {
            let mut sum_r = 0f32;
            let mut sum_g = 0f32;
            let mut sum_b = 0f32;
            let mut count = 0f32;

            for dy in -radius..=radius {
                let ny = y as i32 + dy;
                if ny >= 0 && ny < height as i32 {
                    let p = temp.get_pixel(x, ny as u32);
                    if p[3] > 0 {
                        sum_r += p[0] as f32;
                        sum_g += p[1] as f32;
                        sum_b += p[2] as f32;
                        count += 1.0;
                    }
                }
            }

            if count > 0.0 {
                let orig_a = temp.get_pixel(x, y)[3];
                out.put_pixel(
                    x,
                    y,
                    Rgba([
                        (sum_r / count).round() as u8,
                        (sum_g / count).round() as u8,
                        (sum_b / count).round() as u8,
                        orig_a,
                    ]),
                );
            } else {
                out.put_pixel(x, y, *temp.get_pixel(x, y));
            }
        }
    }

    out
}

fn apply_levels(img: &RgbaImage, shadow_cut: f32, highlight_cut: f32, gamma: f32) -> RgbaImage {
    let (width, height) = img.dimensions();
    let mut out = RgbaImage::new(width, height);

    let low = shadow_cut.clamp(0.0, 0.99);
    let high = highlight_cut.clamp(low + 0.01, 1.0);
    let range = high - low;
    let inv_gamma = 1.0 / gamma.max(0.01);

    for y in 0..height {
        for x in 0..width {
            let p = img.get_pixel(x, y);
            if p[3] == 0 {
                out.put_pixel(x, y, *p);
                continue;
            }

            let mut channels = [0u8; 4];
            for c in 0..3 {
                let norm = (p[c] as f32) / 255.0;
                let mapped = if norm <= low {
                    0.0
                } else if norm >= high {
                    1.0
                } else {
                    let normalized = (norm - low) / range;
                    normalized.powf(inv_gamma)
                };
                channels[c] = (mapped * 255.0).clamp(0.0, 255.0).round() as u8;
            }
            channels[3] = p[3];

            out.put_pixel(x, y, Rgba(channels));
        }
    }

    out
}

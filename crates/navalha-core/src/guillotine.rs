use crate::palette::RgbColor;
use image::{Rgba, RgbaImage};

#[derive(Debug, Clone, PartialEq)]
pub struct GuillotineConfig {
    /// Se true, executa isolamento baseado em luminância
    pub enabled: bool,
    /// Limite de luminância (0.0 a 1.0)
    pub threshold: f32,
    /// Se true, calcula dinamicamente o limiar ótimo de luminância usando o algoritmo de Otsu
    pub auto_threshold: bool,
    /// Se true, isola o sujeito que é mais escuro que o fundo (fundo claro vira transparente)
    /// Se false, isola o sujeito que é mais claro que o fundo (fundo escuro vira transparente)
    pub invert: bool,
    /// Cor sólida a ser inserida atrás da silhueta (se None, mantém transparente)
    pub background_fill: Option<RgbColor>,
}

impl Default for GuillotineConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: 0.85,
            auto_threshold: false,
            invert: true, // Padrão: remove fundos brancos/claros
            background_fill: None,
        }
    }
}

pub fn apply_guillotine(img: &RgbaImage, config: &GuillotineConfig) -> RgbaImage {
    let (width, height) = img.dimensions();
    let mut result = RgbaImage::new(width, height);

    let effective_threshold = if config.auto_threshold {
        crate::prep::compute_otsu_threshold(img)
    } else {
        config.threshold
    };

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let alpha = pixel[3];

            // Se o pixel original já tem transparência alfa
            if alpha == 0 {
                if let Some(bg) = config.background_fill {
                    result.put_pixel(x, y, Rgba([bg.r, bg.g, bg.b, 255]));
                } else {
                    result.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                }
                continue;
            }

            if config.enabled {
                let r = pixel[0] as f32 / 255.0;
                let g = pixel[1] as f32 / 255.0;
                let b = pixel[2] as f32 / 255.0;
                let lum = 0.299 * r + 0.587 * g + 0.114 * b;

                let is_background = if config.invert {
                    // Fundo é claro
                    lum >= effective_threshold
                } else {
                    // Fundo é escuro
                    lum <= effective_threshold
                };

                if is_background {
                    if let Some(bg) = config.background_fill {
                        result.put_pixel(x, y, Rgba([bg.r, bg.g, bg.b, 255]));
                    } else {
                        result.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                    }
                } else {
                    result.put_pixel(x, y, *pixel);
                }
            } else {
                result.put_pixel(x, y, *pixel);
            }
        }
    }

    result
}

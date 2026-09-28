//! Módulo de Separação de Chapas para Impressão (Risografia e Silk-Screen)
//!
//! Fornece utilitários para isolar cada cor pura de uma imagem em matrizes individuais,
//! com suporte a fotolito preto e branco (máscara de insolamento), marcas de registro
//! e exportação nativa em PDF multipágina e PNG.

use crate::palette::RgbColor;
use crate::press::EMBEDDED_FONT_BYTES;
use ab_glyph::{Font, FontRef, PxScale};
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlateError {
    #[error("Imagem vazia ou sem dimensões válidas")]
    EmptyImage,
    #[error("Nenhuma cor encontrada para gerar chapas")]
    NoColorsFound,
    #[error("Erro ao comprimir dados da imagem para PDF: {0}")]
    CompressionError(String),
}

/// Modo de renderização da chapa
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlateMode {
    /// Fotolito / Máscara de insolamento: preto (#000000) onde há tinta, branco (#FFFFFF) onde não há
    #[default]
    InsolationMask,
    /// Prova de cor: cor original isolada sobre fundo transparente
    ColorPreview,
    /// Cor original isolada sobre fundo opaco branco
    ColorOnWhite,
}

impl PlateMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "fotolito" | "insolation" | "mascara" | "mask" => Some(Self::InsolationMask),
            "colorido" | "color" | "preview" => Some(Self::ColorPreview),
            "branco" | "white" | "color-on-white" => Some(Self::ColorOnWhite),
            _ => None,
        }
    }
}

/// Configuração da separação de chapas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlateSeparationConfig {
    /// Modo de geração visual da chapa
    pub mode: PlateMode,
    /// Adicionar cruzes de mira e marcas de registro nas margens
    pub registration_marks: bool,
    /// Tamanho da margem extra em pixels em volta da arte (para marcas e texto)
    pub margin_px: u32,
    /// Incluir texto técnico informativo (HEX, % de cobertura e chapa)
    pub include_header: bool,
    /// Percentual mínimo de presença para gerar uma chapa (0.0 a 1.0, padrão 0.0005 = 0.05%)
    pub min_coverage_threshold: f32,
}

impl Default for PlateSeparationConfig {
    fn default() -> Self {
        Self {
            mode: PlateMode::InsolationMask,
            registration_marks: true,
            margin_px: 50,
            include_header: true,
            min_coverage_threshold: 0.0005,
        }
    }
}

/// Representa uma chapa individual de impressão
#[derive(Debug, Clone)]
pub struct PrintPlate {
    /// Índice sequencial da chapa (iniciando em 1)
    pub index: usize,
    /// Cor pura correspondente à chapa
    pub color: RgbColor,
    /// Código hexadecimal da cor (ex: "#FF0033")
    pub color_hex: String,
    /// Quantidade de pixels desta cor na arte
    pub pixel_count: usize,
    /// Percentual de cobertura da imagem (0.0% a 100.0%)
    pub coverage_percentage: f32,
    /// Imagem renderizada da chapa (já com margens e marcas de registro, se configurado)
    pub image: RgbaImage,
}

/// Isola cada cor presente na imagem em uma chapa de impressão
pub fn separate_plates(
    input: &RgbaImage,
    config: &PlateSeparationConfig,
) -> Result<Vec<PrintPlate>, PlateError> {
    let (width, height) = input.dimensions();
    if width == 0 || height == 0 {
        return Err(PlateError::EmptyImage);
    }

    let total_pixels = (width * height) as f32;

    // 1. Contabilizar frequências de cores opacas (ignora totalmente transparente)
    let mut color_counts: BTreeMap<(u8, u8, u8), usize> = BTreeMap::new();
    for pixel in input.pixels() {
        if pixel[3] > 32 {
            let key = (pixel[0], pixel[1], pixel[2]);
            *color_counts.entry(key).or_insert(0) += 1;
        }
    }

    if color_counts.is_empty() {
        return Err(PlateError::NoColorsFound);
    }

    // 2. Filtrar cores que atingem o limiar mínimo de cobertura
    let mut valid_colors: Vec<((u8, u8, u8), usize)> = color_counts
        .into_iter()
        .filter(|&(_, count)| (count as f32 / total_pixels) >= config.min_coverage_threshold)
        .collect();

    // Ordenar da cor mais abundante para a menos abundante
    valid_colors.sort_by(|a, b| b.1.cmp(&a.1));

    if valid_colors.is_empty() {
        return Err(PlateError::NoColorsFound);
    }

    let total_plates = valid_colors.len();
    let margin = if config.registration_marks {
        config.margin_px
    } else {
        0
    };
    let out_width = width + margin * 2;
    let out_height = height + margin * 2;

    let mut plates = Vec::with_capacity(total_plates);

    for (idx, &((r, g, b), count)) in valid_colors.iter().enumerate() {
        let plate_color = RgbColor::new(r, g, b);
        let coverage = (count as f32 / total_pixels) * 100.0;
        let plate_index = idx + 1;

        // Fundo inicial da chapa
        let mut plate_img = match config.mode {
            PlateMode::InsolationMask | PlateMode::ColorOnWhite => {
                RgbaImage::from_pixel(out_width, out_height, Rgba([255, 255, 255, 255]))
            }
            PlateMode::ColorPreview => {
                RgbaImage::from_pixel(out_width, out_height, Rgba([0, 0, 0, 0]))
            }
        };

        // Preencher a área da arte
        for y in 0..height {
            for x in 0..width {
                let px = input.get_pixel(x, y);
                let out_x = x + margin;
                let out_y = y + margin;

                if px[3] > 32 && px[0] == r && px[1] == g && px[2] == b {
                    match config.mode {
                        PlateMode::InsolationMask => {
                            // Preto puro para insolamento da matriz
                            plate_img.put_pixel(out_x, out_y, Rgba([0, 0, 0, 255]));
                        }
                        PlateMode::ColorPreview | PlateMode::ColorOnWhite => {
                            // Cor original
                            plate_img.put_pixel(out_x, out_y, Rgba([r, g, b, 255]));
                        }
                    }
                }
            }
        }

        // Adicionar marcas de registro se solicitado
        if config.registration_marks && margin >= 20 {
            draw_registration_marks(&mut plate_img, width, height, margin);
        }

        // Adicionar cabeçalho técnico se solicitado
        if config.include_header && margin >= 30 {
            let label = format!(
                "NAVALHA | CHAPA {}/{} | COR {} | COBERTURA {:.1}%",
                plate_index,
                total_plates,
                plate_color.to_hex(),
                coverage
            );
            draw_plate_header(&mut plate_img, &label, margin, out_height - margin / 2);
        }

        plates.push(PrintPlate {
            index: plate_index,
            color: plate_color,
            color_hex: plate_color.to_hex(),
            pixel_count: count,
            coverage_percentage: coverage,
            image: plate_img,
        });
    }

    Ok(plates)
}

/// Desenha marcas de registro (cruzes de mira de alta precisão) nas margens da chapa
fn draw_registration_marks(
    img: &mut RgbaImage,
    art_w: u32,
    art_h: u32,
    margin: u32,
) {
    let mark_color = Rgba([0, 0, 0, 255]); // Preto absoluto para matrizes de serigrafia

    let offset = margin / 2;
    let right = margin + art_w + offset;
    let bottom = margin + art_h + offset;
    let mid_x = margin + art_w / 2;
    let mid_y = margin + art_h / 2;

    // Centros das miras: 4 cantos + 4 pontos cardeais
    let points = [
        (offset, offset),         // Canto superior esquerdo
        (right, offset),          // Canto superior direito
        (offset, bottom),         // Canto inferior esquerdo
        (right, bottom),          // Canto inferior direito
        (mid_x, offset),          // Meio superior
        (mid_x, bottom),          // Meio inferior
        (offset, mid_y),          // Meio esquerdo
        (right, mid_y),           // Meio direito
    ];

    let radius: i32 = (margin as i32 / 3).clamp(6, 14);
    let cross_len: i32 = radius + 6;

    for (cx, cy) in points {
        let cx = cx as i32;
        let cy = cy as i32;
        let w = img.width() as i32;
        let h = img.height() as i32;

        // Linha horizontal da cruz
        for dx in -cross_len..=cross_len {
            let px = cx + dx;
            if px >= 0 && px < w && cy >= 0 && cy < h {
                img.put_pixel(px as u32, cy as u32, mark_color);
            }
        }

        // Linha vertical da cruz
        for dy in -cross_len..=cross_len {
            let py = cy + dy;
            if cx >= 0 && cx < w && py >= 0 && py < h {
                img.put_pixel(cx as u32, py as u32, mark_color);
            }
        }

        // Círculo concêntrico da mira
        for angle_deg in 0..360 {
            let rad = (angle_deg as f32).to_radians();
            let px = cx + (radius as f32 * rad.cos()).round() as i32;
            let py = cy + (radius as f32 * rad.sin()).round() as i32;
            if px >= 0 && px < w && py >= 0 && py < h {
                img.put_pixel(px as u32, py as u32, mark_color);
            }
        }
    }
}

/// Renderiza o texto informativo do cabeçalho da chapa usando a fonte embutida
fn draw_plate_header(img: &mut RgbaImage, text: &str, start_x: u32, center_y: u32) {
    if let Ok(font) = FontRef::try_from_slice(EMBEDDED_FONT_BYTES) {
        let scale = PxScale::from(14.0);
        let color = Rgba([0, 0, 0, 255]);
        let mut curr_x = start_x as f32;

        for ch in text.chars() {
            let glyph = font.glyph_id(ch).with_scale_and_position(
                scale,
                ab_glyph::point(curr_x, center_y as f32 - 7.0),
            );
            if let Some(outline) = font.outline_glyph(glyph) {
                let bounds = outline.px_bounds();
                outline.draw(|x, y, c| {
                    if c > 0.4 {
                        let px = bounds.min.x as u32 + x;
                        let py = bounds.min.y as u32 + y;
                        if px < img.width() && py < img.height() {
                            img.put_pixel(px, py, color);
                        }
                    }
                });
            }
            curr_x += scale.x * 0.58;
        }
    }
}

/// Gera um documento PDF multipágina (formato ISO 32000-1) contendo todas as chapas de impressão
pub fn generate_plates_pdf(
    plates: &[PrintPlate],
    title: Option<&str>,
) -> Result<Vec<u8>, PlateError> {
    if plates.is_empty() {
        return Err(PlateError::NoColorsFound);
    }

    let mut pdf = Vec::with_capacity(plates.len() * 64 * 1024);
    let mut offsets = Vec::new();

    // 1. Cabeçalho PDF 1.4 com bytes binários para forçar parser de 8-bits
    pdf.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    let num_plates = plates.len();

    // Estrutura de objetos:
    // Obj 1: Catalog
    // Obj 2: Pages root
    // Para cada página i (0..num_plates):
    //   Obj (3 + i*3): Page
    //   Obj (4 + i*3): Image XObject
    //   Obj (5 + i*3): Content Stream (desenha a imagem na página)

    let doc_title = title.unwrap_or("Navalha - Chapas de Impressao (Risografia / Silk-screen)");

    // Obj 1: Catalog
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // Obj 2: Pages (antecipando referências dos filhos)
    offsets.push(pdf.len());
    let mut kids = String::new();
    for i in 0..num_plates {
        let page_obj_id = 3 + i * 3;
        kids.push_str(&format!("{} 0 R ", page_obj_id));
    }
    pdf.extend_from_slice(
        format!(
            "2 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
            kids.trim(),
            num_plates
        )
        .as_bytes(),
    );

    // Iterar pelas chapas e criar os objetos de cada página
    for (i, plate) in plates.iter().enumerate() {
        let page_obj_id = 3 + i * 3;
        let image_obj_id = 4 + i * 3;
        let content_obj_id = 5 + i * 3;

        let w = plate.image.width();
        let h = plate.image.height();

        // Dimensões da página PDF em pontos (PostScript points, 72 pt/polegada)
        // Usamos as dimensões nativas em pixels como pontos para escala 1:1 exata
        let pt_w = w;
        let pt_h = h;

        // Obj Page
        offsets.push(pdf.len());
        pdf.extend_from_slice(
            format!(
                "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Resources << /XObject << /Im{} {} 0 R >> >> /Contents {} 0 R >>\nendobj\n",
                page_obj_id, pt_w, pt_h, i + 1, image_obj_id, content_obj_id
            ).as_bytes()
        );

        // Preparar stream de bytes RGB da imagem
        let mut rgb_bytes = Vec::with_capacity((w * h * 3) as usize);
        for pixel in plate.image.pixels() {
            rgb_bytes.push(pixel[0]);
            rgb_bytes.push(pixel[1]);
            rgb_bytes.push(pixel[2]);
        }

        // Comprimir com Flate/Zlib
        let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&rgb_bytes, 6);

        // Obj Image XObject
        offsets.push(pdf.len());
        pdf.extend_from_slice(
            format!(
                "{} 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>\nstream\n",
                image_obj_id, w, h, compressed.len()
            ).as_bytes()
        );
        pdf.extend_from_slice(&compressed);
        pdf.extend_from_slice(b"\nendstream\nendobj\n");

        // Obj Content Stream (desenha a imagem ocupando toda a página: q pt_w 0 0 pt_h 0 0 cm /ImX Do Q)
        offsets.push(pdf.len());
        let stream_cmd = format!("q {} 0 0 {} 0 0 cm /Im{} Do Q\n", pt_w, pt_h, i + 1);
        pdf.extend_from_slice(
            format!(
                "{} 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
                content_obj_id,
                stream_cmd.len(),
                stream_cmd.trim()
            )
            .as_bytes(),
        );
    }

    // Tabela de Referência Cruzada (xref)
    let xref_offset = pdf.len();
    let total_objs = 1 + num_plates * 3;
    pdf.extend_from_slice(format!("xref\n0 {}\n", total_objs + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");

    for offset in offsets {
        pdf.extend_from_slice(format!("{:010} 00000 n \n", offset).as_bytes());
    }

    // Trailer
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R /Info << /Title ({}) /Creator (Navalha Anti-Design Engine) >> >>\nstartxref\n{}\n%%EOF\n",
            total_objs + 1,
            doc_title,
            xref_offset
        ).as_bytes()
    );

    Ok(pdf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separate_plates_two_colors() {
        let mut img = RgbaImage::new(100, 100);
        // Metade superior vermelha, metade inferior preta
        for y in 0..100 {
            for x in 0..100 {
                if y < 50 {
                    img.put_pixel(x, y, Rgba([255, 0, 51, 255]));
                } else {
                    img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
                }
            }
        }

        let config = PlateSeparationConfig {
            mode: PlateMode::InsolationMask,
            registration_marks: true,
            margin_px: 30,
            include_header: true,
            min_coverage_threshold: 0.01,
        };

        let plates = separate_plates(&img, &config).expect("Deve separar chapas com sucesso");
        assert_eq!(plates.len(), 2, "Deve gerar exatamente duas chapas");

        // Dimensões com margem 30px em cada lado (100 + 60 = 160)
        assert_eq!(plates[0].image.width(), 160);
        assert_eq!(plates[0].image.height(), 160);

        // Ambas as chapas devem ter cerca de 50% de cobertura
        assert!((plates[0].coverage_percentage - 50.0).abs() < 1.0);
        assert!((plates[1].coverage_percentage - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_insolation_mask_values() {
        let mut img = RgbaImage::new(10, 10);
        img.put_pixel(5, 5, Rgba([255, 0, 0, 255]));

        let config = PlateSeparationConfig {
            mode: PlateMode::InsolationMask,
            registration_marks: false,
            margin_px: 0,
            include_header: false,
            min_coverage_threshold: 0.001,
        };

        let plates = separate_plates(&img, &config).expect("Deve separar chapa");
        assert_eq!(plates.len(), 1);

        // Pixel 5,5 deve ser preto (tinta a aplicar)
        assert_eq!(plates[0].image.get_pixel(5, 5), &Rgba([0, 0, 0, 255]));
        // Pixel 0,0 deve ser branco (sem tinta)
        assert_eq!(plates[0].image.get_pixel(0, 0), &Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn test_generate_plates_pdf() {
        let mut img = RgbaImage::new(40, 40);
        for y in 0..40 {
            for x in 0..40 {
                img.put_pixel(x, y, Rgba([0, 56, 168, 255]));
            }
        }

        let config = PlateSeparationConfig::default();
        let plates = separate_plates(&img, &config).expect("Deve gerar chapa");

        let pdf = generate_plates_pdf(&plates, Some("Teste Navalha")).expect("PDF gerado");
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF\n") || pdf.windows(5).any(|w| w == b"%%EOF"));
    }
}

#![allow(
    clippy::too_many_arguments,
    clippy::collapsible_if,
    clippy::manual_range_contains,
    clippy::field_reassign_with_default,
    clippy::manual_is_multiple_of
)]

pub mod ascii;
pub mod dither;
pub mod guillotine;
pub mod palette;
pub mod plates;
pub mod prep;
pub mod presets;
pub mod press;
pub mod scale;
pub mod svg;

pub use ascii::{image_to_ascii, render_ascii_to_image, ASCII_DENSITY_RAMP, ASCII_EXTENDED_RAMP};
pub use dither::{apply_dither, DitherAlgorithm};
pub use guillotine::{apply_guillotine, GuillotineConfig};
pub use palette::{Palette, RgbColor};
pub use plates::{
    generate_plates_pdf, separate_plates, PlateError, PlateMode, PlateSeparationConfig, PrintPlate,
};
pub use prep::{
    apply_image_crop, apply_image_orientation_and_zoom, apply_image_orientation_zoom_and_crop,
    apply_image_zoom, compute_otsu_threshold, prepare_plate, PrepConfig,
};
pub use presets::{AestheticPreset, FALLBACK_DEFAULT_PHRASE, PROTEST_PHRASES};
pub use press::{
    apply_sticker_border, compute_subject_seed, lcg_hash, render_manual_drawings, render_poster,
    render_poster_cancellable, BannerFontFamily, DamageEffect, DamageElement, DrawingTool,
    ManualDrawingStroke, PosterComposition, PosterFormat, PunkOverlayConfig, StickerBorder, StickerElement,
    StickerKind, TarjaBorderFinish, TarjaEffect, TextElement, EMBEDDED_FONT_BYTES,
};
pub use scale::{apply_scale, ScaleFactor};
pub use svg::generate_poster_svg;

use image::RgbaImage;

/// Executa os 4 estágios do processamento do sujeito com suporte a cancelamento cooperativo
pub fn process_subject_cancellable<F>(
    input: &RgbaImage,
    guillotine: &GuillotineConfig,
    prep: &PrepConfig,
    dither_algo: DitherAlgorithm,
    palette: &Palette,
    scale_factor: ScaleFactor,
    is_cancelled: F,
) -> Option<RgbaImage>
where
    F: Fn() -> bool,
{
    if is_cancelled() {
        return None;
    }
    // 0. Enquadramento da foto bruta: rotação, crop manual de margens e zoom granular antes do corte
    let framed = apply_image_orientation_zoom_and_crop(
        input,
        prep.rotation,
        prep.zoom,
        prep.crop_top,
        prep.crop_bottom,
        prep.crop_left,
        prep.crop_right,
    );
    if is_cancelled() {
        return None;
    }

    // 1. Guilhotina (o recorte atua sobre a imagem já enquadrada e com zoom)
    let cut = apply_guillotine(&framed, guillotine);
    if is_cancelled() {
        return None;
    }

    // 2. Preparação de Chapa (downsampling, unsharp masking, levels)
    // Como a rotação e o zoom já foram aplicados na imagem bruta, evitamos reprocessar
    let mut prep_plate_config = prep.clone();
    prep_plate_config.rotation = 0;
    prep_plate_config.zoom = 1.0;
    prep_plate_config.crop_top = 0.0;
    prep_plate_config.crop_bottom = 0.0;
    prep_plate_config.crop_left = 0.0;
    prep_plate_config.crop_right = 0.0;
    let prepped = prepare_plate(&cut, &prep_plate_config);
    if is_cancelled() {
        return None;
    }

    // 3. Retícula & Dither (halftone, atkinson, bayer, etc - tamanho do ponto preservado)
    let dithered = apply_dither(&prepped, dither_algo, palette);
    if is_cancelled() {
        return None;
    }

    // 4. Matriz de Escala (Nearest-Neighbor estrito)
    Some(apply_scale(&dithered, scale_factor))
}

/// Executa os 4 estágios do processamento do sujeito (foto bruta -> dither escalado)
pub fn process_subject(
    input: &RgbaImage,
    guillotine: &GuillotineConfig,
    prep: &PrepConfig,
    dither_algo: DitherAlgorithm,
    palette: &Palette,
    scale_factor: ScaleFactor,
) -> RgbaImage {
    process_subject_cancellable(
        input,
        guillotine,
        prep,
        dither_algo,
        palette,
        scale_factor,
        || false,
    )
    .expect("Processamento do sujeito não pode ser cancelado sem callback")
}

/// Chave determinística de identificação de estado da camada de sujeito
#[derive(Debug, Clone, PartialEq)]
pub struct SubjectCacheKey {
    pub image_signature: u64,
    pub guillotine: GuillotineConfig,
    pub prep: PrepConfig,
    pub dither_algo: DitherAlgorithm,
    pub palette_colors: Vec<(u8, u8, u8)>,
    pub scale_factor: ScaleFactor,
}

impl SubjectCacheKey {
    pub fn new(
        input: &RgbaImage,
        guillotine: &GuillotineConfig,
        prep: &PrepConfig,
        dither_algo: DitherAlgorithm,
        palette: &Palette,
        scale_factor: ScaleFactor,
    ) -> Self {
        Self {
            image_signature: compute_image_signature(input),
            guillotine: guillotine.clone(),
            prep: prep.clone(),
            dither_algo,
            palette_colors: palette.colors.iter().map(|c| (c.r, c.g, c.b)).collect(),
            scale_factor,
        }
    }
}

/// Calcula assinatura hash determinística rápida (64-bit FNV-1a com amostragem em grade)
pub fn compute_image_signature(img: &RgbaImage) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    let (w, h) = (img.width(), img.height());
    hash ^= w as u64;
    hash = hash.wrapping_mul(0x100000001b3);
    hash ^= h as u64;
    hash = hash.wrapping_mul(0x100000001b3);

    let raw = img.as_raw();
    if !raw.is_empty() {
        let step = (raw.len() / 256).max(1);
        for chunk in raw.chunks(step).take(256) {
            if let Some(&b) = chunk.first() {
                hash ^= b as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
    }
    hash
}

/// Cache em memória da camada do sujeito processado para composição reativa
#[derive(Debug, Clone, Default)]
pub struct SubjectLayerCache {
    last_key: Option<SubjectCacheKey>,
    cached_subject: Option<RgbaImage>,
    hits: u64,
    misses: u64,
}

impl SubjectLayerCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_process(
        &mut self,
        input: &RgbaImage,
        guillotine: &GuillotineConfig,
        prep: &PrepConfig,
        dither_algo: DitherAlgorithm,
        palette: &Palette,
        scale_factor: ScaleFactor,
    ) -> RgbaImage {
        self.get_or_process_cancellable(
            input,
            guillotine,
            prep,
            dither_algo,
            palette,
            scale_factor,
            || false,
        )
        .expect("Cache get_or_process sem cancelamento deve retornar imagem")
    }

    pub fn get_or_process_cancellable<F>(
        &mut self,
        input: &RgbaImage,
        guillotine: &GuillotineConfig,
        prep: &PrepConfig,
        dither_algo: DitherAlgorithm,
        palette: &Palette,
        scale_factor: ScaleFactor,
        is_cancelled: F,
    ) -> Option<RgbaImage>
    where
        F: Fn() -> bool,
    {
        if is_cancelled() {
            return None;
        }

        let key = SubjectCacheKey::new(input, guillotine, prep, dither_algo, palette, scale_factor);
        if let (Some(last_key), Some(cached)) = (&self.last_key, &self.cached_subject) {
            if *last_key == key {
                self.hits += 1;
                return Some(cached.clone());
            }
        }

        self.misses += 1;
        let processed = process_subject_cancellable(
            input,
            guillotine,
            prep,
            dither_algo,
            palette,
            scale_factor,
            &is_cancelled,
        )?;
        self.last_key = Some(key);
        self.cached_subject = Some(processed.clone());
        Some(processed)
    }

    pub fn hits(&self) -> u64 {
        self.hits
    }

    pub fn misses(&self) -> u64 {
        self.misses
    }

    pub fn clear(&mut self) {
        self.last_key = None;
        self.cached_subject = None;
    }
}

/// Geração instantânea "Foto + Frase + Estética" = Pôster pronto
pub fn generate_quick_poster(
    input: &RgbaImage,
    phrase: &str,
    preset: AestheticPreset,
) -> (RgbaImage, PosterComposition) {
    let (guillotine, prep, dither, palette, scale, comp) =
        preset.build_pipeline_and_composition(phrase);

    let processed = process_subject(input, &guillotine, &prep, dither, &palette, scale);
    let poster = render_poster(&comp, Some(&processed));

    (poster, comp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn create_test_image(w: u32, h: u32) -> RgbaImage {
        let mut img = RgbaImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let val = ((x + y) * 255 / (w + h)) as u8;
                img.put_pixel(x, y, Rgba([val, val, val, 255]));
            }
        }
        img
    }

    #[test]
    fn test_determinism_pipeline() {
        let test_img = create_test_image(100, 100);
        let palette = Palette::classic_punk();
        let guillotine = GuillotineConfig::default();
        let prep = PrepConfig::default();

        let out1 = process_subject(
            &test_img,
            &guillotine,
            &prep,
            DitherAlgorithm::Atkinson,
            &palette,
            ScaleFactor::OneX,
        );

        let out2 = process_subject(
            &test_img,
            &guillotine,
            &prep,
            DitherAlgorithm::Atkinson,
            &palette,
            ScaleFactor::OneX,
        );

        assert_eq!(out1.as_raw(), out2.as_raw(), "Saídas devem ser 100% determinísticas bit a bit");
    }

    #[test]
    fn test_quick_poster_generation() {
        let test_img = create_test_image(80, 80);
        let (poster, comp) = generate_quick_poster(
            &test_img,
            "RESISTÊNCIA SUBVERSIVA",
            AestheticPreset::PunkRiot,
        );

        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_vga256_preset() {
        let test_img = create_test_image(80, 80);
        let palette = Palette::vga_256();
        assert_eq!(palette.colors.len(), 256, "Paleta VGA deve conter exatamente 256 cores");

        let (poster, comp) = generate_quick_poster(
            &test_img,
            "MODO 13H DEMOSCENE",
            AestheticPreset::Vga256,
        );
        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_cga_preset() {
        let test_img = create_test_image(80, 80);
        let palette = Palette::cga_palette1();
        assert_eq!(palette.colors.len(), 4, "Paleta CGA 1 deve conter exatamente 4 cores");

        let (poster, comp) = generate_quick_poster(
            &test_img,
            "IBM PC 1981",
            AestheticPreset::Cga,
        );
        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_old_mac_pop_art_preset() {
        let test_img = create_test_image(80, 80);
        let palette = Palette::pop_art_mac();
        assert_eq!(palette.colors.len(), 5, "Paleta Pop Art deve conter 5 cores serigráficas");

        let (poster, comp) = generate_quick_poster(
            &test_img,
            "ANDY WARHOL 1985",
            AestheticPreset::OldMacPopArt,
        );
        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_trash_polka_preset() {
        let test_img = create_test_image(80, 80);
        let palette = Palette::trash_polka();
        assert_eq!(palette.colors.len(), 5, "Paleta Trash Polka deve conter 5 cores (P&B, cinzas e carmim)");

        let (poster, comp) = generate_quick_poster(
            &test_img,
            "BUENA VISTA 2000",
            AestheticPreset::TrashPolka,
        );
        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_brutalismo_preset() {
        let test_img = create_test_image(80, 80);
        let palette = Palette::brutalismo();
        assert_eq!(palette.colors.len(), 3, "Paleta Brutalismo deve conter 3 cores");

        let (poster, comp) = generate_quick_poster(
            &test_img,
            "CONCRETO E REVOLTA",
            AestheticPreset::Brutalismo,
        );
        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_atari_gaming_preset() {
        let test_img = create_test_image(80, 80);
        let palette = Palette::atari_gaming();
        assert_eq!(palette.colors.len(), 128, "Paleta Atari Gaming Stella deve conter 128 cores");

        let (poster, comp) = generate_quick_poster(
            &test_img,
            "INSERT COIN",
            AestheticPreset::AtariGaming,
        );
        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_mimeografo_preset() {
        let test_img = create_test_image(80, 80);
        let palette = Palette::mimeografo();
        assert_eq!(palette.colors.len(), 3, "Paleta Mimeógrafo deve conter 3 cores");

        let (poster, comp) = generate_quick_poster(
            &test_img,
            "BOLETIM CLANDESTINO",
            AestheticPreset::Mimeografo,
        );
        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_xilogravura_preset() {
        let test_img = create_test_image(80, 80);
        let palette = Palette::xilogravura();
        assert_eq!(palette.colors.len(), 2, "Paleta Xilogravura deve conter 2 cores");

        let (poster, comp) = generate_quick_poster(
            &test_img,
            "CORTE ANCOLONIAL",
            AestheticPreset::Xilogravura,
        );
        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_glitch_crt_preset() {
        let test_img = create_test_image(80, 80);
        let palette = Palette::glitch_crt();
        assert_eq!(palette.colors.len(), 4, "Paleta Glitch CRT deve conter 4 cores");

        let (poster, comp) = generate_quick_poster(
            &test_img,
            "SINAL PIRATA",
            AestheticPreset::GlitchCrt,
        );
        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_ascii_art_preset() {
        let test_img = create_test_image(80, 80);
        let palette = Palette::terminal_green();
        assert_eq!(palette.colors.len(), 4, "Paleta Terminal Verde deve conter 4 tons");

        let (poster, comp) = generate_quick_poster(
            &test_img,
            "WALDEMAR CORDEIRO 1971",
            AestheticPreset::AsciiArt,
        );
        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_tabloide_popular_preset() {
        let test_img = create_test_image(80, 80);
        let palette = Palette::tabloide_popular();
        assert_eq!(palette.colors.len(), 4, "Paleta Tabloide Popular deve conter 4 cores");

        let (poster, comp) = generate_quick_poster(
            &test_img,
            "TRAVESTI NÃO É BAGUNÇA",
            AestheticPreset::TabloidePopular,
        );
        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_navalha_sata_preset() {
        let test_img = create_test_image(80, 80);
        let palette = Palette::linoleo_ouro();
        assert_eq!(palette.colors.len(), 3, "Paleta Linóleo & Ouro Queimado deve conter 3 cores");

        let (poster, comp) = generate_quick_poster(
            &test_img,
            "NO FIO DA NAVALHA",
            AestheticPreset::NavalhaSata,
        );
        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_viagem_solitaria_preset() {
        let test_img = create_test_image(80, 80);
        let palette = Palette::kraft_olivetti();
        assert_eq!(palette.colors.len(), 3, "Paleta Kraft & Olivetti deve conter 3 cores");

        let (poster, comp) = generate_quick_poster(
            &test_img,
            "MEU CORPO MINHA LEI",
            AestheticPreset::ViagemSolitaria,
        );
        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_teatro_operario_preset() {
        let test_img = create_test_image(80, 80);
        let palette = Palette::teatro_operario();
        assert_eq!(palette.colors.len(), 3, "Paleta Teatro Operário deve conter 3 cores");

        let (poster, comp) = generate_quick_poster(
            &test_img,
            "DO PALCO À TRINCHEIRA",
            AestheticPreset::TeatroOperario,
        );
        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_fanzine_digital_preset() {
        let test_img = create_test_image(80, 80);
        let palette = Palette::cyber_bbs();
        assert_eq!(palette.colors.len(), 3, "Paleta Cyber-BBS deve conter 3 cores");

        let (poster, comp) = generate_quick_poster(
            &test_img,
            "DESCONSTRUIR PARA EXISTIR",
            AestheticPreset::FanzineDigital,
        );
        assert_eq!(poster.width(), comp.canvas_width);
        assert_eq!(poster.height(), comp.canvas_height);
    }

    #[test]
    fn test_all_presets_count() {
        assert_eq!(AestheticPreset::all().len(), 24, "O motor deve fornecer exatamente 24 presets canônicos");
        for preset in AestheticPreset::all() {
            assert!(AestheticPreset::parse(preset.id()).is_some(), "Preset {} deve ser re-parseável pelo seu ID", preset.id());
        }
    }

    #[test]
    fn test_ascii_conversion() {
        let test_img = create_test_image(40, 40);
        let dyn_img = image::DynamicImage::ImageRgba8(test_img);
        let ascii_txt = image_to_ascii(&dyn_img, 30, false);
        assert!(!ascii_txt.is_empty(), "ASCII art gerado não pode ser vazio");
        assert!(ascii_txt.contains('\n'), "ASCII art deve conter quebras de linha");
    }

    #[test]
    fn test_rotation_and_otsu() {
        let test_img = create_test_image(60, 40);
        let mut config = PrepConfig::default();
        config.rotation = 90;
        let rotated = prepare_plate(&test_img, &config);
        // Ao girar 90 graus uma imagem 60x40, a largura deve virar a altura
        assert_eq!(rotated.width(), 40);

        let otsu = compute_otsu_threshold(&test_img);
        assert!(otsu > 0.0 && otsu < 1.0, "Otsu threshold deve ser entre 0 e 1");
    }

    #[test]
    fn test_damage_effects() {
        let test_img = create_test_image(80, 80);
        let mut comp = PosterComposition::default();
        comp.canvas_width = 300;
        comp.canvas_height = 400;
        comp.overlay.damages = DamageEffect::all().to_vec();

        let poster = render_poster(&comp, Some(&test_img));
        assert_eq!(poster.width(), 300);
        assert_eq!(poster.height(), 400);

        // Verifica parse de strings
        assert_eq!(DamageEffect::parse("tiro"), Some(DamageEffect::Tiro));
        assert_eq!(DamageEffect::parse("rasgo"), Some(DamageEffect::Rasgo));
        assert_eq!(DamageEffect::parse("chuva"), Some(DamageEffect::Chuva));
        assert_eq!(DamageEffect::parse("sangue"), Some(DamageEffect::Sangue));
        assert_eq!(DamageEffect::parse("tinta"), Some(DamageEffect::Tinta));
        assert_eq!(DamageEffect::parse("faca"), Some(DamageEffect::Faca));
        assert_eq!(DamageEffect::parse("pixo"), Some(DamageEffect::Pixo));
        assert_eq!(DamageEffect::parse("censura"), Some(DamageEffect::Censura));
        assert_eq!(DamageEffect::parse("puliça"), Some(DamageEffect::Pulica));
        assert_eq!(DamageEffect::parse("pulica"), Some(DamageEffect::Pulica));
        assert_eq!(DamageEffect::parse("invalido"), None);
    }

    #[test]
    fn test_damage_element_custom_position_and_scale() {
        // 1. Testa parse_spec simples
        let d_simple = DamageElement::parse_spec("tiro").expect("Deve parsear nome simples");
        assert_eq!(d_simple.kind, DamageEffect::Tiro);
        assert_eq!(d_simple.scale, 1.0);

        // 2. Testa parse_spec completo tipo:posX:posY:escala
        let d_custom = DamageElement::parse_spec("sangue:0.35:0.65:1.5").expect("Deve parsear formato completo");
        assert_eq!(d_custom.kind, DamageEffect::Sangue);
        assert!((d_custom.pos_x - 0.35).abs() < 1e-4);
        assert!((d_custom.pos_y - 0.65).abs() < 1e-4);
        assert!((d_custom.scale - 1.5).abs() < 1e-4);

        // 3. Testa default_for para todos os tipos
        for &kind in DamageEffect::all() {
            let def = DamageElement::default_for(kind);
            assert_eq!(def.kind, kind);
            assert!(def.pos_x >= 0.0 && def.pos_x <= 1.0);
            assert!(def.pos_y >= 0.0 && def.pos_y <= 1.0);
            assert_eq!(def.scale, 1.0);
        }

        // 4. Renderização com damage_elements customizados
        let test_img = create_test_image(80, 80);
        let mut comp = PosterComposition::default();
        comp.canvas_width = 300;
        comp.canvas_height = 400;
        comp.overlay.damage_elements = vec![
            DamageElement::new(DamageEffect::Tiro, 0.25, 0.25, 1.2),
            DamageElement::new(DamageEffect::Sangue, 0.70, 0.60, 0.8),
            DamageElement::new(DamageEffect::Rasgo, 0.50, 0.10, 1.5),
            DamageElement::new(DamageEffect::Pixo, 0.30, 0.80, 1.0),
        ];

        let poster = render_poster(&comp, Some(&test_img));
        let poster2 = render_poster(&comp, Some(&test_img));
        assert_eq!(poster.width(), 300);
        assert_eq!(poster.height(), 400);
        assert_eq!(poster.as_raw(), poster2.as_raw(), "Dano Rasgo realista deve ser estritamente determinístico");
    }

    #[test]
    fn test_sticker_rendering() {
        let test_img = create_test_image(80, 80);
        let mut comp = PosterComposition::default();
        comp.canvas_width = 400;
        comp.canvas_height = 500;
        comp.palette = Some(Palette::classic_punk());

        for (i, &kind) in StickerKind::all().iter().enumerate() {
            comp.stickers.push(StickerElement::new(
                kind,
                0.2 + (i as f32 * 0.08),
                0.2 + (i as f32 * 0.08),
                64,
                15,
                i % 2 == 0,
            ));
        }

        let poster = render_poster(&comp, Some(&test_img));
        assert_eq!(poster.width(), 400);
        assert_eq!(poster.height(), 500);

        assert_eq!(StickerKind::parse("anarchy"), Some(StickerKind::Anarchy));
        assert_eq!(StickerKind::parse("razor"), Some(StickerKind::RazorBlade));
        assert_eq!(StickerKind::parse("skull"), Some(StickerKind::Skull));
        assert_eq!(StickerKind::parse("biohazard"), Some(StickerKind::Biohazard));
        assert_eq!(StickerKind::parse("copyleft"), Some(StickerKind::Copyleft));
        assert_eq!(StickerKind::parse("barcode"), Some(StickerKind::Barcode));
        assert_eq!(StickerKind::parse("safetypin"), Some(StickerKind::SafetyPin));
        assert_eq!(StickerKind::parse("smiley"), Some(StickerKind::SmileyAcid));
    }

    #[test]
    fn test_image_zoom_in_and_out() {
        let test_img = create_test_image(100, 100);

        // Zoom 1.0 (neutro)
        let z1 = apply_image_zoom(&test_img, 1.0);
        assert_eq!(z1.dimensions(), (100, 100));

        // Zoom in 2.0 (teleobjetiva / aproximação)
        let z_in = apply_image_zoom(&test_img, 2.0);
        assert_eq!(z_in.dimensions(), (100, 100));

        // Zoom out 0.5 (grande-angular / afastamento)
        let z_out = apply_image_zoom(&test_img, 0.5);
        assert_eq!(z_out.dimensions(), (100, 100));
        // No zoom out 0.5, as bordas devem ser transparentes
        assert_eq!(z_out.get_pixel(5, 5)[3], 0);
        // E o centro deve conter a imagem escalada
        assert_eq!(z_out.get_pixel(50, 50)[3], 255);
    }

    #[test]
    fn test_zoom_affects_guillotine() {
        // Cria imagem com sujeito escuro no centro e bordas claras
        let mut img = RgbaImage::new(100, 100);
        for y in 0..100 {
            for x in 0..100 {
                if x >= 30 && x <= 70 && y >= 30 && y <= 70 {
                    img.put_pixel(x, y, Rgba([20, 20, 20, 255])); // sujeito
                } else {
                    img.put_pixel(x, y, Rgba([240, 240, 240, 255])); // fundo
                }
            }
        }

        let mut guillotine = GuillotineConfig::default();
        guillotine.enabled = true;
        guillotine.threshold = 0.5;

        // Com zoom 1.0
        let mut prep1 = PrepConfig::default();
        prep1.zoom = 1.0;
        let out1 = process_subject(
            &img,
            &guillotine,
            &prep1,
            DitherAlgorithm::FloydSteinberg,
            &Palette::classic_punk(),
            ScaleFactor::OneX,
        );

        // Com zoom in 2.0, o sujeito ocupa 4x mais área
        let mut prep2 = PrepConfig::default();
        prep2.zoom = 2.0;
        let out2 = process_subject(
            &img,
            &guillotine,
            &prep2,
            DitherAlgorithm::FloydSteinberg,
            &Palette::classic_punk(),
            ScaleFactor::OneX,
        );

        let count_opaque1 = out1.pixels().filter(|p| p[3] > 0).count();
        let count_opaque2 = out2.pixels().filter(|p| p[3] > 0).count();

        // O zoom-in aumentou os pixels preservados do sujeito
        assert!(count_opaque2 > count_opaque1);
    }

    #[test]
    fn test_zoom_preserves_halftone_cell_size() {
        let test_img = create_test_image(120, 120);
        let palette = Palette::classic_punk();
        let guillotine = GuillotineConfig::default();

        let mut prep_normal = PrepConfig::default();
        prep_normal.zoom = 1.0;
        prep_normal.target_width = Some(120);

        let mut prep_zoomed = PrepConfig::default();
        prep_zoomed.zoom = 2.0;
        prep_zoomed.target_width = Some(120);

        let halftone = DitherAlgorithm::Halftone { angle_deg: 45, cell_size: 6 };

        let out1 = process_subject(&test_img, &guillotine, &prep_normal, halftone, &palette, ScaleFactor::OneX);
        let out2 = process_subject(&test_img, &guillotine, &prep_zoomed, halftone, &palette, ScaleFactor::OneX);

        // As duas saídas mantêm a mesma resolução da chapa e dimensão
        assert_eq!(out1.dimensions(), out2.dimensions());
        assert_eq!(out1.width(), 120);
        assert_eq!(out1.height(), 120);
    }

    #[test]
    fn test_poster_subject_offsets() {
        let test_img = create_test_image(100, 100);
        let mut comp = PosterComposition::default();
        comp.canvas_width = 200;
        comp.canvas_height = 200;
        comp.subject_offset_x = 20;
        comp.subject_offset_y = -30;

        let poster = render_poster(&comp, Some(&test_img));
        assert_eq!(poster.width(), 200);
        assert_eq!(poster.height(), 200);
    }

    #[test]
    fn test_xerox_podre_preset_determinism() {
        let test_img = create_test_image(80, 80);
        let preset = AestheticPreset::XeroxPodre;
        let (g1, pr1, dt1, pal1, sc1, comp1) = preset.build_pipeline_and_composition("TESTE XEROX DETERMINISMO");
        let (g2, pr2, dt2, pal2, sc2, comp2) = preset.build_pipeline_and_composition("TESTE XEROX DETERMINISMO");

        assert_eq!(comp1.foreground_texts[0].tarja_effect, TarjaEffect::Xerox);
        assert_eq!(comp2.foreground_texts[0].tarja_effect, TarjaEffect::Xerox);

        let proc1 = process_subject(&test_img, &g1, &pr1, dt1, &pal1, sc1);
        let proc2 = process_subject(&test_img, &g2, &pr2, dt2, &pal2, sc2);
        assert_eq!(proc1.as_raw(), proc2.as_raw());

        let poster1 = render_poster(&comp1, Some(&proc1));
        let poster2 = render_poster(&comp2, Some(&proc2));
        assert_eq!(poster1.as_raw(), poster2.as_raw(), "Renderização do preset XeroxPodre deve ser 100% determinística");
    }

    #[test]
    fn test_xerox_tarja_artifacts_difference() {
        let test_img = create_test_image(80, 80);
        let preset = AestheticPreset::XeroxPodre;
        let (_, _, _, _, _, comp_xerox) = preset.build_pipeline_and_composition("ARTEFATOS XEROX");
        let mut comp_solida = comp_xerox.clone();
        comp_solida.foreground_texts[0].tarja_effect = TarjaEffect::Solida;

        let poster_xerox = render_poster(&comp_xerox, Some(&test_img));
        let poster_solida = render_poster(&comp_solida, Some(&test_img));

        assert_ne!(
            poster_xerox.as_raw(),
            poster_solida.as_raw(),
            "A tarja com defeitos de xerox deve diferir da tarja sólida geométrica"
        );
    }

    #[test]
    fn test_dual_banners_support_and_visual_consistency() {
        let preset = AestheticPreset::PunkRiot;
        let (_, _, _, _, _, comp) = preset.build_pipeline_and_composition_dual(
            "FRASE INFERIOR",
            Some("MANCHETE SUPERIOR"),
        );

        assert_eq!(comp.foreground_texts.len(), 2, "Deve conter exatamente 2 faixas");
        let top = &comp.foreground_texts[0];
        let bottom = &comp.foreground_texts[1];

        assert_eq!(top.text, "MANCHETE SUPERIOR");
        assert_eq!(bottom.text, "FRASE INFERIOR");

        // Posição no topo vs base
        assert_eq!(top.pos_y, 0.08);
        assert!(bottom.pos_y > 0.80);

        // Consistência visual e estilística estrita
        assert_eq!(top.color, bottom.color, "Cores de texto devem ser idênticas");
        assert_eq!(top.bg_tarja, bottom.bg_tarja, "Cores de tarja devem ser idênticas");
        assert_eq!(top.tarja_padding, bottom.tarja_padding, "Padding da tarja deve ser idêntico");
        assert_eq!(top.tarja_effect, bottom.tarja_effect, "Efeito de tarja deve ser idêntico");
        assert_eq!(top.stretch_y, bottom.stretch_y, "Distorção vertical deve ser idêntica");
    }

    #[test]
    fn test_only_top_banner_when_bottom_empty() {
        let preset = AestheticPreset::XeroxPodre;
        let (_, _, _, _, _, comp) = preset.build_pipeline_and_composition_dual(
            "",
            Some("APENAS TOPO"),
        );

        assert_eq!(comp.foreground_texts.len(), 1, "Deve reter apenas a faixa superior quando a inferior for vazia");
        assert_eq!(comp.foreground_texts[0].text, "APENAS TOPO");
        assert_eq!(comp.foreground_texts[0].pos_y, 0.08);
        assert_eq!(comp.foreground_texts[0].tarja_effect, TarjaEffect::Xerox);
    }

    #[test]
    fn test_subject_layer_cache_hits_and_misses() {
        let img = create_test_image(80, 80);
        let g = GuillotineConfig::default();
        let pr = PrepConfig::default();
        let dt = DitherAlgorithm::Atkinson;
        let pal = Palette::classic_punk();
        let sc = ScaleFactor::OneX;

        let mut cache = SubjectLayerCache::new();
        assert_eq!(cache.hits(), 0);
        assert_eq!(cache.misses(), 0);

        // Primeira chamada: miss
        let first = cache.get_or_process(&img, &g, &pr, dt, &pal, sc);
        assert_eq!(cache.hits(), 0);
        assert_eq!(cache.misses(), 1);

        // Segunda chamada idêntica: hit imediato
        let second = cache.get_or_process(&img, &g, &pr, dt, &pal, sc);
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 1);
        assert_eq!(first.as_raw(), second.as_raw());

        // Alteração em parâmetro do sujeito: miss
        let mut pr2 = pr.clone();
        pr2.unsharp_strength = 4.5;
        let third = cache.get_or_process(&img, &g, &pr2, dt, &pal, sc);
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 2);
        assert_eq!(third.dimensions(), first.dimensions());
    }

    #[test]
    fn test_process_subject_cancellation() {
        let img = create_test_image(80, 80);
        let g = GuillotineConfig::default();
        let pr = PrepConfig::default();
        let dt = DitherAlgorithm::FloydSteinberg;
        let pal = Palette::classic_punk();
        let sc = ScaleFactor::OneX;

        // Cancelamento imediato
        let cancelled = process_subject_cancellable(&img, &g, &pr, dt, &pal, sc, || true);
        assert!(cancelled.is_none(), "Deveria abortar imediatamente quando cancelado");

        // Não cancelado
        let ok = process_subject_cancellable(&img, &g, &pr, dt, &pal, sc, || false);
        assert!(ok.is_some(), "Deveria concluir com sucesso quando não cancelado");
    }

    #[test]
    fn test_render_poster_cancellation() {
        let comp = PosterComposition::default();
        let cancelled = render_poster_cancellable(&comp, None, || true);
        assert!(cancelled.is_none(), "Renderização de pôster cancelada deve retornar None");

        let ok = render_poster_cancellable(&comp, None, || false);
        assert!(ok.is_some(), "Renderização de pôster não cancelada deve retornar Some");
    }

    #[test]
    fn test_multiline_banner_rendering() {
        let mut comp = PosterComposition::default();
        comp.foreground_texts.push(TextElement {
            text: "PRIMEIRA LINHA\nSEGUNDA LINHA\nTERCEIRA LINHA".to_string(),
            font_size: 32.0,
            stretch_x: 1.0,
            stretch_y: 1.0,
            color: RgbColor::new(255, 255, 255),
            bg_tarja: Some(RgbColor::new(0, 0, 0)),
            tarja_padding: 8,
            pos_x: 0.5,
            pos_y: 0.5,
            center_x: true,
            rotation: 0,
            tarja_effect: TarjaEffect::Solida,
            ..Default::default()
        });

        let poster = render_poster(&comp, None);
        assert_eq!(poster.dimensions(), (comp.canvas_width, comp.canvas_height));

        // Teste em SVG
        let svg = generate_poster_svg(&comp, None);
        assert!(svg.contains("<tspan"), "SVG deve conter elementos <tspan> para múltiplas linhas");
        assert!(svg.contains("PRIMEIRA LINHA"));
        assert!(svg.contains("SEGUNDA LINHA"));
    }

    #[test]
    fn test_banner_font_family_and_autofit() {
        let mut comp = PosterComposition::default();
        comp.foreground_texts.push(TextElement {
            text: "TEXTO LONGO COM AUTO FIT ATIVADO PARA CABER NA LARGURA DISPONIVEL DO CARTAZ".to_string(),
            font_size: 72.0,
            font_family: BannerFontFamily::Monospace,
            auto_fit: true,
            ..Default::default()
        });

        let poster_autofit = render_poster(&comp, None);
        assert_eq!(poster_autofit.dimensions(), (comp.canvas_width, comp.canvas_height));

        // SVG deve conter a família monospace
        let svg = generate_poster_svg(&comp, None);
        assert!(svg.contains("monospace") || svg.contains("Courier"));
    }

    #[test]
    fn test_banner_position_and_rotation() {
        let mut comp = PosterComposition::default();
        comp.foreground_texts.push(TextElement {
            text: "FAIXA INCLINADA".to_string(),
            font_size: 40.0,
            pos_x: 0.35,
            pos_y: 0.45,
            rotation: 15,
            ..Default::default()
        });

        let poster1 = render_poster(&comp, None);
        let poster2 = render_poster(&comp, None);
        assert_eq!(poster1.dimensions(), (comp.canvas_width, comp.canvas_height));
        assert_eq!(poster1.into_raw(), poster2.into_raw(), "Renderização com rotação deve ser 100% determinística");

        // SVG deve conter transform="rotate(15
        let svg = generate_poster_svg(&comp, None);
        assert!(svg.contains(r#"transform="rotate(15"#));
    }

    #[test]
    fn test_banner_colors_and_border_customization() {
        let mut comp = PosterComposition::default();
        comp.foreground_texts.push(TextElement {
            text: "TESTE BORDA E CORES".to_string(),
            font_size: 44.0,
            color: RgbColor::new(255, 238, 0),
            bg_tarja: Some(RgbColor::new(255, 0, 51)),
            border_finish: TarjaBorderFinish::Tracejada,
            border_width: 4,
            border_color: Some(RgbColor::new(0, 0, 0)),
            ..Default::default()
        });

        let poster1 = render_poster(&comp, None);
        let poster2 = render_poster(&comp, None);
        assert_eq!(poster1.dimensions(), (comp.canvas_width, comp.canvas_height));
        assert_eq!(poster1.into_raw(), poster2.into_raw());

        let svg = generate_poster_svg(&comp, None);
        assert!(svg.contains(r##"stroke="#000000""##));
        assert!(svg.contains(r#"stroke-dasharray="12,6""#));

        // Testes dos parsers
        assert_eq!(TarjaBorderFinish::parse("tracejada"), Some(TarjaBorderFinish::Tracejada));
        assert_eq!(TarjaBorderFinish::parse("picote"), Some(TarjaBorderFinish::Picote));
        assert_eq!(TarjaBorderFinish::parse("durex"), Some(TarjaBorderFinish::FitaDurex));
        assert_eq!(TarjaBorderFinish::parse("rasgado"), Some(TarjaBorderFinish::Rasgado));
        assert_eq!(TarjaBorderFinish::parse("poligonal"), Some(TarjaBorderFinish::Poligonal));
        assert_eq!(RgbColor::parse_color_str("amarelo"), Some(RgbColor::new(255, 238, 0)));
        assert_eq!(RgbColor::parse_color_str("#FF0033"), Some(RgbColor::new(255, 0, 51)));
    }

    #[test]
    fn test_banner_adapt_to_active_preset() {
        let presets = [
            AestheticPreset::PunkRiot,
            AestheticPreset::NavalhaSata,
            AestheticPreset::XeroxPodre,
            AestheticPreset::ViagemSolitaria,
            AestheticPreset::Risografia,
            AestheticPreset::AsfaltoAlerta,
            AestheticPreset::Vga256,
        ];

        for &preset in &presets {
            let banner = preset.default_banner_element("TESTE PRESET");
            assert_eq!(banner.text, "TESTE PRESET");
            assert!(banner.font_size > 30.0);

            let mut comp = PosterComposition::default();
            comp.foreground_texts = vec![banner];
            let poster = render_poster(&comp, None);
            assert_eq!(poster.dimensions(), (comp.canvas_width, comp.canvas_height));
        }

        // Casos estéticos específicos do modo adaptado
        let xerox_b = AestheticPreset::XeroxPodre.default_banner_element("XEROX");
        assert_eq!(xerox_b.tarja_effect, TarjaEffect::Xerox);
        assert_eq!(xerox_b.border_finish, TarjaBorderFinish::Picote);

        let nery_b = AestheticPreset::ViagemSolitaria.default_banner_element("NERY");
        assert_eq!(nery_b.font_family, BannerFontFamily::Monospace);
        assert_eq!(nery_b.border_finish, TarjaBorderFinish::Tracejada);

        let mimeo_b = AestheticPreset::Mimeografo.default_banner_element("MIMEO");
        assert_eq!(mimeo_b.font_family, BannerFontFamily::Monospace);

        let trash_b = AestheticPreset::TrashPolka.default_banner_element("TRASH");
        assert_eq!(trash_b.font_family, BannerFontFamily::Monospace);

        let sata_b = AestheticPreset::NavalhaSata.default_banner_element("SATA");
        assert_eq!(sata_b.font_family, BannerFontFamily::Serif);

        let riso_b = AestheticPreset::Risografia.default_banner_element("RISO");
        assert_eq!(riso_b.font_family, BannerFontFamily::Sans);
        assert_eq!(riso_b.border_finish, TarjaBorderFinish::Picote);

        let mac_b = AestheticPreset::OldMacPopArt.default_banner_element("MAC");
        assert_eq!(mac_b.font_family, BannerFontFamily::Sans);

        // Presets digitais e manchetes de choque mantêm estritamente a fonte flat Impact
        for &flat_preset in &[
            AestheticPreset::TabloidePopular,
            AestheticPreset::Vga256,
            AestheticPreset::Cga,
            AestheticPreset::AtariGaming,
            AestheticPreset::GlitchCrt,
            AestheticPreset::AsciiArt,
            AestheticPreset::FanzineDigital,
            AestheticPreset::PunkRiot,
            AestheticPreset::Brutalismo,
            AestheticPreset::Xilogravura,
        ] {
            let b = flat_preset.default_banner_element("FLAT");
            assert_eq!(
                b.font_family,
                BannerFontFamily::Impact,
                "Preset {:?} deve manter a fonte flat original Impact",
                flat_preset
            );
        }
    }

    #[test]
    fn test_all_presets_banner_adequation() {
        for &preset in AestheticPreset::all() {
            let expected_banner = preset.default_banner_element("PROLETARIADO");
            let (_, _, _, _, _, comp) = preset.build_pipeline_and_composition("PROLETARIADO");

            // Deve conter a faixa esperada correspondente ao preset
            let found = comp.foreground_texts.iter().find(|t| t.text == "PROLETARIADO");
            assert!(
                found.is_some(),
                "Preset {:?} deve conter a faixa da frase principal",
                preset
            );
            let banner = found.unwrap();
            assert_eq!(banner.font_family, expected_banner.font_family);
            assert_eq!(banner.border_finish, expected_banner.border_finish);
            assert_eq!(banner.border_width, expected_banner.border_width);
            assert_eq!(banner.tarja_effect, expected_banner.tarja_effect);
            assert_eq!(banner.bg_tarja, expected_banner.bg_tarja);
            assert_eq!(banner.color, expected_banner.color);
        }
    }

    #[test]
    fn test_sem_faixa_and_preset_guillotine_override() {
        let (mut g, pr, dt, pal, sc, mut comp) =
            AestheticPreset::XeroxPodre.build_pipeline_and_composition("TESTE SEM FAIXA");

        // Validação da desativação/remoção completa de faixa (modo sem faixa)
        assert!(!comp.foreground_texts.is_empty());
        comp.foreground_texts.clear();
        assert!(comp.foreground_texts.is_empty());

        let raw = image::RgbaImage::new(100, 100);
        let poster = render_poster(&comp, None);
        assert_eq!(poster.dimensions(), (comp.canvas_width, comp.canvas_height));

        let svg = generate_poster_svg(&comp, None);
        assert!(!svg.contains("TESTE SEM FAIXA"), "SVG no modo sem faixa não deve conter o texto");

        // Validação de override de guilhotina sobre o preset
        assert!(g.enabled);
        let processed_normal = process_subject(&raw, &g, &pr, dt, &pal, sc);

        // Override desativando guilhotina
        g.enabled = false;
        let processed_no_cut = process_subject(&raw, &g, &pr, dt, &pal, sc);

        // Override customizando limiar e invertendo
        g.enabled = true;
        g.threshold = 0.25;
        g.invert = false;
        let processed_custom = process_subject(&raw, &g, &pr, dt, &pal, sc);

        assert_eq!(processed_normal.dimensions(), processed_no_cut.dimensions());
        assert_eq!(processed_normal.dimensions(), processed_custom.dimensions());
    }

    #[test]
    fn test_sticker_borders_and_elements() {
        assert_eq!(StickerBorder::parse("diecut"), Some(StickerBorder::DieCutBranco));
        assert_eq!(StickerBorder::parse("picote"), Some(StickerBorder::Picote));
        assert_eq!(StickerBorder::parse("serrilhada"), Some(StickerBorder::Serrilhada));
        assert_eq!(StickerBorder::parse("preta"), Some(StickerBorder::LinhaPreta));
        assert_eq!(StickerBorder::parse("nenhuma"), Some(StickerBorder::Nenhuma));
        assert_eq!(StickerBorder::parse("invalido"), None);

        let mut comp = PosterComposition {
            canvas_width: 300,
            canvas_height: 400,
            ..Default::default()
        };
        let s1 = StickerElement::new(StickerKind::Skull, 0.5, 0.5, 80, 15, false)
            .with_border(StickerBorder::DieCutBranco, 6);
        let s2 = StickerElement::new(StickerKind::Anarchy, 0.2, 0.2, 60, -10, true)
            .with_border(StickerBorder::Picote, 4);
        let s3 = StickerElement::new(StickerKind::RazorBlade, 0.8, 0.8, 70, 0, false)
            .with_border(StickerBorder::Serrilhada, 5);

        comp.stickers.push(s1);
        comp.stickers.push(s2);
        comp.stickers.push(s3);

        let poster = render_poster(&comp, None);
        assert_eq!(poster.dimensions(), (300, 400));

        let svg = generate_poster_svg(&comp, None);
        assert!(svg.contains("stroke-dasharray"), "SVG deve conter borda picotada");
        assert!(svg.contains("stroke=\"#777777\""), "SVG deve conter diecut branco");
    }

    #[test]
    fn test_custom_image_sticker_rendering() {
        assert_eq!(StickerKind::parse("custom"), Some(StickerKind::Custom));
        assert_eq!(StickerKind::parse("proprio"), Some(StickerKind::Custom));
        assert_eq!(StickerKind::parse("personalizado"), Some(StickerKind::Custom));

        let mut custom_img = image::RgbaImage::new(40, 40);
        for y in 0..40 {
            for x in 0..40 {
                custom_img.put_pixel(x, y, image::Rgba([255, 0, 50, 255]));
            }
        }
        let arc_img = std::sync::Arc::new(custom_img);

        let mut comp = PosterComposition {
            canvas_width: 320,
            canvas_height: 480,
            ..Default::default()
        };

        let custom_sticker = StickerElement::new_custom(arc_img.clone(), 0.5, 0.5, 60, -15, false)
            .with_border(StickerBorder::DieCutBranco, 5);

        assert_eq!(custom_sticker.kind, StickerKind::Custom);
        assert!(custom_sticker.custom_image.is_some());
        assert_eq!(custom_sticker.border, StickerBorder::DieCutBranco);

        comp.stickers.push(custom_sticker);

        let poster = render_poster(&comp, None);
        assert_eq!(poster.dimensions(), (320, 480));

        let svg = generate_poster_svg(&comp, None);
        assert!(svg.contains("data:image/png;base64,"), "SVG deve embutir o sticker customizado como imagem base64");
    }

    #[test]
    fn test_xerox_podre_stays_strictly_straight() {
        let banner = AestheticPreset::XeroxPodre.default_banner_element("RESISTÊNCIA DADAÍSTA");
        assert_eq!(banner.wave_amplitude, 0.0, "XeroxPodre deve ter ondulação desativada");
        assert_eq!(banner.word_jitter_rotation, 0, "XeroxPodre não deve ter rotação de palavras");
        assert!(!banner.word_tarja_cut, "XeroxPodre deve usar tarja contínua tradicional e reta");
    }

    #[test]
    fn test_presets_wave_and_word_cut_distribution() {
        // Presets punk/colagem com palavras recortadas
        for preset in [
            AestheticPreset::PunkRiot,
            AestheticPreset::TrashPolka,
            AestheticPreset::TabloidePopular,
            AestheticPreset::AsfaltoAlerta,
            AestheticPreset::Punk80Br,
            AestheticPreset::AnarcoZine,
        ] {
            let b = preset.default_banner_element("PROTESTO PUNK");
            assert!(b.word_tarja_cut, "Preset {:?} deve ter word_tarja_cut ativo", preset);
            assert!(b.word_jitter_rotation > 0, "Preset {:?} deve ter jitter angular ativo", preset);
        }

        // Presets com ondulação não-linear
        for preset in [
            AestheticPreset::GlitchCrt,
            AestheticPreset::FanzineDigital,
            AestheticPreset::LapaNoturna,
        ] {
            let b = preset.default_banner_element("ONDA ANALÓGICA");
            assert!(b.wave_amplitude > 0.0, "Preset {:?} deve ter wave_amplitude > 0", preset);
        }

        // Presets sóbrios estritamente retos
        for preset in [
            AestheticPreset::XeroxPodre,
            AestheticPreset::ViagemSolitaria,
            AestheticPreset::Brutalismo,
            AestheticPreset::TeatroOperario,
            AestheticPreset::AsciiArt,
            AestheticPreset::Vga256,
            AestheticPreset::Cga,
            AestheticPreset::AtariGaming,
            AestheticPreset::Risografia,
            AestheticPreset::Mimeografo,
            AestheticPreset::Xilogravura,
        ] {
            let b = preset.default_banner_element("TARJA RETA");
            assert_eq!(b.wave_amplitude, 0.0, "Preset {:?} deve ser reto", preset);
            assert_eq!(b.word_jitter_rotation, 0, "Preset {:?} deve ter jitter 0", preset);
            assert!(!b.word_tarja_cut, "Preset {:?} não deve ter recorte individual", preset);
        }
    }

    #[test]
    fn test_wave_and_word_cut_determinism_with_image_seed() {
        let (_, _, _, _, _, comp) = AestheticPreset::PunkRiot.build_pipeline_and_composition("NUNCA SEREMOS DÓCEIS");

        // Cria imagem com padrão específico
        let mut img = image::RgbaImage::new(120, 160);
        for y in 0..160 {
            for x in 0..120 {
                let val = ((x * 7 + y * 13) % 256) as u8;
                img.put_pixel(x, y, image::Rgba([val, 255 - val, val / 2, 255]));
            }
        }

        let poster1 = render_poster(&comp, Some(&img));
        let poster2 = render_poster(&comp, Some(&img));

        // Determinismo absoluto: a mesma imagem e mesma frase geram pixels 100% idênticos
        assert_eq!(poster1.as_raw(), poster2.as_raw(), "Renderizações repetidas devem ser deterministicamente idênticas");

        // Imagem diferente gera pixels diferentes
        let mut img_diff = img.clone();
        img_diff.put_pixel(50, 50, image::Rgba([0, 0, 0, 255]));
        let poster_diff = render_poster(&comp, Some(&img_diff));
        assert_ne!(poster1.as_raw(), poster_diff.as_raw(), "Imagens de entrada distintas geram sementes distintas");
    }

    #[test]
    fn test_punk80br_and_anarcozine_presets() {
        // Validação Punk 80 BR
        let p80 = AestheticPreset::Punk80Br;
        assert_eq!(p80.id(), "punk-80-br");
        assert!(AestheticPreset::parse("punk-80-br").is_some());
        assert!(AestheticPreset::parse("k7").is_some());
        assert!(AestheticPreset::parse("marinamathey").is_some());

        let (_g80, _pr80, dt80, pal80, _sc80, comp80) = p80.build_pipeline_and_composition("BOTAS E GRAVURAS");
        assert_eq!(dt80, DitherAlgorithm::Atkinson);
        assert_eq!(pal80.colors.len(), 3);
        assert!(comp80.stickers.iter().any(|s| s.kind == StickerKind::Cassette));
        assert!(comp80.overlay.damage_elements.iter().any(|d| d.kind == DamageEffect::Rasgo));

        // Validação Anarco-Zine
        let az = AestheticPreset::AnarcoZine;
        assert_eq!(az.id(), "anarco-zine");
        assert!(AestheticPreset::parse("anarco-zine").is_some());
        assert!(AestheticPreset::parse("stencil").is_some());
        assert!(AestheticPreset::parse("jaquelinegomesdejesus").is_some());

        let (_gaz, _praz, dtaz, palaz, _scav, compaz) = az.build_pipeline_and_composition("REBELDIA LIBERTÁRIA");
        assert!(matches!(dtaz, DitherAlgorithm::Halftone { .. }));
        assert_eq!(palaz.colors.len(), 3);
        assert!(compaz.stickers.iter().any(|s| s.kind == StickerKind::Anarchy));
        assert!(compaz.overlay.damage_elements.iter().any(|d| d.kind == DamageEffect::Grampo));

        // Teste de determinismo de renderização
        let test_img = create_test_image(80, 80);
        let poster_p80_1 = render_poster(&comp80, Some(&test_img));
        let poster_p80_2 = render_poster(&comp80, Some(&test_img));
        assert_eq!(poster_p80_1.as_raw(), poster_p80_2.as_raw(), "Punk80Br deve ser 100% determinístico");

        let poster_az_1 = render_poster(&compaz, Some(&test_img));
        let poster_az_2 = render_poster(&compaz, Some(&test_img));
        assert_eq!(poster_az_1.as_raw(), poster_az_2.as_raw(), "AnarcoZine deve ser 100% determinístico");
    }

    #[test]
    fn test_image_crop_functionality() {
        let mut img = image::RgbaImage::new(100, 100);
        for y in 0..100 {
            for x in 0..100 {
                img.put_pixel(x, y, image::Rgba([x as u8, y as u8, 128, 255]));
            }
        }

        // Teste de crop zero (identidade)
        let uncropped = apply_image_crop(&img, 0.0, 0.0, 0.0, 0.0);
        assert_eq!(uncropped.dimensions(), (100, 100));
        assert_eq!(uncropped.as_raw(), img.as_raw());

        // Teste de corte de 10% nas quatro bordas
        let cropped = apply_image_crop(&img, 10.0, 10.0, 10.0, 10.0);
        assert_eq!(cropped.dimensions(), (80, 80));

        // Teste de corte assimétrico
        let asym = apply_image_crop(&img, 20.0, 10.0, 5.0, 15.0);
        assert_eq!(asym.dimensions(), (80, 70));

        // Teste integrado de rotação, corte e zoom
        let framed = apply_image_orientation_zoom_and_crop(&img, 0, 1.0, 10.0, 10.0, 10.0, 10.0);
        assert_eq!(framed.dimensions(), (80, 80));
    }

    #[test]
    fn test_ransom_note_magazine_collage_mode() {
        let (_, _, _, _, _, mut comp) = AestheticPreset::PunkRiot.build_pipeline_and_composition("PUNK FANZINE REVOLTA");
        if let Some(ref mut elem) = comp.foreground_texts.get_mut(0) {
            elem.ransom_note = true;
        }

        let test_img = create_test_image(80, 80);
        let poster1 = render_poster(&comp, Some(&test_img));
        let poster2 = render_poster(&comp, Some(&test_img));

        // Determinismo rigoroso em modo colagem de revista
        assert_eq!(poster1.as_raw(), poster2.as_raw(), "Modo Ransom Note deve ser 100% determinístico");
        assert_eq!(poster1.dimensions(), (800, 1100));
    }

    #[test]
    fn test_raw_image_seed_stability_across_filter_changes() {
        let raw_img = create_test_image(100, 100);
        let phrase = "RESISTÊNCIA SUBVERSIVA";

        // Seed determinístico calculado sobre a foto bruta não editada (Ticket #19)
        let stable_seed = compute_subject_seed(Some(&raw_img), phrase);

        // Simulamos duas variações extremas de limiar de corte da guilhotina (ex: 40% vs 85%)
        let mut g1 = GuillotineConfig::default();
        g1.enabled = true;
        g1.threshold = 0.40;
        let mut g2 = GuillotineConfig::default();
        g2.enabled = true;
        g2.threshold = 0.85;

        let prep = PrepConfig::default();
        let palette = Palette::classic_punk();

        let cut1 = process_subject(&raw_img, &g1, &prep, DitherAlgorithm::Atkinson, &palette, ScaleFactor::OneX);
        let cut2 = process_subject(&raw_img, &g2, &prep, DitherAlgorithm::Atkinson, &palette, ScaleFactor::OneX);

        // Os sujeitos processados diferem nos seus pixels
        assert_ne!(cut1.as_raw(), cut2.as_raw(), "Sujeitos com limiares distintos devem ter pixels distintos");

        // Porém, com custom_seed ancorado na foto original, as duas composições usam a mesma semente
        let mut comp1 = PosterComposition {
            custom_seed: Some(stable_seed),
            ..Default::default()
        };
        let mut comp2 = PosterComposition {
            custom_seed: Some(stable_seed),
            ..Default::default()
        };

        let mut banner = AestheticPreset::PunkRiot.default_banner_element(phrase);
        banner.word_jitter_rotation = 8;
        banner.wave_amplitude = 12.0;

        comp1.foreground_texts.push(banner.clone());
        comp2.foreground_texts.push(banner);

        // Renderizamos pôsteres apenas com a camada de texto (None no sujeito) para isolar o seed da faixa
        let text_layer_1 = render_poster(&comp1, None);
        let text_layer_2 = render_poster(&comp2, None);

        // A camada tipográfica é 100% idêntica e estável, imune a variações de limiar da imagem!
        assert_eq!(text_layer_1.as_raw(), text_layer_2.as_raw(), "O seed fixado na imagem bruta garante estabilidade total das palavras");
    }

    #[test]
    fn test_manual_drawing_tools_parsing_and_serialization() {
        assert_eq!(DrawingTool::parse("freehand"), Some(DrawingTool::Freehand));
        assert_eq!(DrawingTool::parse("mão-livre"), Some(DrawingTool::Freehand));
        assert_eq!(DrawingTool::parse("mao-livre"), Some(DrawingTool::Freehand));
        assert_eq!(DrawingTool::parse("cross"), Some(DrawingTool::Cross));
        assert_eq!(DrawingTool::parse("cruz"), Some(DrawingTool::Cross));
        assert_eq!(DrawingTool::parse("cruz-punk"), Some(DrawingTool::Cross));
        assert_eq!(DrawingTool::parse("arrow"), Some(DrawingTool::Arrow));
        assert_eq!(DrawingTool::parse("seta"), Some(DrawingTool::Arrow));
        assert_eq!(DrawingTool::parse("line"), Some(DrawingTool::Line));
        assert_eq!(DrawingTool::parse("reta"), Some(DrawingTool::Line));
        assert_eq!(DrawingTool::parse("desconhecido"), None);

        let stroke = ManualDrawingStroke::new(
            DrawingTool::Cross,
            vec![(0.2, 0.3), (0.4, 0.5)],
            RgbColor::new(255, 0, 51),
            6.0,
        );
        assert_eq!(stroke.tool, DrawingTool::Cross);
        assert_eq!(stroke.points.len(), 2);
        assert_eq!(stroke.color, RgbColor::new(255, 0, 51));
        assert_eq!(stroke.width, 6.0);
    }

    #[test]
    fn test_manual_drawing_raster_and_determinism() {
        let mut comp = PosterComposition {
            canvas_width: 200,
            canvas_height: 200,
            background_color: RgbColor::new(245, 242, 235),
            ..Default::default()
        };

        // Traço 1: Mão livre
        comp.manual_drawings.push(ManualDrawingStroke::new(
            DrawingTool::Freehand,
            vec![(0.1, 0.1), (0.2, 0.15), (0.3, 0.1)],
            RgbColor::new(0, 0, 0),
            4.0,
        ));
        // Traço 2: Cruz punk
        comp.manual_drawings.push(ManualDrawingStroke::new(
            DrawingTool::Cross,
            vec![(0.5, 0.5), (0.7, 0.7)],
            RgbColor::new(255, 0, 51),
            5.0,
        ));
        // Traço 3: Seta direcional
        comp.manual_drawings.push(ManualDrawingStroke::new(
            DrawingTool::Arrow,
            vec![(0.1, 0.8), (0.8, 0.8)],
            RgbColor::new(0, 0, 0),
            4.0,
        ));
        // Traço 4: Linha reta
        comp.manual_drawings.push(ManualDrawingStroke::new(
            DrawingTool::Line,
            vec![(0.2, 0.9), (0.7, 0.9)],
            RgbColor::new(255, 0, 51),
            3.0,
        ));

        let poster1 = render_poster(&comp, None);
        let poster2 = render_poster(&comp, None);

        assert_eq!(poster1.dimensions(), (200, 200));
        assert_eq!(
            poster1.as_raw(),
            poster2.as_raw(),
            "Desenho manual deve ser 100% determinístico e reproduzível"
        );

        // Verifica que houve pixels pintados na cor preta e vermelha
        let has_black = poster1.pixels().any(|p| p[0] == 0 && p[1] == 0 && p[2] == 0);
        let has_red = poster1.pixels().any(|p| p[0] == 255 && p[1] == 0 && p[2] == 51);
        assert!(has_black, "Pôster deve conter traços pretos renderizados");
        assert!(has_red, "Pôster deve conter traços vermelhos renderizados");

        // SVG export
        let svg = generate_poster_svg(&comp, None);
        assert!(svg.contains("<polyline"), "SVG deve conter elemento polyline para mão livre");
        assert!(svg.contains("<line"), "SVG deve conter elementos line para cruz, seta e linha");
        assert!(svg.contains("#FF0033"), "SVG deve conter a cor vermelha punk");
    }

    #[test]
    fn test_paulete_and_symmy_presets() {
        // 1. Paulete Furacão
        let p_furacao = AestheticPreset::PauleteFuracao;
        assert_eq!(p_furacao.id(), "paulete-furacao");
        assert_eq!(AestheticPreset::parse("paulete-furacao"), Some(AestheticPreset::PauleteFuracao));
        assert_eq!(AestheticPreset::parse("paulete"), Some(AestheticPreset::PauleteFuracao));
        assert_eq!(AestheticPreset::parse("comunicacao-popular"), Some(AestheticPreset::PauleteFuracao));
        let (_, _, dither_p, pal_p, _, comp_p) = p_furacao.build_pipeline_and_composition("VOZ DAS RUAS");
        assert_eq!(dither_p, DitherAlgorithm::SauvolaWoodcut);
        assert_eq!(pal_p.name, "Comunicação Popular // Linogravura & Voz das Ruas (Paulete Furacão)");
        assert_eq!(comp_p.background_color, RgbColor::new(246, 240, 226));
        let poster_p = render_poster(&comp_p, None);
        assert_eq!(poster_p.dimensions(), (800, 1100));

        // 2. Symmy Larrat
        let p_symmy = AestheticPreset::SymmyLarrat;
        assert_eq!(p_symmy.id(), "symmy-larrat");
        assert_eq!(AestheticPreset::parse("symmy-larrat"), Some(AestheticPreset::SymmyLarrat));
        assert_eq!(AestheticPreset::parse("symmy"), Some(AestheticPreset::SymmyLarrat));
        assert_eq!(AestheticPreset::parse("imprensa-amazonica"), Some(AestheticPreset::SymmyLarrat));
        let (_, _, dither_s, pal_s, _, comp_s) = p_symmy.build_pipeline_and_composition("FLORESTA VIVA");
        assert_eq!(dither_s, DitherAlgorithm::Halftone { angle_deg: 45, cell_size: 4 });
        assert_eq!(pal_s.name, "Imprensa Amazônica // Retícula Viva & Resistência (Symmy Larrat)");
        assert_eq!(comp_s.background_color, RgbColor::new(13, 20, 18));
        let poster_s = render_poster(&comp_s, None);
        assert_eq!(poster_s.dimensions(), (800, 1100));
    }

    #[test]
    fn test_embedded_font_families_and_ransom_note() {
        let test_img = create_test_image(80, 80);

        for fam in [
            BannerFontFamily::Impact,
            BannerFontFamily::Monospace,
            BannerFontFamily::Serif,
            BannerFontFamily::Sans,
        ] {
            let (_, _, _, _, _, mut comp) = AestheticPreset::PunkRiot.build_pipeline_and_composition("TIPOGRAFIA CRUA");
            if let Some(ref mut elem) = comp.foreground_texts.get_mut(0) {
                elem.font_family = fam;
            }
            let poster = render_poster(&comp, Some(&test_img));
            assert_eq!(poster.dimensions(), (800, 1100), "Renderização com família {:?} deve ter dimensões corretas", fam);
        }

        // Teste de determinismo estrito no modo Ransom Note com múltiplas fontes
        let (_, _, _, _, _, mut comp_ransom) = AestheticPreset::ViagemSolitaria.build_pipeline_and_composition("MÁQUINA OLIVETTI JORNAL REVOLTA");
        if let Some(ref mut elem) = comp_ransom.foreground_texts.get_mut(0) {
            elem.ransom_note = true;
        }
        let p1 = render_poster(&comp_ransom, Some(&test_img));
        let p2 = render_poster(&comp_ransom, Some(&test_img));
        assert_eq!(p1.as_raw(), p2.as_raw(), "Modo Ransom Note com múltiplas fontes deve ser 100% determinístico");
    }

    #[test]
    fn test_poster_formats_horizontal_and_quadrado() {
        use crate::press::PosterFormat;

        assert_eq!(PosterFormat::Vertical.dimensions(), (800, 1100));
        assert_eq!(PosterFormat::Horizontal.dimensions(), (1100, 800));
        assert_eq!(PosterFormat::Quadrado.dimensions(), (1000, 1000));

        assert_eq!(PosterFormat::parse("horizontal"), Some(PosterFormat::Horizontal));
        assert_eq!(PosterFormat::parse("paisagem"), Some(PosterFormat::Horizontal));
        assert_eq!(PosterFormat::parse("quadrado"), Some(PosterFormat::Quadrado));
        assert_eq!(PosterFormat::parse("square"), Some(PosterFormat::Quadrado));
        assert_eq!(PosterFormat::parse("vertical"), Some(PosterFormat::Vertical));
        assert_eq!(PosterFormat::parse("desconhecido"), None);

        let test_img = create_test_image(80, 80);
        let (_, _, _, _, _, comp) = AestheticPreset::PunkRiot.build_pipeline_and_composition("SUBVERSÃO");

        let comp_horiz = comp.clone().with_format(PosterFormat::Horizontal);
        assert_eq!(comp_horiz.canvas_width, 1100);
        assert_eq!(comp_horiz.canvas_height, 800);
        let poster_h = render_poster(&comp_horiz, Some(&test_img));
        assert_eq!(poster_h.dimensions(), (1100, 800));

        let comp_quad = comp.clone().with_format(PosterFormat::Quadrado);
        assert_eq!(comp_quad.canvas_width, 1000);
        assert_eq!(comp_quad.canvas_height, 1000);
        let poster_q = render_poster(&comp_quad, Some(&test_img));
        assert_eq!(poster_q.dimensions(), (1000, 1000));
    }

    #[test]
    fn test_ransom_note_word_spacing_and_overlap_prevention() {
        use crate::press::TextElement;

        let elem_normal = TextElement {
            text: String::from("EU SOBREVIVO AO MUNDO"),
            ransom_note: true,
            font_size: 48.0,
            word_spacing: 1.0,
            ..Default::default()
        };

        let mut elem_spaced = elem_normal.clone();
        elem_spaced.word_spacing = 2.0;

        let comp1 = PosterComposition {
            canvas_width: 800,
            canvas_height: 1100,
            foreground_texts: vec![elem_normal],
            ..Default::default()
        };

        let comp2 = PosterComposition {
            canvas_width: 800,
            canvas_height: 1100,
            foreground_texts: vec![elem_spaced],
            ..Default::default()
        };

        let poster1 = render_poster(&comp1, None);
        let poster2 = render_poster(&comp2, None);

        // A renderização com maior espaçamento produz distribuição diferente de pixels não-fundo
        let diff_pixels = poster1
            .pixels()
            .zip(poster2.pixels())
            .filter(|(p1, p2)| p1 != p2)
            .count();

        assert!(diff_pixels > 0, "Deveria haver diferenca visual entre word_spacing 1.0 e 2.0");
    }

    #[test]
    fn test_protest_phrases_collection() {
        assert!(PROTEST_PHRASES.len() >= 50, "Coleção deve conter cerca de 50 frases de protesto");
        assert!(PROTEST_PHRASES.contains(&FALLBACK_DEFAULT_PHRASE), "Coleção deve conter o fallback seguro");
        for phrase in PROTEST_PHRASES {
            assert!(!phrase.trim().is_empty(), "Frase não pode ser vazia");
            assert_eq!(*phrase, phrase.to_uppercase(), "Frase deve estar em caixa alta para pôsteres");
        }
    }
}





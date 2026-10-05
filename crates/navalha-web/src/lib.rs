#![allow(
    non_snake_case,
    clippy::collapsible_if,
    clippy::redundant_closure,
    clippy::useless_borrows_in_formatting,
    clippy::too_many_arguments,
    clippy::type_complexity
)]

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use image::{ImageFormat, ImageReader, Rgba, RgbaImage};
use navalha_core::{
    generate_poster_svg, AestheticPreset, DamageEffect, DamageElement,
    DitherAlgorithm, Palette, ScaleFactor, StickerBorder, StickerElement, StickerKind, TarjaEffect,
};
use navalha_core::plates::{generate_plates_pdf, separate_plates, PlateSeparationConfig};
use std::io::Cursor;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlAnchorElement, HtmlInputElement};

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    dioxus::launch(App);
}

fn create_default_graphic(w: u32, h: u32) -> RgbaImage {
    let mut img = RgbaImage::new(w, h);
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let r_head = w as f32 * 0.32;

    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            // Desenha silhueta gráfica tosca para início imediato
            if dist < r_head {
                let inside_eye1 = (x as f32 - (cx - 35.0)).powi(2) + (y as f32 - (cy - 20.0)).powi(2) < 220.0;
                let inside_eye2 = (x as f32 - (cx + 35.0)).powi(2) + (y as f32 - (cy - 20.0)).powi(2) < 220.0;
                let inside_mouth = y as f32 > cy + 25.0 && (y as f32) < cy + 45.0 && dx.abs() < 40.0;

                if inside_eye1 || inside_eye2 || inside_mouth {
                    img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                } else {
                    let grad = ((dist / r_head) * 180.0) as u8;
                    img.put_pixel(x, y, Rgba([grad, grad, grad, 255]));
                }
            } else {
                // Fundo claro recortável pela guilhotina
                img.put_pixel(x, y, Rgba([245, 245, 245, 255]));
            }
        }
    }
    img
}

/// Catálogo de 20 imagens temáticas genéricas com licença Creative Commons / Domínio Público
pub const BACKGROUND_COLLECTION: &[(&str, &str)] = &[
    ("assets/backgrounds/01-busto-marmore.jpg", "Busto de Mármore Clássico"),
    ("assets/backgrounds/02-gargula-gotica.jpg", "Gárgula Gótica de Pedra"),
    ("assets/backgrounds/03-monumento-brutalista.jpg", "Monumento Brutalista Spomenik"),
    ("assets/backgrounds/04-o-grito-munch.jpg", "O Grito (Edvard Munch)"),
    ("assets/backgrounds/05-esqueleto-vesalius.jpg", "Esqueleto Anatômico (Vesalius)"),
    ("assets/backgrounds/06-onda-kanagawa.jpg", "A Grande Onda de Kanagawa"),
    ("assets/backgrounds/07-caravaggio-claroscuro.jpg", "São Jerônimo (Caravaggio)"),
    ("assets/backgrounds/08-corvo-urbano.jpg", "Corvo Urbano"),
    ("assets/backgrounds/09-cachorro-rua.jpg", "Cachorro em Alerta"),
    ("assets/backgrounds/10-pombo-urbano.jpg", "Pombo Urbano"),
    ("assets/backgrounds/11-caveira-anatomica.jpg", "Crânio Humano"),
    ("assets/backgrounds/12-protesto-rua.jpg", "Manifestação de Rua e Fumaça"),
    ("assets/backgrounds/13-cartazes-muro.jpg", "Cartazes e Lambe-Lambe"),
    ("assets/backgrounds/14-pichacao-urbana.jpg", "Pichação e Grafite"),
    ("assets/backgrounds/15-predio-brutalista.jpg", "Fachada de Edifício Brutalista"),
    ("assets/backgrounds/16-torre-transmissao.jpg", "Torre de Transmissão Elétrica"),
    ("assets/backgrounds/17-ruina-industrial.jpg", "Ruína Industrial Desativada"),
    ("assets/backgrounds/18-escada-incendio.jpg", "Escadaria de Incêndio de Ferro"),
    ("assets/backgrounds/19-televisao-crt.jpg", "Televisor de Tubo CRT Vintage"),
    ("assets/backgrounds/20-fita-cassete.jpg", "Fita Cassete K7 Analógica"),
];


/// Busca assincronamente os bytes de uma imagem remota do acervo e decodifica para RGBA
pub async fn load_remote_image(url: &str) -> Option<RgbaImage> {
    let window = web_sys::window()?;
    let promise = window.fetch_with_str(url);
    let resp_val = wasm_bindgen_futures::JsFuture::from(promise).await.ok()?;
    let resp: web_sys::Response = resp_val.dyn_into().ok()?;
    if !resp.ok() {
        return None;
    }
    let array_buffer_promise = resp.array_buffer().ok()?;
    let buf_val = wasm_bindgen_futures::JsFuture::from(array_buffer_promise).await.ok()?;
    let uint8_arr = js_sys::Uint8Array::new(&buf_val);
    let bytes = uint8_arr.to_vec();
    let decoded = ImageReader::new(Cursor::new(bytes)).with_guessed_format().ok()?.decode().ok()?;
    Some(decoded.to_rgba8())
}

/// Sorteia um índice aleatório válido dentro do catálogo de backgrounds
pub fn pick_random_background_index() -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        let rand = js_sys::Math::random();
        let idx = (rand * (BACKGROUND_COLLECTION.len() as f64)).floor() as usize;
        idx.min(BACKGROUND_COLLECTION.len() - 1)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::SystemTime;
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(42);
        (nanos as usize) % BACKGROUND_COLLECTION.len()
    }
}



fn trigger_download(href: &str, filename: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Ok(elem) = document.create_element("a") {
                let a: HtmlAnchorElement = elem.unchecked_into();
                a.set_href(href);
                a.set_download(filename);
                a.click();
            }
        }
    }
}

fn preset_short_name(preset: AestheticPreset) -> &'static str {
    match preset {
        AestheticPreset::PunkRiot => "Punk Riot",
        AestheticPreset::NavalhaSata => "Navalha Satã",
        AestheticPreset::XeroxPodre => "Xerox Podre",
        AestheticPreset::ViagemSolitaria => "Viagem Solitária",
        AestheticPreset::LapaNoturna => "Lapa Noturna",
        AestheticPreset::TeatroOperario => "Teatro Operário",
        AestheticPreset::Risografia => "Risografia",
        AestheticPreset::FanzineDigital => "Fanzine Digital",
        AestheticPreset::AsfaltoAlerta => "Asfalto & Ouro",
        AestheticPreset::Vga256 => "VGA 256",
        AestheticPreset::Cga => "CGA 1981",
        AestheticPreset::OldMacPopArt => "Old Mac Pop Art",
        AestheticPreset::AsciiArt => "ASCII Art",
        AestheticPreset::TabloidePopular => "Tablóide Popular",
        AestheticPreset::Brutalismo => "Brutalismo",
        AestheticPreset::TrashPolka => "Trash Polka",
        AestheticPreset::AtariGaming => "Atari Gaming",
        AestheticPreset::Mimeografo => "Mimeógrafo",
        AestheticPreset::Xilogravura => "Xilogravura",
        AestheticPreset::GlitchCrt => "Glitch CRT",
        AestheticPreset::Punk80Br => "Punk 80 BR",
        AestheticPreset::AnarcoZine => "Anarco-Zine",
    }
}

#[component]
pub fn App() -> Element {
    // Imagem base decodificada
    let raw_image = use_signal(|| create_default_graphic(360, 480));
    let current_bg_subject = use_signal(String::new);


    // Carregamento aleatório inicial a partir do acervo CC de 20 retratos históricos
    use_hook(|| {
        let mut raw_setter = raw_image;
        let mut subject_setter = current_bg_subject;
        spawn(async move {
            let idx = pick_random_background_index();
            let (path, name) = BACKGROUND_COLLECTION[idx];
            if let Some(img) = load_remote_image(path).await {
                raw_setter.set(img);
                subject_setter.set(name.to_string());
            }
        });
    });

    // Abas do painel de controle ("poster", "intervencao", "ajustes")
    let mut active_tab = use_signal(|| String::from("poster"));

    // Alternador de visão mobile ("controls", "preview") e colapso de preview
    let mut mobile_view = use_signal(|| String::from("controls"));
    let mut preview_collapsed = use_signal(|| false);

    // Controles principais do Pôster Relâmpago
    // Controles principais do Pôster Relâmpago
    let mut phrase = use_signal(|| String::from("EU SOBREVIVO AO MUNDO"));
    let mut phrase_top = use_signal(|| String::new());
    let mut aesthetic_id = use_signal(|| String::from("punk-riot"));

    // Aba ativa de faixa ("bottom" = Inferior, "top" = Superior - Ticket #16)
    let mut banner_active_tab = use_signal(|| String::from("bottom"));

    // Faixa Inferior (Manchete principal da base)
    let mut banner_font_family = use_signal(|| String::from("impact"));
    let mut banner_font_size = use_signal(|| 48u32);
    let mut banner_auto_fit = use_signal(|| true);
    let mut banner_pos_x = use_signal(|| 50u32); // 50%
    let mut banner_pos_y = use_signal(|| 85u32); // 85%
    let mut banner_rotation = use_signal(|| 0i16); // 0 graus
    let mut banner_fg_color = use_signal(|| String::from("auto"));
    let mut banner_bg_color = use_signal(|| String::from("auto"));
    let mut banner_border_finish = use_signal(|| String::from("nenhuma"));
    let mut banner_border_width = use_signal(|| 3u32);
    let banner_border_color = use_signal(|| String::from("auto"));
    let mut banner_effect = use_signal(|| String::from("solida"));
    let mut banner_wave_amp = use_signal(|| 0u32);
    let mut banner_word_rot = use_signal(|| 0u32);
    let mut banner_word_cut = use_signal(|| false);
    let mut banner_ransom_note = use_signal(|| false); // Ticket #18
    let mut banner_adapt_preset = use_signal(|| true);

    // Faixa Superior (Manchete do topo - Tickets #15, #16 e #18)
    let mut banner_top_font_family = use_signal(|| String::from("impact"));
    let mut banner_top_font_size = use_signal(|| 48u32);
    let mut banner_top_auto_fit = use_signal(|| true);
    let mut banner_top_pos_x = use_signal(|| 50u32); // 50%
    let mut banner_top_pos_y = use_signal(|| 10u32); // 10% (Inicia no topo!)
    let mut banner_top_rotation = use_signal(|| 0i16); // 0 graus
    let mut banner_top_fg_color = use_signal(|| String::from("auto"));
    let mut banner_top_bg_color = use_signal(|| String::from("auto"));
    let mut banner_top_border_finish = use_signal(|| String::from("nenhuma"));
    let mut banner_top_border_width = use_signal(|| 3u32);
    let banner_top_border_color = use_signal(|| String::from("auto"));
    let mut banner_top_effect = use_signal(|| String::from("solida"));
    let mut banner_top_wave_amp = use_signal(|| 0u32);
    let mut banner_top_word_rot = use_signal(|| 0u32);
    let mut banner_top_word_cut = use_signal(|| false);
    let mut banner_top_ransom_note = use_signal(|| false);
    let mut banner_top_adapt_preset = use_signal(|| true);

    let mut banner_sem_faixa = use_signal(|| false);

    // Controles de edição de recorte (guilhotina) a partir do preset ativo
    let mut recorte_adapt_preset = use_signal(|| true);
    let mut recorte_ativo = use_signal(|| true);
    let mut recorte_threshold = use_signal(|| 85u32); // 85%
    let mut recorte_inverter = use_signal(|| true);

    // Painel de ajustes avançados (acordeão)
    let mut show_advanced = use_signal(|| false);
    let mut dither_override = use_signal(|| String::from("auto"));
    let mut palette_override = use_signal(|| String::from("auto"));
    let mut threshold_slider = use_signal(|| 85u32); // 85%
    let mut unsharp_slider = use_signal(|| 22u32);   // 2.2x
    let mut stretch_y_slider = use_signal(|| 15u32); // 1.5x
    let mut scale_select = use_signal(|| String::from("2x"));
    let mut rotation_deg = use_signal(|| 0u32);
    let mut image_zoom = use_signal(|| 100u32); // percentual: 100 = 1.0x
    let mut image_pos_x = use_signal(|| 50u32); // percentual: 50 = centro (0 a 100)
    let mut image_pos_y = use_signal(|| 50u32); // percentual: 50 = centro (0 a 100)
    let mut image_crop_top = use_signal(|| 0u32);    // percentual de corte 0 a 45% (Ticket #17)
    let mut image_crop_bottom = use_signal(|| 0u32); // percentual de corte 0 a 45%
    let mut image_crop_left = use_signal(|| 0u32);   // percentual de corte 0 a 45%
    let mut image_crop_right = use_signal(|| 0u32);  // percentual de corte 0 a 45%
    let mut auto_threshold = use_signal(|| false);
    let mut active_damages = use_signal(|| Vec::<String>::new());
    let mut active_stickers = use_signal(|| Vec::<String>::new());
    let mut filter_stickers = use_signal(|| false);

    // Caixas de texto extras arbitrárias — (texto, pos_x*100, pos_y*100, tamanho, inclinacao, acabamento)
    // Acabamentos: "tarja", "tape", "serrilhado", "rasgo"
    let mut extra_texts: Signal<Vec<(String, u32, u32, u32, i32, String)>> = use_signal(|| Vec::new());
    // Estado do formulário de nova caixa de texto
    let mut form_texto   = use_signal(|| String::new());
    let mut form_pos_x   = use_signal(|| 50u32);
    let mut form_pos_y   = use_signal(|| 50u32);
    let mut form_tamanho = use_signal(|| 48u32);
    let mut form_incl    = use_signal(|| 0i32);
    let mut form_acabamento = use_signal(|| String::from("tarja"));
    let mut form_editando   = use_signal(|| Option::<usize>::None); // índice em edição

    // Parâmetros individuais de danos: mapa tipo -> (posX*100, posY*100, escala*100)
    // Posição em % inteiro (0–100), escala em % inteiro (10–500, padrão 100 = 1.0x)
    use std::collections::HashMap;
    let mut damage_params = use_signal(|| HashMap::<String, (u32, u32, u32)>::new());
    // Dano selecionado no subpainel de ajuste
    let mut selected_damage = use_signal(|| Option::<String>::None);

    // Parâmetros individuais de stickers: mapa tipo -> (posX*100, posY*100, tamanho_px, rotacao)
    let mut sticker_params = use_signal(|| HashMap::<String, (u32, u32, u32, i32)>::new());
    // Bordas individuais de stickers: mapa tipo -> (estilo_str, largura_px)
    let mut sticker_borders = use_signal(|| HashMap::<String, (String, u32)>::new());
    // Imagem personalizada carregada para sticker/recorte próprio
    let custom_sticker_image = use_signal(|| Option::<std::sync::Arc<RgbaImage>>::None);
    // Sticker selecionado no subpainel de ajuste
    let mut selected_sticker = use_signal(|| Option::<String>::None);

    // Feedback de cópia
    let mut copy_status = use_signal(|| String::from("COPIAR COMANDO"));

    // Sinais para retenção do cartaz renderizado e status de debounce/processamento
    let mut rendered_png_uri = use_signal(|| String::new());
    let mut rendered_svg = use_signal(|| String::new());
    let mut latest_poster = use_signal(|| None::<RgbaImage>);
    let mut is_processing = use_signal(|| false);
    let render_seq = use_hook(|| std::rc::Rc::new(std::cell::Cell::new(0u64)));
    let subject_cache = use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(navalha_core::SubjectLayerCache::new())));

    // Efeito reativo com debounce para reprocessamento do cartaz
    {
        let seq_ref = render_seq.clone();
        let cache_ref = subject_cache.clone();
        use_effect(move || {
            // Rastreamento explícito de todos os sinais que alteram a estética do pôster
            let raw = raw_image();
            let phr = phrase();
            let phr_top = phrase_top();
            let aes = aesthetic_id();
            let adv = show_advanced();
            let d_override = dither_override();
            let p_override = palette_override();
            let th_slider = threshold_slider();
            let un_slider = unsharp_slider();
            let st_slider = stretch_y_slider();
            let sc_sel = scale_select();
            let rot_deg = rotation_deg();
            let zoom = image_zoom();
            let pos_x = image_pos_x();
            let pos_y = image_pos_y();
            let auto_th = auto_threshold();
            let damages = active_damages();
            let d_params = damage_params();
            let stickers = active_stickers();
            let s_params = sticker_params();
            let s_borders = sticker_borders();
            let custom_stk_img = custom_sticker_image();
            let f_stickers = filter_stickers();
            let extras = extra_texts();
            let f_txt = form_texto();
            let f_px = form_pos_x();
            let f_py = form_pos_y();
            let f_tam = form_tamanho();
            let f_inc = form_incl();
            let f_acab = form_acabamento();
            let f_edit = form_editando();
            let b_fam = banner_font_family();
            let b_sz = banner_font_size();
            let b_af = banner_auto_fit();
            let b_px = banner_pos_x();
            let b_py = banner_pos_y();
            let b_rot = banner_rotation();
            let b_fg = banner_fg_color();
            let b_bg = banner_bg_color();
            let b_bfinish = banner_border_finish();
            let b_bwidth = banner_border_width();
            let b_bcol = banner_border_color();
            let b_eff = banner_effect();
            let b_wamp = banner_wave_amp();
            let b_wrot = banner_word_rot();
            let b_wcut = banner_word_cut();
            let b_ransom = banner_ransom_note();
            let b_adapt = banner_adapt_preset();
            let b_sem_faixa = banner_sem_faixa();

            let b_top_fam = banner_top_font_family();
            let b_top_sz = banner_top_font_size();
            let b_top_af = banner_top_auto_fit();
            let b_top_px = banner_top_pos_x();
            let b_top_py = banner_top_pos_y();
            let b_top_rot = banner_top_rotation();
            let b_top_fg = banner_top_fg_color();
            let b_top_bg = banner_top_bg_color();
            let b_top_bfinish = banner_top_border_finish();
            let b_top_bwidth = banner_top_border_width();
            let b_top_bcol = banner_top_border_color();
            let b_top_eff = banner_top_effect();
            let b_top_wamp = banner_top_wave_amp();
            let b_top_wrot = banner_top_word_rot();
            let b_top_wcut = banner_top_word_cut();
            let b_top_ransom = banner_top_ransom_note();
            let b_top_adapt = banner_top_adapt_preset();

            let c_top = image_crop_top();
            let c_bot = image_crop_bottom();
            let c_left = image_crop_left();
            let c_right = image_crop_right();

            let r_adapt = recorte_adapt_preset();
            let r_ativo = recorte_ativo();
            let r_th = recorte_threshold();
            let r_inv = recorte_inverter();

            let my_seq = seq_ref.get() + 1;
            seq_ref.set(my_seq);

            let checker = seq_ref.clone();
            let task_cache = cache_ref.clone();
            spawn(async move {
                // Debounce de 250ms após interações do usuário (imediato apenas na carga inicial)
                if my_seq > 1 {
                    TimeoutFuture::new(250).await;
                }

                // Se uma nova interação ocorreu durante a janela de inatividade, descarta esta tarefa obsoleta
                if checker.get() != my_seq {
                    return;
                }

                is_processing.set(true);

                // Libera um micro-tick para permitir que a interface pinte o indicador de carregamento
                TimeoutFuture::new(0).await;
                if checker.get() != my_seq {
                    return;
                }

                let current_preset = AestheticPreset::parse(&aes).unwrap_or(AestheticPreset::PunkRiot);
                let (mut g, mut pr, mut dt, mut pal, mut sc, mut comp) =
                    current_preset.build_pipeline_and_composition_dual(&phr, if phr_top.trim().is_empty() { None } else { Some(&phr_top) });

                if !r_adapt {
                    g.enabled = r_ativo;
                    g.threshold = (r_th as f32) / 100.0;
                    g.invert = r_inv;
                }

                pr.rotation = rot_deg;
                pr.zoom = (zoom as f32) / 100.0;
                pr.crop_top = c_top as f32;
                pr.crop_bottom = c_bot as f32;
                pr.crop_left = c_left as f32;
                pr.crop_right = c_right as f32;
                comp.subject_offset_x = ((pos_x as i32 - 50) * 800) / 100;
                comp.subject_offset_y = ((pos_y as i32 - 50) * 1100) / 100;
                if auto_th {
                    g.auto_threshold = true;
                }

                // Semente determinística fixada na foto original bruta (Ticket #19)
                comp.custom_seed = Some(navalha_core::compute_subject_seed(Some(&raw), &phr));

                // Injeta danos com posição e escala customizáveis
                for d_str in damages.iter() {
                    if let Some(kind) = DamageEffect::parse(d_str) {
                        let ja_existe = comp.overlay.damage_elements.iter().any(|e| e.kind == kind);
                        if !ja_existe {
                            let params = d_params.get(d_str).copied();
                            let elem = if let Some((px100, py100, sc100)) = params {
                                DamageElement::new(
                                    kind,
                                    (px100 as f32) / 100.0,
                                    (py100 as f32) / 100.0,
                                    (sc100 as f32) / 100.0,
                                )
                            } else {
                                DamageElement::default_for(kind)
                            };
                            comp.overlay.damage_elements.push(elem);
                        }
                    }
                }

                // Posições automáticas para stickers sem parâmetros customizados
                let sticker_auto_positions: &[(f32, f32, i16)] = &[
                    (0.84, 0.16, -10),
                    (0.16, 0.82, 12),
                    (0.16, 0.18, -6),
                    (0.84, 0.78, 8),
                    (0.85, 0.48, -12),
                    (0.15, 0.48, 14),
                ];
                for (idx, s_str) in stickers.iter().enumerate() {
                    if let Some(kind) = StickerKind::parse(s_str) {
                        let params = s_params.get(s_str).copied();
                        let (px, py, size, rot) = if let Some((px100, py100, sz, rot100)) = params {
                            ((px100 as f32) / 100.0, (py100 as f32) / 100.0, sz, rot100 as i16)
                        } else {
                            let (apx, apy, arot) = sticker_auto_positions[idx % sticker_auto_positions.len()];
                            (apx, apy, 100, arot)
                        };
                        let (b_style, b_width) = s_borders.get(s_str).cloned().unwrap_or_else(|| (String::from("diecut"), 4u32));
                        let border = StickerBorder::parse(&b_style).unwrap_or(StickerBorder::DieCutBranco);
                        let el = if kind == StickerKind::Custom {
                            if let Some(ref c_img) = custom_stk_img {
                                StickerElement::new_custom(c_img.clone(), px, py, size, rot, f_stickers)
                            } else {
                                StickerElement::new(kind, px, py, size, rot, f_stickers)
                            }
                        } else {
                            StickerElement::new(kind, px, py, size, rot, f_stickers)
                        };
                        comp.stickers.push(el.with_border(border, b_width));
                    }
                }

                // Injeta caixas de texto extras salvas (pulando a que estiver atualmente em edição)
                for (idx, (txt, px100, py100, tam, incl, acab)) in extras.iter().enumerate() {
                    if f_edit == Some(idx) {
                        continue;
                    }
                    use navalha_core::{RgbColor, TextElement};
                    let pos_x = (*px100 as f32) / 100.0;
                    let pos_y = (*py100 as f32) / 100.0;
                    let (cor_texto, bg_tarja, padding, tarja_effect) = match acab.as_str() {
                        "xerox"      => (RgbColor::new(245, 242, 235), Some(RgbColor::new(28, 25, 23)), 14u32, TarjaEffect::Xerox),
                        "tape"       => (RgbColor::new(20, 20, 20),   Some(RgbColor::new(200, 200, 180)), 6u32, TarjaEffect::Solida),
                        "serrilhado" => (RgbColor::new(0, 0, 0),      Some(RgbColor::new(255, 230, 0)),   10u32, TarjaEffect::Solida),
                        "rasgo"      => (RgbColor::new(200, 0, 0),    None,                               0u32,  TarjaEffect::Solida),
                        _            => (RgbColor::new(255, 255, 255),Some(RgbColor::new(0, 0, 0)),      10u32, TarjaEffect::Solida),
                    };
                    comp.foreground_texts.push(TextElement {
                        text: txt.to_uppercase(),
                        font_size: *tam as f32,
                        stretch_x: 1.0,
                        stretch_y: 1.4,
                        color: cor_texto,
                        bg_tarja,
                        tarja_padding: padding,
                        pos_x,
                        pos_y,
                        center_x: true,
                        rotation: *incl as i16,
                        tarja_effect,
                        ..Default::default()
                    });
                }

                // Preview em tempo real da caixa de texto digitada/editada no formulário
                let form_txt_up = f_txt.trim().to_uppercase();
                if !form_txt_up.is_empty() {
                    use navalha_core::{RgbColor, TextElement};
                    let pos_x = (f_px as f32) / 100.0;
                    let pos_y = (f_py as f32) / 100.0;
                    let (cor_texto, bg_tarja, padding, tarja_effect) = match f_acab.as_str() {
                        "xerox"      => (RgbColor::new(245, 242, 235), Some(RgbColor::new(28, 25, 23)), 14u32, TarjaEffect::Xerox),
                        "tape"       => (RgbColor::new(20, 20, 20),   Some(RgbColor::new(200, 200, 180)), 6u32, TarjaEffect::Solida),
                        "serrilhado" => (RgbColor::new(0, 0, 0),      Some(RgbColor::new(255, 230, 0)),   10u32, TarjaEffect::Solida),
                        "rasgo"      => (RgbColor::new(200, 0, 0),    None,                               0u32,  TarjaEffect::Solida),
                        _            => (RgbColor::new(255, 255, 255),Some(RgbColor::new(0, 0, 0)),      10u32, TarjaEffect::Solida),
                    };
                    comp.foreground_texts.push(TextElement {
                        text: form_txt_up,
                        font_size: f_tam as f32,
                        stretch_x: 1.0,
                        stretch_y: 1.4,
                        color: cor_texto,
                        bg_tarja,
                        tarja_padding: padding,
                        pos_x,
                        pos_y,
                        center_x: true,
                        rotation: f_inc as i16,
                        tarja_effect,
                        ..Default::default()
                    });
                }

                let has_top = !phr_top.trim().is_empty();
                let has_bottom = !phr.trim().is_empty();

                let top_idx = if has_top { Some(0) } else { None };
                let bottom_idx = match (has_top, has_bottom) {
                    (true, true) => Some(1),
                    (false, true) => Some(0),
                    _ => None,
                };

                // Aplica estilização da Faixa Superior (Tickets #15, #16 e #18)
                if let Some(t_idx) = top_idx {
                    if let Some(t) = comp.foreground_texts.get_mut(t_idx) {
                        t.ransom_note = b_top_ransom;
                        if b_top_adapt {
                            let def_banner = current_preset.default_banner_element(&phr_top);
                            t.font_family = def_banner.font_family;
                            t.auto_fit = b_top_af;
                            t.pos_x = (b_top_px as f32) / 100.0;
                            t.pos_y = (b_top_py as f32) / 100.0;
                            t.rotation = b_top_rot;
                            t.color = def_banner.color;
                            t.bg_tarja = def_banner.bg_tarja;
                            t.border_finish = def_banner.border_finish;
                            t.border_width = def_banner.border_width;
                            t.border_color = def_banner.border_color;
                            t.tarja_effect = def_banner.tarja_effect;
                            t.word_jitter_rotation = def_banner.word_jitter_rotation;
                            t.word_tarja_cut = def_banner.word_tarja_cut;
                            t.wave_amplitude = def_banner.wave_amplitude;
                            t.wave_wavelength = def_banner.wave_wavelength;
                        } else {
                            let parsed_fam = navalha_core::BannerFontFamily::parse(&b_top_fam).unwrap_or(navalha_core::BannerFontFamily::Impact);
                            t.font_family = parsed_fam;
                            t.auto_fit = b_top_af;
                            t.font_size = b_top_sz as f32;
                            t.pos_x = (b_top_px as f32) / 100.0;
                            t.pos_y = (b_top_py as f32) / 100.0;
                            t.rotation = b_top_rot;

                            if b_top_fg != "auto" {
                                if let Some(col) = navalha_core::RgbColor::parse_color_str(&b_top_fg) {
                                    t.color = col;
                                }
                            }
                            if b_top_bg != "auto" {
                                if b_top_bg == "none" || b_top_bg == "sem" {
                                    t.bg_tarja = None;
                                } else if let Some(col) = navalha_core::RgbColor::parse_color_str(&b_top_bg) {
                                    t.bg_tarja = Some(col);
                                }
                            }
                            if let Some(finish) = navalha_core::TarjaBorderFinish::parse(&b_top_bfinish) {
                                t.border_finish = finish;
                            }
                            t.border_width = b_top_bwidth;
                            if b_top_bcol != "auto" {
                                if let Some(col) = navalha_core::RgbColor::parse_color_str(&b_top_bcol) {
                                    t.border_color = Some(col);
                                }
                            }
                            if let Some(ef) = navalha_core::TarjaEffect::parse(&b_top_eff) {
                                t.tarja_effect = ef;
                            }
                            t.wave_amplitude = b_top_wamp as f32;
                            t.word_jitter_rotation = b_top_wrot as i16;
                            t.word_tarja_cut = b_top_wcut || (b_top_wrot > 0);
                        }
                    }
                }

                // Aplica estilização da Faixa Inferior (Tickets #15, #16 e #18)
                if let Some(b_idx) = bottom_idx {
                    if let Some(t) = comp.foreground_texts.get_mut(b_idx) {
                        t.ransom_note = b_ransom;
                        if b_adapt {
                            let def_banner = current_preset.default_banner_element(&phr);
                            t.font_family = def_banner.font_family;
                            t.auto_fit = b_af;
                            t.pos_x = (b_px as f32) / 100.0;
                            t.pos_y = (b_py as f32) / 100.0;
                            t.rotation = b_rot;
                            t.color = def_banner.color;
                            t.bg_tarja = def_banner.bg_tarja;
                            t.border_finish = def_banner.border_finish;
                            t.border_width = def_banner.border_width;
                            t.border_color = def_banner.border_color;
                            t.tarja_effect = def_banner.tarja_effect;
                            t.word_jitter_rotation = def_banner.word_jitter_rotation;
                            t.word_tarja_cut = def_banner.word_tarja_cut;
                            t.wave_amplitude = def_banner.wave_amplitude;
                            t.wave_wavelength = def_banner.wave_wavelength;
                        } else {
                            let parsed_fam = navalha_core::BannerFontFamily::parse(&b_fam).unwrap_or(navalha_core::BannerFontFamily::Impact);
                            t.font_family = parsed_fam;
                            t.auto_fit = b_af;
                            t.font_size = b_sz as f32;
                            t.pos_x = (b_px as f32) / 100.0;
                            t.pos_y = (b_py as f32) / 100.0;
                            t.rotation = b_rot;

                            if b_fg != "auto" {
                                if let Some(col) = navalha_core::RgbColor::parse_color_str(&b_fg) {
                                    t.color = col;
                                }
                            }
                            if b_bg != "auto" {
                                if b_bg == "none" || b_bg == "sem" {
                                    t.bg_tarja = None;
                                } else if let Some(col) = navalha_core::RgbColor::parse_color_str(&b_bg) {
                                    t.bg_tarja = Some(col);
                                }
                            }
                            if let Some(finish) = navalha_core::TarjaBorderFinish::parse(&b_bfinish) {
                                t.border_finish = finish;
                            }
                            t.border_width = b_bwidth;
                            if b_bcol != "auto" {
                                if let Some(col) = navalha_core::RgbColor::parse_color_str(&b_bcol) {
                                    t.border_color = Some(col);
                                }
                            }
                            if let Some(ef) = navalha_core::TarjaEffect::parse(&b_eff) {
                                t.tarja_effect = ef;
                            }
                            t.wave_amplitude = b_wamp as f32;
                            t.word_jitter_rotation = b_wrot as i16;
                            t.word_tarja_cut = b_wcut || (b_wrot > 0);
                        }
                    }
                }

                // Aplica overrides caso o usuário tenha ajustado os controles finos
                if adv {
                    g.threshold = (th_slider as f32) / 100.0;
                    pr.unsharp_strength = (un_slider as f32) / 10.0;
                    let sy = (st_slider as f32) / 10.0;

                    for t in comp.foreground_texts.iter_mut() {
                        t.stretch_y = sy;
                    }

                    if d_override != "auto" {
                        if let Some(d) = DitherAlgorithm::parse(&d_override) {
                            dt = d;
                        }
                    }

                    if p_override != "auto" {
                        if let Some(p) = Palette::find_preset(&p_override) {
                            pal = p;
                        }
                    }

                    if let Some(s) = ScaleFactor::parse(&sc_sel) {
                        sc = s;
                    }
                }

                if b_sem_faixa {
                    comp.foreground_texts.clear();
                }

                if checker.get() != my_seq {
                    return;
                }

                let is_cancelled = {
                    let chk = checker.clone();
                    move || chk.get() != my_seq
                };

                let processed_opt = {
                    let mut cache = task_cache.borrow_mut();
                    cache.get_or_process_cancellable(&raw, &g, &pr, dt, &pal, sc, &is_cancelled)
                };

                let processed = match processed_opt {
                    Some(p) => p,
                    None => return, // Processamento cancelado ou tornado obsoleto por nova interação
                };

                if checker.get() != my_seq {
                    return;
                }

                let poster_opt = navalha_core::render_poster_cancellable(&comp, Some(&processed), &is_cancelled);
                let poster = match poster_opt {
                    Some(p) => p,
                    None => return,
                };

                if checker.get() != my_seq {
                    return;
                }

                // Converte pôster para data URI PNG
                let mut png_bytes = Vec::new();
                let _ = poster.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png);
                let png_b64 = BASE64.encode(&png_bytes);
                let png_data_uri = format!("data:image/png;base64,{png_b64}");

                // Gera SVG
                let svg_string = generate_poster_svg(&comp, Some(&png_b64));

                // Atualiza os sinais se nenhuma outra alteração ocorreu no meio do caminho
                if checker.get() == my_seq {
                    rendered_png_uri.set(png_data_uri);
                    rendered_svg.set(svg_string);
                    latest_poster.set(Some(poster));
                    is_processing.set(false);
                }
            });
        });
    }

    let dither_str = dither_override();
    let palette_str = palette_override();
    let dither_label = if dither_str == "auto" { "atkinson" } else { &dither_str };
    let palette_label = if palette_str == "auto" { "classic-punk" } else { &palette_str };

    // Gera espelho da sintaxe CLI em tempo real
    let dano_flag = if active_damages().is_empty() {
        String::new()
    } else {
        let specs: Vec<String> = active_damages().iter().map(|d_str| {
            if let Some(&(px100, py100, sc100)) = damage_params().get(d_str.as_str()) {
                let default_elem = DamageEffect::parse(d_str).map(|k| DamageElement::default_for(k));
                let is_default = default_elem.map(|def| {
                    let dpx = (def.pos_x * 100.0).round() as u32;
                    let dpy = (def.pos_y * 100.0).round() as u32;
                    px100 == dpx && py100 == dpy && sc100 == 100
                }).unwrap_or(true);
                if is_default {
                    d_str.clone()
                } else {
                    format!("{}:{:.2}:{:.2}:{:.2}", d_str,
                        (px100 as f32) / 100.0,
                        (py100 as f32) / 100.0,
                        (sc100 as f32) / 100.0)
                }
            } else {
                d_str.clone()
            }
        }).collect();
        format!(" --dano {}", specs.join(","))
    };

    let sticker_flag = if active_stickers().is_empty() {
        String::new()
    } else {
        let sticker_auto_positions: &[(f32, f32, i16)] = &[
            (0.84, 0.16, -10), (0.16, 0.82, 12), (0.16, 0.18, -6),
            (0.84, 0.78, 8), (0.85, 0.48, -12), (0.15, 0.48, 14),
        ];
        let s_borders_map = sticker_borders();
        let specs: Vec<String> = active_stickers().iter().enumerate().map(|(idx, s_str)| {
            let (b_style, b_width) = s_borders_map.get(s_str.as_str()).cloned().unwrap_or_else(|| (String::from("diecut"), 4u32));
            let has_custom_border = b_style != "diecut" || b_width != 4;
            if let Some(&(px100, py100, sz, rot)) = sticker_params().get(s_str.as_str()) {
                let (apx, apy, arot) = sticker_auto_positions[idx % sticker_auto_positions.len()];
                let is_default = px100 == (apx * 100.0).round() as u32
                    && py100 == (apy * 100.0).round() as u32
                    && sz == 100 && rot == arot as i32;
                if is_default && !has_custom_border {
                    s_str.clone()
                } else if has_custom_border {
                    format!("{}:{:.2}:{:.2}:{}:{}:{}:{}", s_str,
                        (px100 as f32) / 100.0,
                        (py100 as f32) / 100.0,
                        sz, rot, b_style, b_width)
                } else {
                    format!("{}:{:.2}:{:.2}:{}:{}", s_str,
                        (px100 as f32) / 100.0,
                        (py100 as f32) / 100.0,
                        sz, rot)
                }
            } else if has_custom_border {
                let (apx, apy, arot) = sticker_auto_positions[idx % sticker_auto_positions.len()];
                format!("{}:{:.2}:{:.2}:100:{}:{}:{}", s_str, apx, apy, arot, b_style, b_width)
            } else {
                s_str.clone()
            }
        }).collect();
        let filter_opt = if filter_stickers() { " --filtrar-stickers" } else { "" };
        format!(" --sticker {}{}", specs.join(","), filter_opt)
    };

    let texto_extra_flag = extra_texts().iter().map(|(txt, px, py, tam, incl, acab)| {
        format!(" --texto-extra \"{}:{:.2}:{:.2}:{}:{}:{}\"",
            txt,
            (*px as f32) / 100.0,
            (*py as f32) / 100.0,
            tam,
            incl,
            acab
        )
    }).collect::<String>();

    let rotacao_flag = if rotation_deg() > 0 {
        format!(" --rotacao {}", rotation_deg())
    } else {
        String::new()
    };

    let zoom_flag = if image_zoom() != 100 {
        format!(" --zoom {:.2}", (image_zoom() as f32) / 100.0)
    } else {
        String::new()
    };

    let offset_x = ((image_pos_x() as i32 - 50) * 800) / 100;
    let offset_y = ((image_pos_y() as i32 - 50) * 1100) / 100;
    let offset_flag = if offset_x != 0 || offset_y != 0 {
        format!(" --offset-x {} --offset-y {}", offset_x, offset_y)
    } else {
        String::new()
    };

    let frase_superior_flag = if !phrase_top().trim().is_empty() {
        format!(" --frase-superior \"{}\"", phrase_top())
    } else {
        String::new()
    };

    let texto_topo_flag = if !phrase_top().trim().is_empty() {
        format!(" --texto-topo \"{}\"", phrase_top().to_uppercase())
    } else {
        String::new()
    };

    let banner_style_flags = {
        let mut flags = String::new();
        if banner_pos_x() != 50 {
            flags.push_str(&format!(" --banner-x {:.2}", (banner_pos_x() as f32) / 100.0));
        }
        if banner_pos_y() != 85 {
            flags.push_str(&format!(" --banner-y {:.2}", (banner_pos_y() as f32) / 100.0));
        }
        if banner_rotation() != 0 {
            flags.push_str(&format!(" --banner-rot {}", banner_rotation()));
        }
        if banner_fg_color() != "auto" {
            flags.push_str(&format!(" --banner-fg \"{}\"", banner_fg_color()));
        }
        if banner_bg_color() != "auto" {
            flags.push_str(&format!(" --banner-bg \"{}\"", banner_bg_color()));
        }
        if banner_border_finish() != "nenhuma" {
            flags.push_str(&format!(" --banner-borda \"{}\"", banner_border_finish()));
            if banner_border_width() != 3 {
                flags.push_str(&format!(" --banner-borda-largura {}", banner_border_width()));
            }
        }
        if banner_effect() != "solida" {
            flags.push_str(&format!(" --banner-efeito \"{}\"", banner_effect()));
        }
        if banner_wave_amp() > 0 {
            flags.push_str(&format!(" --onda-amp {}", banner_wave_amp()));
        }
        if banner_word_rot() > 0 {
            flags.push_str(&format!(" --palavra-rot {}", banner_word_rot()));
        }
        flags
    };

    let sem_faixa_flag = if banner_sem_faixa() { " --sem-faixa" } else { "" };
    let recorte_flags = if !recorte_adapt_preset() {
        let mut rf = String::new();
        if !recorte_ativo() {
            rf.push_str(" --sem-recorte");
        } else {
            rf.push_str(&format!(" --recorte-threshold {:.2}", (recorte_threshold() as f32) / 100.0));
            if recorte_inverter() {
                rf.push_str(" --recorte-inverter");
            }
        }
        rf
    } else {
        String::new()
    };

    let cli_cmd = if !show_advanced() {
        format!(
            "navalha poster foto.jpg --frase \"{}\"{}{}{}{}{}{}{}{}{}{}{} -o poster.png",
            phrase(),
            frase_superior_flag,
            sem_faixa_flag,
            recorte_flags,
            banner_style_flags,
            aesthetic_id(),
            rotacao_flag,
            zoom_flag,
            offset_flag,
            dano_flag,
            sticker_flag,
            texto_extra_flag
        )
    } else {
        format!(
            "navalha compor foto.jpg --recorte --recorte-threshold {:.2} --dither {} --paleta {} --escala {}{}{}{}{} --tarja-baixo \"{}\" --distorcao-vertical {:.1}{}{}{} -o poster.png",
            (threshold_slider() as f32) / 100.0,
            dither_label,
            palette_label,
            scale_select(),
            rotacao_flag,
            zoom_flag,
            offset_flag,
            texto_topo_flag,
            phrase().to_uppercase(),
            (stretch_y_slider() as f32) / 10.0,
            dano_flag,
            sticker_flag,
            texto_extra_flag
        )
    };

    // Callback para upload de imagem
    let on_file_upload = move |_evt: Event<FormData>| {
        async move {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            if let Some(input) = document.get_element_by_id("image-uploader") {
                let input_el: HtmlInputElement = input.unchecked_into();
                if let Some(files) = input_el.files() {
                    if let Some(file) = files.get(0) {
                        let reader = web_sys::FileReader::new().unwrap();
                        let r_clone = reader.clone();
                        let mut raw_image_setter = raw_image;
                        let mut subject_setter = current_bg_subject;

                        let onload = Closure::wrap(Box::new(move |_: web_sys::Event| {
                            if let Ok(result) = r_clone.result() {
                                let array = js_sys::Uint8Array::new(&result);
                                let bytes = array.to_vec();
                                if let Ok(img_reader) = ImageReader::new(Cursor::new(bytes)).with_guessed_format() {
                                    if let Ok(decoded) = img_reader.decode() {
                                        raw_image_setter.set(decoded.to_rgba8());
                                        subject_setter.set(String::from("Foto do Usuário"));
                                    }
                                }
                            }
                        }) as Box<dyn FnMut(_)>);

                        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                        let _ = reader.read_as_array_buffer(&file);
                        onload.forget();
                    }
                }
            }
        }
    };

    // Callback para sortear outra imagem do acervo CC
    let on_random_bg = move |_| {
        let mut raw_setter = raw_image;
        let mut subject_setter = current_bg_subject;
        spawn(async move {
            let idx = pick_random_background_index();
            let (path, name) = BACKGROUND_COLLECTION[idx];
            if let Some(img) = load_remote_image(path).await {
                raw_setter.set(img);
                subject_setter.set(name.to_string());
            }
        });
    };


    let on_custom_sticker_upload = move |_evt: Event<FormData>| {
        async move {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            if let Some(input) = document.get_element_by_id("custom-sticker-uploader") {
                let input_el: HtmlInputElement = input.unchecked_into();
                if let Some(files) = input_el.files() {
                    if let Some(file) = files.get(0) {
                        let reader = web_sys::FileReader::new().unwrap();
                        let r_clone = reader.clone();
                        let mut custom_stk_setter = custom_sticker_image;
                        let mut active_stk_setter = active_stickers;
                        let mut sel_stk_setter = selected_sticker;

                        let onload = Closure::wrap(Box::new(move |_: web_sys::Event| {
                            if let Ok(result) = r_clone.result() {
                                let array = js_sys::Uint8Array::new(&result);
                                let bytes = array.to_vec();
                                if let Ok(img_reader) = ImageReader::new(Cursor::new(bytes)).with_guessed_format() {
                                    if let Ok(decoded) = img_reader.decode() {
                                        let arc_img = std::sync::Arc::new(decoded.to_rgba8());
                                        custom_stk_setter.set(Some(arc_img));
                                        let mut list = active_stk_setter();
                                        if !list.contains(&"custom".to_string()) {
                                            list.push("custom".to_string());
                                        }
                                        active_stk_setter.set(list);
                                        sel_stk_setter.set(Some("custom".to_string()));
                                    }
                                }
                            }
                        }) as Box<dyn FnMut(_)>);

                        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                        let _ = reader.read_as_array_buffer(&file);
                        onload.forget();
                    }
                }
            }
        }
    };

    // Callbacks de download PNG
    let download_png = move |_| {
        let uri = rendered_png_uri();
        if !uri.is_empty() {
            trigger_download(&uri, "navalha_poster.png");
        }
    };
    let download_png_dock = move |_| {
        let uri = rendered_png_uri();
        if !uri.is_empty() {
            trigger_download(&uri, "navalha_poster.png");
        }
    };
    let download_png_preview = move |_| {
        let uri = rendered_png_uri();
        if !uri.is_empty() {
            trigger_download(&uri, "navalha_poster.png");
        }
    };

    // Callback de download SVG
    let download_svg = move |_| {
        let svg_raw = rendered_svg();
        if !svg_raw.is_empty() {
            let encoded = format!("data:image/svg+xml;utf8,{}", js_sys::encode_uri_component(&svg_raw));
            trigger_download(&encoded, "navalha_poster.svg");
        }
    };

    // Callback de download de chapas para impressão (PDF)
    let download_pdf_chapas = move |_| {
        if let Some(poster) = latest_poster() {
            let config = PlateSeparationConfig::default();
            if let Ok(plates) = separate_plates(&poster, &config) {
                if let Ok(pdf_bytes) = generate_plates_pdf(&plates, Some("Navalha - Chapas Riso/Silk")) {
                    let pdf_b64 = BASE64.encode(&pdf_bytes);
                    let pdf_data_uri = format!("data:application/pdf;base64,{pdf_b64}");
                    trigger_download(&pdf_data_uri, "navalha_chapas.pdf");
                }
            }
        }
    };

    // Strings de display formatadas
    let unsharp_display = format!("{:.1}x", unsharp_slider() as f32 / 10.0);
    let stretch_y_display = format!("{:.1}x", stretch_y_slider() as f32 / 10.0);
    let image_zoom_display = format!("{:.2}x ({}%)", (image_zoom() as f32) / 100.0, image_zoom());

    let preset_summary = match aesthetic_id().as_str() {
        "punk-riot"        => "ATKINSON // 720PX // CLÁSSICA PUNK (PRETO/BRANCO/VERMELHO)",
        "navalha-sata"     => "ATKINSON // 720PX // LINÓLEO & OURO QUEIMADO",
        "xerox-podre"      => "BAYER 8x8 // 640PX // XEROX & PAPEL OXIDADO",
        "viagem-solitaria" => "FLOYD-STEINBERG // 640PX // PAPEL KRAFT & OLIVETTI",
        "lapa-noturna"     => "FLOYD-STEINBERG // 720PX // DUOTONE MAGENTA & CIANO",
        "teatro-operario"  => "BAYER 4x4 // 720PX // CONSTRUTIVISMO & CARMESIM",
        "risografia"       => "HALFTONE 45° // 720PX // SEPARAÇÃO MECÂNICA RISO",
        "fanzine-digital"  => "BAYER 8x8 // 640PX // CYBER-BBS ANOS 90 AZUL/AMARELO",
        "asfalto-alerta"   => "ATKINSON // 720PX // PRETO PICHE & AMARELO PERIGO",
        "vga256"           => "BAYER 4x4 // 320PX 13H // PALETA INDEXADA DOS",
        "cga"              => "BAYER 4x4 // 320PX 1981 // 4 CORES MC6845",
        "ascii-art"        => "RASTER ASCII REAL // ~82 COLS // FÓSFORO VERDE CRT",
        "tabloide-popular" => "FLOYD-STEINBERG // 720PX // ROTATIVA POPULAR NP",
        "brutalismo"       => "BINARIZAÇÃO GLOBAL // 720PX // CONCRETO & TARJAS MACIÇAS",
        "trash-polka"      => "FLOYD-STEINBERG // 720PX // GRAYSCALE, CARMIM & SPLATTER",
        "atari-gaming"     => "TIA STELLA NTSC // 160x192 // 128 CORES & SCANLINES",
        "mimeografo"       => "BAYER 4x4 // 640PX // VIOLETA ANILINA & SANGRAMENTO",
        "xilogravura"      => "SAUVOLA 15x15 // 720PX // TALHE EM MADEIRA & KRAFT",
        "glitch-crt"       => "SYNC JITTER // 640PX // SPLIT CROMÁTICO & V-HOLD",
        "punk-80-br"       => "ATKINSON // 720PX // ABC PAULISTA & FITAS K7",
        "anarco-zine"      => "HALFTONE 45° // 640PX // STENCIL & GRAMPOS FANZINE",
        _                  => "MOTOR ANTI-DESIGN // DETERMINÍSTICO // 800x1100",
    };

    // Callback para cópia do comando CLI
    let cmd_to_copy = cli_cmd.clone();
    let copy_cmd = move |_| {
        let window = web_sys::window().unwrap();
        let nav = window.navigator();
        let clipboard = nav.clipboard();
        let _ = clipboard.write_text(&cmd_to_copy);
        copy_status.set(String::from("✓ COPIADO!"));
    };

    rsx! {
        div { class: "navalha-app",
            // Barra de cabeçalho brutalista
            header { class: "navalha-header",
                div { class: "logo-box",
                    span { class: "logo-icon", "⚡🗡️" }
                    h1 { class: "logo-title", "NAVALHA" }
                    span { class: "logo-badge", "v0.1.0 • RUST 2024" }
                }
                div { class: "header-manifesto",
                    "ANTI-DESIGN DETERMINÍSTICO // ZERO IA GENERATIVA // PRIVACIDADE LOCAL 100%"
                }
                nav { class: "header-nav",
                    a {
                        href: "/doc",
                        class: "nav-link-doc",
                        title: "Manifesto Visual, história dos presets e documentação",
                        span { class: "nav-doc-desktop", "📖 DOCUMENTAÇÃO (/doc) →" }
                        span { class: "nav-doc-mobile", "📖 DOCS" }
                    }
                }
            }

            // Alternador de visão mobile (< 900px)
            div { class: "mobile-view-toggle",
                button {
                    class: if mobile_view() == "controls" { "mobile-toggle-btn active" } else { "mobile-toggle-btn" },
                    onclick: move |_| mobile_view.set("controls".into()),
                    "⚡ CONTROLES"
                }
                button {
                    class: if mobile_view() == "preview" { "mobile-toggle-btn active" } else { "mobile-toggle-btn" },
                    onclick: move |_| mobile_view.set("preview".into()),
                    "👁️ VER CARTAZ"
                }
            }

            // Área de trabalho dividida
            main { class: "navalha-workspace",
                // Painel de controles à esquerda
                section {
                    class: if mobile_view() == "preview" { "panel-controls mobile-hidden" } else { "panel-controls" },

                    // Ações de Download e Exportação fixas no topo
                    div { class: "export-box",
                        button {
                            class: "btn-download-primary",
                            onclick: download_png,
                            "💾 BAIXAR PÔSTER (PNG)"
                        }
                        button {
                            class: "btn-download-secondary",
                            onclick: download_svg,
                            "📐 EXPORTAR VETOR (SVG)"
                        }
                        button {
                            class: "btn-download-secondary",
                            onclick: download_pdf_chapas,
                            "🖨️ CHAPAS RISO/SILK (PDF)"
                        }
                    }

                    // Abas de controle tátil
                    div { class: "controls-tab-bar",
                        button {
                            class: if active_tab() == "poster" { "tab-btn active" } else { "tab-btn" },
                            onclick: move |_| active_tab.set("poster".into()),
                            "⚡ PÔSTER"
                        }
                        button {
                            class: if active_tab() == "intervencao" { "tab-btn active" } else { "tab-btn" },
                            onclick: move |_| active_tab.set("intervencao".into()),
                            "🎨 INTERVENÇÃO"
                        }
                        button {
                            class: if active_tab() == "ajustes" { "tab-btn active" } else { "tab-btn" },
                            onclick: move |_| active_tab.set("ajustes".into()),
                            "⚙️ AJUSTES"
                        }
                    }

                    // Conteúdo dinâmico da aba ativa
                    div { class: "controls-tab-body",
                        if active_tab() == "poster" {
                            div { class: "control-box",
                                label { class: "section-tag", "1. CARREGAR FOTO & EDIÇÃO DA CHAPA" }
                                input {
                                    r#type: "file",
                                    id: "image-uploader",
                                    accept: "image/*",
                                    class: "file-input-punk",
                                    onchange: on_file_upload,
                                }
                                div { class: "random-bg-row",
                                    button {
                                        class: "btn-random-bg",
                                        r#type: "button",
                                        onclick: on_random_bg,
                                        "🎲 IMAGEM ALEATÓRIA (ACERVO CC)"
                                    }
                                    if !current_bg_subject().is_empty() {
                                        span { class: "current-subject-badge",
                                            "✨ Sujeito: {current_bg_subject()}"
                                        }
                                    }

                                }
                                div { class: "img-edit-tools-row",

                                    button {
                                        class: "btn-edit-action",
                                        onclick: move |_| {
                                            let curr = rotation_deg();
                                            rotation_deg.set((curr + 90) % 360);
                                        },
                                        "↺ GIRAR 90° ({rotation_deg}°)"
                                    }
                                    button {
                                        class: if auto_threshold() { "btn-edit-action active" } else { "btn-edit-action" },
                                        onclick: move |_| {
                                            let curr = auto_threshold();
                                            auto_threshold.set(!curr);
                                        },
                                        if auto_threshold() { "✓ AUTO-THRESHOLD (OTSU)" } else { "🤖 AUTO-THRESHOLD (OTSU)" }
                                    }
                                }

                                // Bloco de Recorte de Silhueta / Guilhotina
                                div { class: "zoom-control-box", style: "margin-top: 10px;",
                                    div { class: "zoom-header-row",
                                        span { class: "zoom-title", "✂️ RECORTE DE SILHUETA (GUILHOTINA)" }
                                        button {
                                            class: if recorte_adapt_preset() { "btn-zoom-reset active" } else { "btn-zoom-reset" },
                                            onclick: move |_| {
                                                recorte_adapt_preset.set(true);
                                            },
                                            "↺ RECORTE DO PRESET"
                                        }
                                    }
                                    div { style: "display: flex; align-items: center; justify-content: space-between; margin-top: 8px;",
                                        div { style: "display: flex; align-items: center; gap: 6px;",
                                            input {
                                                r#type: "checkbox",
                                                id: "recorte-ativo-cb",
                                                checked: "{recorte_ativo}",
                                                onchange: move |e| {
                                                    recorte_adapt_preset.set(false);
                                                    recorte_ativo.set(e.checked());
                                                },
                                            }
                                            label { r#for: "recorte-ativo-cb", style: "font-size: 0.75rem; color: #fff; cursor: pointer;", "Ativar Guilhotina (Recorte)" }
                                        }
                                        div { style: "display: flex; align-items: center; gap: 6px;",
                                            input {
                                                r#type: "checkbox",
                                                id: "recorte-inverter-cb",
                                                checked: "{recorte_inverter}",
                                                onchange: move |e| {
                                                    recorte_adapt_preset.set(false);
                                                    recorte_inverter.set(e.checked());
                                                },
                                            }
                                            label { r#for: "recorte-inverter-cb", style: "font-size: 0.75rem; color: #aaa; cursor: pointer;", "Inverter Silhueta" }
                                        }
                                    }
                                    div { class: "stepper-slider-box", style: "margin-top: 8px;",
                                        div { class: "stepper-header-row",
                                            span { class: "stepper-title", "LIMIAR CORTE: {recorte_threshold}%" }
                                            button {
                                                r#type: "button",
                                                class: "btn-stepper-reset",
                                                onclick: move |_| {
                                                    recorte_adapt_preset.set(false);
                                                    recorte_threshold.set(85);
                                                },
                                                "↺ PADRÃO (85%)"
                                            }
                                        }
                                        div { class: "stepper-row",
                                            button {
                                                r#type: "button",
                                                class: "btn-stepper",
                                                onclick: move |_| {
                                                    let curr = recorte_threshold();
                                                    if curr > 10 {
                                                        recorte_adapt_preset.set(false);
                                                        recorte_threshold.set(curr.saturating_sub(5).max(10));
                                                    }
                                                },
                                                "−"
                                            }
                                            input {
                                                r#type: "range",
                                                min: "10",
                                                max: "98",
                                                step: "1",
                                                value: "{recorte_threshold}",
                                                class: "slider-punk-zoom",
                                                oninput: move |e| {
                                                    if let Ok(v) = e.value().parse::<u32>() {
                                                        recorte_adapt_preset.set(false);
                                                        recorte_threshold.set(v);
                                                    }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "btn-stepper",
                                                onclick: move |_| {
                                                    let curr = recorte_threshold();
                                                    if curr < 98 {
                                                        recorte_adapt_preset.set(false);
                                                        recorte_threshold.set((curr + 5).min(98));
                                                    }
                                                },
                                                "+"
                                            }
                                        }
                                    }
                                }

                                // Bloco de Zoom Granular da Chapa
                                div { class: "zoom-control-box",
                                    div { class: "zoom-header-row",
                                        span { class: "zoom-title", "🔍 ZOOM DA FOTO: {image_zoom_display}" }
                                        button {
                                            class: "btn-zoom-reset",
                                            onclick: move |_| image_zoom.set(100),
                                            "1.0x (RESET)"
                                        }
                                    }
                                    div { class: "zoom-slider-row",
                                        button {
                                            class: "btn-zoom-step",
                                            onclick: move |_| {
                                                let curr = image_zoom();
                                                if curr > 25 { image_zoom.set(curr.saturating_sub(10)); }
                                            },
                                            "−"
                                        }
                                        input {
                                            r#type: "range",
                                            min: "25",
                                            max: "300",
                                            step: "5",
                                            value: "{image_zoom}",
                                            class: "slider-punk-zoom",
                                            oninput: move |e| {
                                                if let Ok(v) = e.value().parse::<u32>() { image_zoom.set(v); }
                                            }
                                        }
                                        button {
                                            class: "btn-zoom-step",
                                            onclick: move |_| {
                                                let curr = image_zoom();
                                                if curr < 300 { image_zoom.set(curr + 10); }
                                            },
                                            "+"
                                        }
                                    }
                                    div { class: "zoom-presets-row",
                                        button {
                                            class: if image_zoom() == 70 { "btn-zoom-quick active" } else { "btn-zoom-quick" },
                                            onclick: move |_| image_zoom.set(70),
                                            "AFASTAR (0.7x)"
                                        }
                                        button {
                                            class: if image_zoom() == 100 { "btn-zoom-quick active" } else { "btn-zoom-quick" },
                                            onclick: move |_| image_zoom.set(100),
                                            "PADRÃO (1.0x)"
                                        }
                                        button {
                                            class: if image_zoom() == 150 { "btn-zoom-quick active" } else { "btn-zoom-quick" },
                                            onclick: move |_| image_zoom.set(150),
                                            "APROXIMAR (1.5x)"
                                        }
                                    }
                                }

                                // Bloco de Posição da Foto no Canvas
                                div { class: "zoom-control-box", style: "margin-top: 10px;",
                                    div { class: "zoom-header-row",
                                        span { class: "zoom-title", "📐 POSIÇÃO DA FOTO (X: {image_pos_x}%, Y: {image_pos_y}%)" }
                                        button {
                                            class: "btn-zoom-reset",
                                            onclick: move |_| {
                                                image_pos_x.set(50);
                                                image_pos_y.set(50);
                                            },
                                            "↺ CENTRALIZAR"
                                        }
                                    }
                                    div { style: "display: flex; flex-direction: column; gap: 8px; margin-top: 8px;",
                                        div { style: "display: flex; align-items: center; gap: 6px;",
                                            span { style: "font-size: 0.72rem; min-width: 48px; opacity: 0.8;", "EIXO X:" }
                                            button {
                                                r#type: "button",
                                                class: "btn-stepper",
                                                onclick: move |_| {
                                                    let curr = image_pos_x();
                                                    if curr > 0 { image_pos_x.set(curr.saturating_sub(5)); }
                                                },
                                                "−"
                                            }
                                            input {
                                                r#type: "range",
                                                min: "0",
                                                max: "100",
                                                step: "1",
                                                value: "{image_pos_x}",
                                                class: "slider-punk-zoom",
                                                style: "flex: 1;",
                                                oninput: move |e| {
                                                    if let Ok(v) = e.value().parse::<u32>() { image_pos_x.set(v); }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "btn-stepper",
                                                onclick: move |_| {
                                                    let curr = image_pos_x();
                                                    if curr < 100 { image_pos_x.set((curr + 5).min(100)); }
                                                },
                                                "+"
                                            }
                                            span { style: "font-size: 0.75rem; min-width: 36px; text-align: right; font-weight: bold;", "{image_pos_x}%" }
                                        }
                                        div { style: "display: flex; align-items: center; gap: 6px;",
                                            span { style: "font-size: 0.72rem; min-width: 48px; opacity: 0.8;", "EIXO Y:" }
                                            button {
                                                r#type: "button",
                                                class: "btn-stepper",
                                                onclick: move |_| {
                                                    let curr = image_pos_y();
                                                    if curr > 0 { image_pos_y.set(curr.saturating_sub(5)); }
                                                },
                                                "−"
                                            }
                                            input {
                                                r#type: "range",
                                                min: "0",
                                                max: "100",
                                                step: "1",
                                                value: "{image_pos_y}",
                                                class: "slider-punk-zoom",
                                                style: "flex: 1;",
                                                oninput: move |e| {
                                                    if let Ok(v) = e.value().parse::<u32>() { image_pos_y.set(v); }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "btn-stepper",
                                                onclick: move |_| {
                                                    let curr = image_pos_y();
                                                    if curr < 100 { image_pos_y.set((curr + 5).min(100)); }
                                                },
                                                "+"
                                            }
                                            span { style: "font-size: 0.75rem; min-width: 36px; text-align: right; font-weight: bold;", "{image_pos_y}%" }
                                        }
                                    }
                                }

                                // Bloco de Enquadramento e Corte Manual (Crop / Bounding Box - Ticket #17)
                                div { class: "zoom-control-box", style: "margin-top: 10px;",
                                    div { class: "zoom-header-row",
                                        span { class: "zoom-title", "✂️ RECORTE DE BORDAS / CROP (0% A 45%)" }
                                        button {
                                            r#type: "button",
                                            class: "btn-zoom-reset",
                                            onclick: move |_| {
                                                image_crop_top.set(0);
                                                image_crop_bottom.set(0);
                                                image_crop_left.set(0);
                                                image_crop_right.set(0);
                                            },
                                            "↺ ZERAR (0%)"
                                        }
                                    }
                                    div { style: "display: flex; flex-direction: column; gap: 8px; margin-top: 8px;",
                                        div { style: "display: flex; align-items: center; gap: 6px;",
                                            span { style: "font-size: 0.72rem; min-width: 68px; opacity: 0.8;", "CORTAR TOPO:" }
                                            button {
                                                r#type: "button",
                                                class: "btn-stepper",
                                                onclick: move |_| {
                                                    let curr = image_crop_top();
                                                    if curr > 0 { image_crop_top.set(curr.saturating_sub(5)); }
                                                },
                                                "−"
                                            }
                                            input {
                                                r#type: "range",
                                                min: "0",
                                                max: "45",
                                                step: "1",
                                                value: "{image_crop_top}",
                                                class: "slider-punk-zoom",
                                                style: "flex: 1;",
                                                oninput: move |e| {
                                                    if let Ok(v) = e.value().parse::<u32>() { image_crop_top.set(v); }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "btn-stepper",
                                                onclick: move |_| {
                                                    let curr = image_crop_top();
                                                    if curr < 45 { image_crop_top.set((curr + 5).min(45)); }
                                                },
                                                "+"
                                            }
                                            span { style: "font-size: 0.75rem; min-width: 36px; text-align: right; font-weight: bold;", "{image_crop_top}%" }
                                        }
                                        div { style: "display: flex; align-items: center; gap: 6px;",
                                            span { style: "font-size: 0.72rem; min-width: 68px; opacity: 0.8;", "CORTAR BASE:" }
                                            button {
                                                r#type: "button",
                                                class: "btn-stepper",
                                                onclick: move |_| {
                                                    let curr = image_crop_bottom();
                                                    if curr > 0 { image_crop_bottom.set(curr.saturating_sub(5)); }
                                                },
                                                "−"
                                            }
                                            input {
                                                r#type: "range",
                                                min: "0",
                                                max: "45",
                                                step: "1",
                                                value: "{image_crop_bottom}",
                                                class: "slider-punk-zoom",
                                                style: "flex: 1;",
                                                oninput: move |e| {
                                                    if let Ok(v) = e.value().parse::<u32>() { image_crop_bottom.set(v); }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "btn-stepper",
                                                onclick: move |_| {
                                                    let curr = image_crop_bottom();
                                                    if curr < 45 { image_crop_bottom.set((curr + 5).min(45)); }
                                                },
                                                "+"
                                            }
                                            span { style: "font-size: 0.75rem; min-width: 36px; text-align: right; font-weight: bold;", "{image_crop_bottom}%" }
                                        }
                                        div { style: "display: flex; align-items: center; gap: 6px;",
                                            span { style: "font-size: 0.72rem; min-width: 68px; opacity: 0.8;", "CORTAR ESQ:" }
                                            button {
                                                r#type: "button",
                                                class: "btn-stepper",
                                                onclick: move |_| {
                                                    let curr = image_crop_left();
                                                    if curr > 0 { image_crop_left.set(curr.saturating_sub(5)); }
                                                },
                                                "−"
                                            }
                                            input {
                                                r#type: "range",
                                                min: "0",
                                                max: "45",
                                                step: "1",
                                                value: "{image_crop_left}",
                                                class: "slider-punk-zoom",
                                                style: "flex: 1;",
                                                oninput: move |e| {
                                                    if let Ok(v) = e.value().parse::<u32>() { image_crop_left.set(v); }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "btn-stepper",
                                                onclick: move |_| {
                                                    let curr = image_crop_left();
                                                    if curr < 45 { image_crop_left.set((curr + 5).min(45)); }
                                                },
                                                "+"
                                            }
                                            span { style: "font-size: 0.75rem; min-width: 36px; text-align: right; font-weight: bold;", "{image_crop_left}%" }
                                        }
                                        div { style: "display: flex; align-items: center; gap: 6px;",
                                            span { style: "font-size: 0.72rem; min-width: 68px; opacity: 0.8;", "CORTAR DIR:" }
                                            button {
                                                r#type: "button",
                                                class: "btn-stepper",
                                                onclick: move |_| {
                                                    let curr = image_crop_right();
                                                    if curr > 0 { image_crop_right.set(curr.saturating_sub(5)); }
                                                },
                                                "−"
                                            }
                                            input {
                                                r#type: "range",
                                                min: "0",
                                                max: "45",
                                                step: "1",
                                                value: "{image_crop_right}",
                                                class: "slider-punk-zoom",
                                                style: "flex: 1;",
                                                oninput: move |e| {
                                                    if let Ok(v) = e.value().parse::<u32>() { image_crop_right.set(v); }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "btn-stepper",
                                                onclick: move |_| {
                                                    let curr = image_crop_right();
                                                    if curr < 45 { image_crop_right.set((curr + 5).min(45)); }
                                                },
                                                "+"
                                            }
                                            span { style: "font-size: 0.75rem; min-width: 36px; text-align: right; font-weight: bold;", "{image_crop_right}%" }
                                        }
                                    }
                                }

                                span { class: "help-text", "Suporta PNG com alfa, JPEG e WebP." }
                            }

                            div { class: "control-box",
                                label { class: "section-tag", "2. FAIXAS DE TEXTO" }
                                div { style: "display: flex; align-items: center; justify-content: space-between; padding: 6px 10px; background: rgba(0, 0, 0, 0.4); border: 1px dashed #ff0033; margin-bottom: 10px;",
                                    div { style: "display: flex; align-items: center; gap: 8px;",
                                        input {
                                            r#type: "checkbox",
                                            id: "banner-sem-faixa-cb",
                                            checked: "{banner_sem_faixa}",
                                            onchange: move |e| banner_sem_faixa.set(e.checked()),
                                        }
                                        label { r#for: "banner-sem-faixa-cb", style: "font-size: 0.75rem; color: #ff3366; font-weight: bold; cursor: pointer; text-transform: uppercase;", "🚫 Sem Faixa (Remover Tarjas do Pôster)" }
                                    }
                                    if banner_sem_faixa() {
                                        span { style: "font-size: 0.68rem; color: #ff9999; font-style: italic;", "[ FAIXAS DESATIVADAS ]" }
                                    }
                                }
                                div { style: "display: flex; flex-direction: column; gap: 10px;",
                                    div {
                                        label { style: "font-size: 0.72rem; color: #888; text-transform: uppercase; letter-spacing: 1px; display: block; margin-bottom: 4px; font-weight: bold;", "⬆️ Faixa Superior (Opcional - Suporta Multilinha)" }
                                        textarea {
                                            class: "text-input-punk",
                                            rows: "2",
                                            style: "resize: vertical; font-family: inherit;",
                                            value: "{phrase_top}",
                                            placeholder: "DIGITE A MANCHETE SUPERIOR (SUPORTA QUEBRA DE LINHA)...",
                                            oninput: move |evt| phrase_top.set(evt.value()),
                                        }
                                    }
                                    div {
                                        label { style: "font-size: 0.72rem; color: #888; text-transform: uppercase; letter-spacing: 1px; display: block; margin-bottom: 4px; font-weight: bold;", "⬇️ Faixa Inferior (Suporta Multilinha)" }
                                        textarea {
                                            class: "text-input-punk",
                                            rows: "2",
                                            style: "resize: vertical; font-family: inherit;",
                                            value: "{phrase}",
                                            placeholder: "DIGITE A FRASE PRINCIPAL (SUPORTA QUEBRA DE LINHA)...",
                                            oninput: move |evt| phrase.set(evt.value()),
                                        }
                                    }

                                    div { class: "banner-tabs-container",
                                        button {
                                            r#type: "button",
                                            class: if banner_active_tab() == "bottom" { "btn-banner-tab active" } else { "btn-banner-tab" },
                                            onclick: move |_| banner_active_tab.set(String::from("bottom")),
                                            "⬇️ FAIXA INFERIOR (BASE)"
                                        }
                                        button {
                                            r#type: "button",
                                            class: if banner_active_tab() == "top" { "btn-banner-tab active" } else { "btn-banner-tab" },
                                            onclick: move |_| banner_active_tab.set(String::from("top")),
                                            "⬆️ FAIXA SUPERIOR (TOPO)"
                                        }
                                    }

                                    if banner_active_tab() == "top" {
                                        // --- CONTROLES DA FAIXA SUPERIOR (Tickets #15, #16 e #18) ---
                                        div { style: "display: flex; flex-direction: column; gap: 8px;",
                                            // Posicionamento e inclinação no canvas (sempre editáveis)
                                            div { class: "form-row-3col",
                                                div {
                                                    div { class: "stepper-slider-box",
                                                        div { class: "stepper-header-row",
                                                            span { class: "stepper-title", "Posição X: {banner_top_pos_x}%" }
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper-reset",
                                                                onclick: move |_| banner_top_pos_x.set(50),
                                                                "↺ 50%"
                                                            }
                                                        }
                                                        div { class: "stepper-row",
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper",
                                                                onclick: move |_| {
                                                                    let curr = banner_top_pos_x();
                                                                    if curr > 0 { banner_top_pos_x.set(curr.saturating_sub(5)); }
                                                                },
                                                                "−"
                                                            }
                                                            input {
                                                                r#type: "range", min: "0", max: "100",
                                                                value: "{banner_top_pos_x}",
                                                                class: "slider-punk-zoom",
                                                                oninput: move |e| {
                                                                    if let Ok(v) = e.value().parse::<u32>() { banner_top_pos_x.set(v); }
                                                                }
                                                            }
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper",
                                                                onclick: move |_| {
                                                                    let curr = banner_top_pos_x();
                                                                    if curr < 100 { banner_top_pos_x.set((curr + 5).min(100)); }
                                                                },
                                                                "+"
                                                            }
                                                        }
                                                    }
                                                }
                                                div {
                                                    div { class: "stepper-slider-box",
                                                        div { class: "stepper-header-row",
                                                            span { class: "stepper-title", "Posição Y: {banner_top_pos_y}%" }
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper-reset",
                                                                onclick: move |_| banner_top_pos_y.set(10),
                                                                "↺ 10%"
                                                            }
                                                        }
                                                        div { class: "stepper-row",
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper",
                                                                onclick: move |_| {
                                                                    let curr = banner_top_pos_y();
                                                                    if curr > 0 { banner_top_pos_y.set(curr.saturating_sub(5)); }
                                                                },
                                                                "−"
                                                            }
                                                            input {
                                                                r#type: "range", min: "0", max: "100",
                                                                value: "{banner_top_pos_y}",
                                                                class: "slider-punk-zoom",
                                                                oninput: move |e| {
                                                                    if let Ok(v) = e.value().parse::<u32>() { banner_top_pos_y.set(v); }
                                                                }
                                                            }
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper",
                                                                onclick: move |_| {
                                                                    let curr = banner_top_pos_y();
                                                                    if curr < 100 { banner_top_pos_y.set((curr + 5).min(100)); }
                                                                },
                                                                "+"
                                                            }
                                                        }
                                                    }
                                                }
                                                div {
                                                    div { class: "stepper-slider-box",
                                                        div { class: "stepper-header-row",
                                                            span { class: "stepper-title", "Inclinação: {banner_top_rotation}°" }
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper-reset",
                                                                onclick: move |_| banner_top_rotation.set(0),
                                                                "↺ 0°"
                                                            }
                                                        }
                                                        div { class: "stepper-row",
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper",
                                                                onclick: move |_| {
                                                                    let curr = banner_top_rotation();
                                                                    if curr > -45 { banner_top_rotation.set((curr - 5).max(-45)); }
                                                                },
                                                                "−"
                                                            }
                                                            input {
                                                                r#type: "range", min: "-45", max: "45",
                                                                value: "{banner_top_rotation}",
                                                                class: "slider-punk-zoom",
                                                                oninput: move |e| {
                                                                    if let Ok(v) = e.value().parse::<i16>() { banner_top_rotation.set(v); }
                                                                }
                                                            }
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper",
                                                                onclick: move |_| {
                                                                    let curr = banner_top_rotation();
                                                                    if curr < 45 { banner_top_rotation.set((curr + 5).min(45)); }
                                                                },
                                                                "+"
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            // Sincronização estética com o preset ativo
                                            div { style: "display: flex; align-items: center; gap: 6px; padding: 6px 8px; background: rgba(255, 0, 51, 0.1); border: 1px dashed #ff0033;",
                                                input {
                                                    r#type: "checkbox",
                                                    id: "banner-top-adapt-preset-cb",
                                                    checked: "{banner_top_adapt_preset}",
                                                    onchange: move |e| banner_top_adapt_preset.set(e.checked()),
                                                }
                                                label { r#for: "banner-top-adapt-preset-cb", style: "font-size: 0.72rem; color: #ff3366; font-weight: bold; cursor: pointer; text-transform: uppercase;", "⚡ Modo Adequado ao Preset Ativo (Sincronizar Cores, Fontes e Bordas)" }
                                            }

                                            if banner_top_adapt_preset() {
                                                div { class: "preset-sync-notice",
                                                    "🔒 ESTÉTICA SINCRONIZADA AO PRESET (DESMARQUE ACIMA PARA PERSONALIZAR CORES, FONTES E BORDAS)"
                                                }
                                            }

                                            // Controles estéticos (desativados quando sincronizados ao preset)
                                            div { class: if banner_top_adapt_preset() { "controls-locked-by-preset" } else { "" },
                                                div { class: "form-row-2col",
                                                    div {
                                                        label { style: "font-size: 0.68rem; color: #888; text-transform: uppercase; display: block; margin-bottom: 2px;", "Família Tipográfica" }
                                                        select {
                                                            class: "select-preset-punk",
                                                            disabled: "{banner_top_adapt_preset}",
                                                            value: "{banner_top_font_family}",
                                                            onchange: move |e| banner_top_font_family.set(e.value()),
                                                            option { value: "impact", "IMPACT // Bebas" }
                                                            option { value: "monospace", "MONO // Terminal" }
                                                            option { value: "serif", "SERIF // Jornal" }
                                                            option { value: "sans", "SANS // Moderna" }
                                                        }
                                                    }
                                                    div {
                                                        div { class: "stepper-slider-box",
                                                            div { class: "stepper-header-row",
                                                                span { class: "stepper-title", "Tamanho: {banner_top_font_size}px" }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper-reset",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    onclick: move |_| banner_top_font_size.set(48),
                                                                    "↺ 48px"
                                                                }
                                                            }
                                                            div { class: "stepper-row",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_top_font_size();
                                                                        if curr > 20 { banner_top_font_size.set(curr.saturating_sub(4).max(20)); }
                                                                    },
                                                                    "−"
                                                                }
                                                                input {
                                                                    r#type: "range", min: "20", max: "96",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    value: "{banner_top_font_size}",
                                                                    class: "slider-punk-zoom",
                                                                    oninput: move |e| {
                                                                        if let Ok(v) = e.value().parse::<u32>() { banner_top_font_size.set(v); }
                                                                    }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_top_font_size();
                                                                        if curr < 96 { banner_top_font_size.set((curr + 4).min(96)); }
                                                                    },
                                                                    "+"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                div { style: "margin-top: 4px; display: flex; align-items: center; gap: 6px;",
                                                    input {
                                                        r#type: "checkbox",
                                                        id: "banner-top-auto-fit-cb",
                                                        disabled: "{banner_top_adapt_preset}",
                                                        checked: "{banner_top_auto_fit}",
                                                        onchange: move |e| banner_top_auto_fit.set(e.checked()),
                                                    }
                                                    label { r#for: "banner-top-auto-fit-cb", style: "font-size: 0.72rem; color: #ccc; cursor: pointer;", "Reduzir fonte automaticamente se exceder a faixa (auto-fit)" }
                                                }

                                                div { class: "form-row-3col", style: "margin-top: 6px;",
                                                    div {
                                                        label { style: "font-size: 0.68rem; color: #888; text-transform: uppercase; display: block; margin-bottom: 2px;", "Cor do Texto" }
                                                        select {
                                                            class: "select-preset-punk",
                                                            disabled: "{banner_top_adapt_preset}",
                                                            value: "{banner_top_fg_color}",
                                                            onchange: move |e| banner_top_fg_color.set(e.value()),
                                                            option { value: "auto", "AUTO // Preset" }
                                                            option { value: "white", "BRANCO // #FFFFFF" }
                                                            option { value: "black", "PRETO // #000000" }
                                                            option { value: "yellow", "AMARELO // #FFEE00" }
                                                            option { value: "red", "VERMELHO // #FF0033" }
                                                            option { value: "cyan", "CIANO // #00FFFF" }
                                                            option { value: "green", "VERDE // #00FF66" }
                                                        }
                                                    }
                                                    div {
                                                        label { style: "font-size: 0.68rem; color: #888; text-transform: uppercase; display: block; margin-bottom: 2px;", "Fundo da Tarja" }
                                                        select {
                                                            class: "select-preset-punk",
                                                            disabled: "{banner_top_adapt_preset}",
                                                            value: "{banner_top_bg_color}",
                                                            onchange: move |e| banner_top_bg_color.set(e.value()),
                                                            option { value: "auto", "AUTO // Preset" }
                                                            option { value: "black", "PRETO // #000000" }
                                                            option { value: "white", "BRANCO // #FFFFFF" }
                                                            option { value: "red", "VERMELHO // #FF0033" }
                                                            option { value: "yellow", "AMARELO // #FFEE00" }
                                                            option { value: "none", "SEM FUNDO // Transparente" }
                                                        }
                                                    }
                                                    div {
                                                        label { style: "font-size: 0.68rem; color: #888; text-transform: uppercase; display: block; margin-bottom: 2px;", "Acabamento" }
                                                        select {
                                                            class: "select-preset-punk",
                                                            disabled: "{banner_top_adapt_preset}",
                                                            value: "{banner_top_effect}",
                                                            onchange: move |e| banner_top_effect.set(e.value()),
                                                            option { value: "solida", "SÓLIDA" }
                                                            option { value: "xerox", "XEROX // Toner" }
                                                        }
                                                    }
                                                }

                                                div { class: "form-row-2col", style: "margin-top: 6px;",
                                                    div {
                                                        label { style: "font-size: 0.68rem; color: #888; text-transform: uppercase; display: block; margin-bottom: 2px;", "Estilo de Borda" }
                                                        select {
                                                            class: "select-preset-punk",
                                                            disabled: "{banner_top_adapt_preset}",
                                                            value: "{banner_top_border_finish}",
                                                            onchange: move |e| banner_top_border_finish.set(e.value()),
                                                            option { value: "nenhuma", "NENHUMA" }
                                                            option { value: "solida", "SÓLIDA" }
                                                            option { value: "tracejada", "TRACEJADA" }
                                                            option { value: "picote", "PICOTE // Serrilhado" }
                                                            option { value: "durex", "FITA DUREX // Adesiva" }
                                                        }
                                                    }
                                                    div {
                                                        div { class: "stepper-slider-box",
                                                            div { class: "stepper-header-row",
                                                                span { class: "stepper-title", "Espessura Borda: {banner_top_border_width}px" }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper-reset",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    onclick: move |_| banner_top_border_width.set(3),
                                                                    "↺ 3px"
                                                                }
                                                            }
                                                            div { class: "stepper-row",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_top_border_width();
                                                                        if curr > 1 { banner_top_border_width.set(curr.saturating_sub(1)); }
                                                                    },
                                                                    "−"
                                                                }
                                                                input {
                                                                    r#type: "range", min: "1", max: "12",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    value: "{banner_top_border_width}",
                                                                    class: "slider-punk-zoom",
                                                                    oninput: move |e| {
                                                                        if let Ok(v) = e.value().parse::<u32>() { banner_top_border_width.set(v); }
                                                                    }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_top_border_width();
                                                                        if curr < 12 { banner_top_border_width.set(curr + 1); }
                                                                    },
                                                                    "+"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                div { class: "form-row-2col", style: "margin-top: 6px;",
                                                    div {
                                                        div { class: "stepper-slider-box",
                                                            div { class: "stepper-header-row",
                                                                span { class: "stepper-title", "Ondulação: {banner_top_wave_amp}px" }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper-reset",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    onclick: move |_| banner_top_wave_amp.set(0),
                                                                    "↺ 0px"
                                                                }
                                                            }
                                                            div { class: "stepper-row",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_top_wave_amp();
                                                                        if curr > 0 { banner_top_wave_amp.set(curr.saturating_sub(2)); }
                                                                    },
                                                                    "−"
                                                                }
                                                                input {
                                                                    r#type: "range", min: "0", max: "30",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    value: "{banner_top_wave_amp}",
                                                                    class: "slider-punk-zoom",
                                                                    oninput: move |e| {
                                                                        if let Ok(v) = e.value().parse::<u32>() { banner_top_wave_amp.set(v); }
                                                                    }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_top_wave_amp();
                                                                        if curr < 30 { banner_top_wave_amp.set((curr + 2).min(30)); }
                                                                    },
                                                                    "+"
                                                                }
                                                            }
                                                        }
                                                    }
                                                    div {
                                                        div { class: "stepper-slider-box",
                                                            div { class: "stepper-header-row",
                                                                span { class: "stepper-title", "Jitter Palavras: {banner_top_word_rot}°" }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper-reset",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        banner_top_word_rot.set(0);
                                                                        banner_top_word_cut.set(false);
                                                                    },
                                                                    "↺ 0°"
                                                                }
                                                            }
                                                            div { class: "stepper-row",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_top_word_rot();
                                                                        if curr > 0 {
                                                                            let next = curr.saturating_sub(2);
                                                                            banner_top_word_rot.set(next);
                                                                            if next == 0 { banner_top_word_cut.set(false); }
                                                                        }
                                                                    },
                                                                    "−"
                                                                }
                                                                input {
                                                                    r#type: "range", min: "0", max: "20",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    value: "{banner_top_word_rot}",
                                                                    class: "slider-punk-zoom",
                                                                    oninput: move |e| {
                                                                        if let Ok(v) = e.value().parse::<u32>() {
                                                                            banner_top_word_rot.set(v);
                                                                            if v > 0 { banner_top_word_cut.set(true); } else { banner_top_word_cut.set(false); }
                                                                        }
                                                                    }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_top_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_top_word_rot();
                                                                        if curr < 20 {
                                                                            banner_top_word_rot.set((curr + 2).min(20));
                                                                            banner_top_word_cut.set(true);
                                                                        }
                                                                    },
                                                                    "+"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                // Modo Fanzine / Ransom Note (Ticket #18)
                                                div { style: "margin-top: 8px; display: flex; align-items: center; gap: 6px; padding: 6px 8px; background: rgba(0, 0, 0, 0.4); border: 1px dashed #555;",
                                                    input {
                                                        r#type: "checkbox",
                                                        id: "banner-top-ransom-note-cb",
                                                        disabled: "{banner_top_adapt_preset}",
                                                        checked: "{banner_top_ransom_note}",
                                                        onchange: move |e| banner_top_ransom_note.set(e.checked()),
                                                    }
                                                    label {
                                                        r#for: "banner-top-ransom-note-cb",
                                                        style: "font-size: 0.72rem; color: #ff007f; font-weight: bold; cursor: pointer; text-transform: uppercase;",
                                                        "✂️ Colagem de Revista / Ransom Note (Fontes e cores alternadas por palavra)"
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        // --- CONTROLES DA FAIXA INFERIOR (Tickets #15, #16 e #18) ---
                                        div { style: "display: flex; flex-direction: column; gap: 8px;",
                                            // Posicionamento e inclinação no canvas (sempre editáveis)
                                            div { class: "form-row-3col",
                                                div {
                                                    div { class: "stepper-slider-box",
                                                        div { class: "stepper-header-row",
                                                            span { class: "stepper-title", "Posição X: {banner_pos_x}%" }
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper-reset",
                                                                onclick: move |_| banner_pos_x.set(50),
                                                                "↺ 50%"
                                                            }
                                                        }
                                                        div { class: "stepper-row",
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper",
                                                                onclick: move |_| {
                                                                    let curr = banner_pos_x();
                                                                    if curr > 0 { banner_pos_x.set(curr.saturating_sub(5)); }
                                                                },
                                                                "−"
                                                            }
                                                            input {
                                                                r#type: "range", min: "0", max: "100",
                                                                value: "{banner_pos_x}",
                                                                class: "slider-punk-zoom",
                                                                oninput: move |e| {
                                                                    if let Ok(v) = e.value().parse::<u32>() { banner_pos_x.set(v); }
                                                                }
                                                            }
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper",
                                                                onclick: move |_| {
                                                                    let curr = banner_pos_x();
                                                                    if curr < 100 { banner_pos_x.set((curr + 5).min(100)); }
                                                                },
                                                                "+"
                                                            }
                                                        }
                                                    }
                                                }
                                                div {
                                                    div { class: "stepper-slider-box",
                                                        div { class: "stepper-header-row",
                                                            span { class: "stepper-title", "Posição Y: {banner_pos_y}%" }
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper-reset",
                                                                onclick: move |_| banner_pos_y.set(85),
                                                                "↺ 85%"
                                                            }
                                                        }
                                                        div { class: "stepper-row",
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper",
                                                                onclick: move |_| {
                                                                    let curr = banner_pos_y();
                                                                    if curr > 0 { banner_pos_y.set(curr.saturating_sub(5)); }
                                                                },
                                                                "−"
                                                             }
                                                            input {
                                                                r#type: "range", min: "0", max: "100",
                                                                value: "{banner_pos_y}",
                                                                class: "slider-punk-zoom",
                                                                oninput: move |e| {
                                                                    if let Ok(v) = e.value().parse::<u32>() { banner_pos_y.set(v); }
                                                                }
                                                            }
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper",
                                                                onclick: move |_| {
                                                                    let curr = banner_pos_y();
                                                                    if curr < 100 { banner_pos_y.set((curr + 5).min(100)); }
                                                                },
                                                                "+"
                                                            }
                                                        }
                                                    }
                                                }
                                                div {
                                                    div { class: "stepper-slider-box",
                                                        div { class: "stepper-header-row",
                                                            span { class: "stepper-title", "Inclinação: {banner_rotation}°" }
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper-reset",
                                                                onclick: move |_| banner_rotation.set(0),
                                                                "↺ 0°"
                                                            }
                                                        }
                                                        div { class: "stepper-row",
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper",
                                                                onclick: move |_| {
                                                                    let curr = banner_rotation();
                                                                    if curr > -45 { banner_rotation.set((curr - 5).max(-45)); }
                                                                },
                                                                "−"
                                                            }
                                                            input {
                                                                r#type: "range", min: "-45", max: "45",
                                                                value: "{banner_rotation}",
                                                                class: "slider-punk-zoom",
                                                                oninput: move |e| {
                                                                    if let Ok(v) = e.value().parse::<i16>() { banner_rotation.set(v); }
                                                                }
                                                            }
                                                            button {
                                                                r#type: "button",
                                                                class: "btn-stepper",
                                                                onclick: move |_| {
                                                                    let curr = banner_rotation();
                                                                    if curr < 45 { banner_rotation.set((curr + 5).min(45)); }
                                                                },
                                                                "+"
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            // Sincronização estética com o preset ativo
                                            div { style: "display: flex; align-items: center; gap: 6px; padding: 6px 8px; background: rgba(255, 0, 51, 0.1); border: 1px dashed #ff0033;",
                                                input {
                                                    r#type: "checkbox",
                                                    id: "banner-adapt-preset-cb",
                                                    checked: "{banner_adapt_preset}",
                                                    onchange: move |e| banner_adapt_preset.set(e.checked()),
                                                }
                                                label { r#for: "banner-adapt-preset-cb", style: "font-size: 0.72rem; color: #ff3366; font-weight: bold; cursor: pointer; text-transform: uppercase;", "⚡ Modo Adequado ao Preset Ativo (Sincronizar Cores, Fontes e Bordas)" }
                                            }

                                            if banner_adapt_preset() {
                                                div { class: "preset-sync-notice",
                                                    "🔒 ESTÉTICA SINCRONIZADA AO PRESET (DESMARQUE ACIMA PARA PERSONALIZAR CORES, FONTES E BORDAS)"
                                                }
                                            }

                                            // Controles estéticos (desativados quando sincronizados ao preset)
                                            div { class: if banner_adapt_preset() { "controls-locked-by-preset" } else { "" },
                                                div { class: "form-row-2col",
                                                    div {
                                                        label { style: "font-size: 0.68rem; color: #888; text-transform: uppercase; display: block; margin-bottom: 2px;", "Família Tipográfica" }
                                                        select {
                                                            class: "select-preset-punk",
                                                            disabled: "{banner_adapt_preset}",
                                                            value: "{banner_font_family}",
                                                            onchange: move |e| banner_font_family.set(e.value()),
                                                            option { value: "impact", "IMPACT // Bebas" }
                                                            option { value: "monospace", "MONO // Terminal" }
                                                            option { value: "serif", "SERIF // Jornal" }
                                                            option { value: "sans", "SANS // Moderna" }
                                                        }
                                                    }
                                                    div {
                                                        div { class: "stepper-slider-box",
                                                            div { class: "stepper-header-row",
                                                                span { class: "stepper-title", "Tamanho: {banner_font_size}px" }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper-reset",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    onclick: move |_| banner_font_size.set(48),
                                                                    "↺ 48px"
                                                                }
                                                            }
                                                            div { class: "stepper-row",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_font_size();
                                                                        if curr > 20 { banner_font_size.set(curr.saturating_sub(4).max(20)); }
                                                                    },
                                                                    "−"
                                                                }
                                                                input {
                                                                    r#type: "range", min: "20", max: "96",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    value: "{banner_font_size}",
                                                                    class: "slider-punk-zoom",
                                                                    oninput: move |e| {
                                                                        if let Ok(v) = e.value().parse::<u32>() { banner_font_size.set(v); }
                                                                    }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_font_size();
                                                                        if curr < 96 { banner_font_size.set((curr + 4).min(96)); }
                                                                    },
                                                                    "+"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                div { style: "margin-top: 4px; display: flex; align-items: center; gap: 6px;",
                                                    input {
                                                        r#type: "checkbox",
                                                        id: "banner-auto-fit-cb",
                                                        disabled: "{banner_adapt_preset}",
                                                        checked: "{banner_auto_fit}",
                                                        onchange: move |e| banner_auto_fit.set(e.checked()),
                                                    }
                                                    label { r#for: "banner-auto-fit-cb", style: "font-size: 0.72rem; color: #ccc; cursor: pointer;", "Reduzir fonte automaticamente se exceder a faixa (auto-fit)" }
                                                }

                                                div { class: "form-row-3col", style: "margin-top: 6px;",
                                                    div {
                                                        label { style: "font-size: 0.68rem; color: #888; text-transform: uppercase; display: block; margin-bottom: 2px;", "Cor do Texto" }
                                                        select {
                                                            class: "select-preset-punk",
                                                            disabled: "{banner_adapt_preset}",
                                                            value: "{banner_fg_color}",
                                                            onchange: move |e| banner_fg_color.set(e.value()),
                                                            option { value: "auto", "AUTO // Preset" }
                                                            option { value: "white", "BRANCO // #FFFFFF" }
                                                            option { value: "black", "PRETO // #000000" }
                                                            option { value: "yellow", "AMARELO // #FFEE00" }
                                                            option { value: "red", "VERMELHO // #FF0033" }
                                                            option { value: "cyan", "CIANO // #00FFFF" }
                                                            option { value: "green", "VERDE // #00FF66" }
                                                        }
                                                    }
                                                    div {
                                                        label { style: "font-size: 0.68rem; color: #888; text-transform: uppercase; display: block; margin-bottom: 2px;", "Fundo da Tarja" }
                                                        select {
                                                            class: "select-preset-punk",
                                                            disabled: "{banner_adapt_preset}",
                                                            value: "{banner_bg_color}",
                                                            onchange: move |e| banner_bg_color.set(e.value()),
                                                            option { value: "auto", "AUTO // Preset" }
                                                            option { value: "black", "PRETO // #000000" }
                                                            option { value: "white", "BRANCO // #FFFFFF" }
                                                            option { value: "red", "VERMELHO // #FF0033" }
                                                            option { value: "yellow", "AMARELO // #FFEE00" }
                                                            option { value: "none", "SEM FUNDO // Transparente" }
                                                        }
                                                    }
                                                    div {
                                                        label { style: "font-size: 0.68rem; color: #888; text-transform: uppercase; display: block; margin-bottom: 2px;", "Acabamento" }
                                                        select {
                                                            class: "select-preset-punk",
                                                            disabled: "{banner_adapt_preset}",
                                                            value: "{banner_effect}",
                                                            onchange: move |e| banner_effect.set(e.value()),
                                                            option { value: "solida", "SÓLIDA" }
                                                            option { value: "xerox", "XEROX // Toner" }
                                                        }
                                                    }
                                                }

                                                div { class: "form-row-2col", style: "margin-top: 6px;",
                                                    div {
                                                        label { style: "font-size: 0.68rem; color: #888; text-transform: uppercase; display: block; margin-bottom: 2px;", "Estilo de Borda" }
                                                        select {
                                                            class: "select-preset-punk",
                                                            disabled: "{banner_adapt_preset}",
                                                            value: "{banner_border_finish}",
                                                            onchange: move |e| banner_border_finish.set(e.value()),
                                                            option { value: "nenhuma", "NENHUMA" }
                                                            option { value: "solida", "SÓLIDA" }
                                                            option { value: "tracejada", "TRACEJADA" }
                                                            option { value: "picote", "PICOTE // Serrilhado" }
                                                            option { value: "durex", "FITA DUREX // Adesiva" }
                                                        }
                                                    }
                                                    div {
                                                        div { class: "stepper-slider-box",
                                                            div { class: "stepper-header-row",
                                                                span { class: "stepper-title", "Espessura Borda: {banner_border_width}px" }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper-reset",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    onclick: move |_| banner_border_width.set(3),
                                                                    "↺ 3px"
                                                                }
                                                            }
                                                            div { class: "stepper-row",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_border_width();
                                                                        if curr > 1 { banner_border_width.set(curr.saturating_sub(1)); }
                                                                    },
                                                                    "−"
                                                                }
                                                                input {
                                                                    r#type: "range", min: "1", max: "12",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    value: "{banner_border_width}",
                                                                    class: "slider-punk-zoom",
                                                                    oninput: move |e| {
                                                                        if let Ok(v) = e.value().parse::<u32>() { banner_border_width.set(v); }
                                                                    }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_border_width();
                                                                        if curr < 12 { banner_border_width.set(curr + 1); }
                                                                    },
                                                                    "+"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                div { class: "form-row-2col", style: "margin-top: 6px;",
                                                    div {
                                                        div { class: "stepper-slider-box",
                                                            div { class: "stepper-header-row",
                                                                span { class: "stepper-title", "Ondulação: {banner_wave_amp}px" }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper-reset",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    onclick: move |_| banner_wave_amp.set(0),
                                                                    "↺ 0px"
                                                                }
                                                            }
                                                            div { class: "stepper-row",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_wave_amp();
                                                                        if curr > 0 { banner_wave_amp.set(curr.saturating_sub(2)); }
                                                                    },
                                                                    "−"
                                                                }
                                                                input {
                                                                    r#type: "range", min: "0", max: "30",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    value: "{banner_wave_amp}",
                                                                    class: "slider-punk-zoom",
                                                                    oninput: move |e| {
                                                                        if let Ok(v) = e.value().parse::<u32>() { banner_wave_amp.set(v); }
                                                                    }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_wave_amp();
                                                                        if curr < 30 { banner_wave_amp.set((curr + 2).min(30)); }
                                                                    },
                                                                    "+"
                                                                }
                                                            }
                                                        }
                                                    }
                                                    div {
                                                        div { class: "stepper-slider-box",
                                                            div { class: "stepper-header-row",
                                                                span { class: "stepper-title", "Jitter Palavras: {banner_word_rot}°" }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper-reset",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        banner_word_rot.set(0);
                                                                        banner_word_cut.set(false);
                                                                    },
                                                                    "↺ 0°"
                                                                }
                                                            }
                                                            div { class: "stepper-row",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_word_rot();
                                                                        if curr > 0 {
                                                                            let next = curr.saturating_sub(2);
                                                                            banner_word_rot.set(next);
                                                                            if next == 0 { banner_word_cut.set(false); }
                                                                        }
                                                                    },
                                                                    "−"
                                                                }
                                                                input {
                                                                    r#type: "range", min: "0", max: "20",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    value: "{banner_word_rot}",
                                                                    class: "slider-punk-zoom",
                                                                    oninput: move |e| {
                                                                        if let Ok(v) = e.value().parse::<u32>() {
                                                                            banner_word_rot.set(v);
                                                                            if v > 0 { banner_word_cut.set(true); } else { banner_word_cut.set(false); }
                                                                        }
                                                                    }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    disabled: "{banner_adapt_preset}",
                                                                    onclick: move |_| {
                                                                        let curr = banner_word_rot();
                                                                        if curr < 20 {
                                                                            banner_word_rot.set((curr + 2).min(20));
                                                                            banner_word_cut.set(true);
                                                                        }
                                                                    },
                                                                    "+"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                // Modo Fanzine / Ransom Note (Ticket #18)
                                                div { style: "margin-top: 8px; display: flex; align-items: center; gap: 6px; padding: 6px 8px; background: rgba(0, 0, 0, 0.4); border: 1px dashed #555;",
                                                    input {
                                                        r#type: "checkbox",
                                                        id: "banner-ransom-note-cb",
                                                        disabled: "{banner_adapt_preset}",
                                                        checked: "{banner_ransom_note}",
                                                        onchange: move |e| banner_ransom_note.set(e.checked()),
                                                    }
                                                    label {
                                                        r#for: "banner-ransom-note-cb",
                                                        style: "font-size: 0.72rem; color: #ff007f; font-weight: bold; cursor: pointer; text-transform: uppercase;",
                                                        "✂️ Colagem de Revista / Ransom Note (Fontes e cores alternadas por palavra)"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "control-box",
                                label { class: "section-tag", "3. ESTÉTICA ANTI-DESIGN (22 PRESETS CANÔNICOS)" }
                                div { class: "preset-carousel-container",
                                    div { class: "preset-carousel-scroll",
                                        {AestheticPreset::all().iter().enumerate().map(|(idx, p)| {
                                            let pid = p.id().to_string();
                                            let is_sel = aesthetic_id() == pid;
                                            let pal = Palette::find_preset(p.id()).unwrap_or_else(Palette::classic_punk);
                                            let colors = pal.colors.clone();
                                            let num = format!("{:02}", idx + 1);
                                            let short_title = preset_short_name(*p);
                                            let p_target = pid.clone();
                                            rsx! {
                                                button {
                                                    key: "{pid}",
                                                    r#type: "button",
                                                    class: if is_sel { "preset-chip active" } else { "preset-chip" },
                                                    onclick: move |_| aesthetic_id.set(p_target.clone()),
                                                    div { class: "preset-chip-header",
                                                        span { class: "preset-chip-num", "{num}" }
                                                        div { class: "preset-chip-dots",
                                                            {colors.iter().take(3).map(|c| {
                                                                let hex = c.to_hex();
                                                                rsx! {
                                                                    span {
                                                                        key: "{hex}",
                                                                        class: "preset-color-dot",
                                                                        style: "background-color: {hex};",
                                                                    }
                                                                }
                                                            })}
                                                        }
                                                    }
                                                    span { class: "preset-chip-name", "{short_title}" }
                                                }
                                            }
                                        })}
                                    }
                                }
                                select {
                                    class: "select-preset-punk",
                                    value: "{aesthetic_id}",
                                    onchange: move |e| aesthetic_id.set(e.value()),
                                    option { value: "punk-riot",        "01. PUNK RIOT // Claudia Wonder" }
                                    option { value: "navalha-sata",     "02. NAVALHA SATÃ // Madame Satã" }
                                    option { value: "xerox-podre",      "03. XEROX PODRE // Indianarae Siqueira" }
                                    option { value: "viagem-solitaria", "04. VIAGEM SOLITÁRIA // João W. Nery" }
                                    option { value: "lapa-noturna",     "05. LAPA NOTURNA // Linn da Quebrada" }
                                    option { value: "teatro-operario",  "06. TEATRO OPERÁRIO // Leo Moreira Sá" }
                                    option { value: "risografia",       "07. RISOGRAFIA // Erika Hilton" }
                                    option { value: "fanzine-digital",  "08. FANZINE DIGITAL // Amara Moira" }
                                    option { value: "asfalto-alerta",   "09. ASFALTO & OURO // Kátia Tapety" }
                                    option { value: "vga256",           "10. VGA 256 (13H) // Márcia Pantera" }
                                    option { value: "cga",              "11. CGA (1981) // Neon Cunha" }
                                    option { value: "old-mac-pop-art",  "12. OLD MAC POP ART // Rogéria" }
                                    option { value: "ascii-art",        "13. ASCII ART (1971) // Laerte Coutinho" }
                                    option { value: "tabloide-popular", "14. TABLÓIDE POPULAR // Luana Muniz" }
                                    option { value: "brutalismo",       "15. BRUTALISMO // Drica D'arc Meirelles" }
                                    option { value: "trash-polka",      "16. TRASH POLKA // Renata Carvalho" }
                                    option { value: "atari-gaming",     "17. ATARI GAMING // Jovanna Baby" }
                                    option { value: "mimeografo",       "18. MIMEÓGRAFO // Keila Simpson" }
                                    option { value: "xilogravura",      "19. XILOGRAVURA // Xica Manicongo" }
                                    option { value: "glitch-crt",       "20. GLITCH CRT // Robeyoncé Lima" }
                                    option { value: "punk-80-br",       "21. PUNK 80 BR // Marina Mathey" }
                                    option { value: "anarco-zine",      "22. ANARCO-ZINE // Jaqueline Gomes de Jesus" }
                                }
                                div { class: "preset-badge-summary",
                                    span { class: "preset-summary-tag", "⚡ {preset_summary}" }
                                }
                                button {
                                    r#type: "button",
                                    class: "btn-reset-preset",
                                    onclick: move |_| {
                                        banner_adapt_preset.set(true);
                                        banner_sem_faixa.set(false);
                                        recorte_adapt_preset.set(true);
                                        recorte_ativo.set(true);
                                        recorte_threshold.set(85);
                                        recorte_inverter.set(true);
                                        image_zoom.set(100);
                                        image_pos_x.set(50);
                                        image_pos_y.set(50);
                                        rotation_deg.set(0);
                                        banner_pos_x.set(50);
                                        banner_pos_y.set(85);
                                        banner_rotation.set(0);
                                        banner_font_size.set(48);
                                        banner_font_family.set(String::from("impact"));
                                        banner_fg_color.set(String::from("auto"));
                                        banner_bg_color.set(String::from("auto"));
                                        banner_effect.set(String::from("solida"));
                                        banner_border_finish.set(String::from("nenhuma"));
                                        banner_border_width.set(3);
                                        banner_wave_amp.set(0);
                                        banner_word_rot.set(0);
                                        banner_word_cut.set(false);
                                        banner_ransom_note.set(false);
                                        banner_top_adapt_preset.set(true);
                                        banner_top_pos_x.set(50);
                                        banner_top_pos_y.set(10);
                                        banner_top_rotation.set(0);
                                        banner_top_font_size.set(48);
                                        banner_top_font_family.set(String::from("impact"));
                                        banner_top_fg_color.set(String::from("auto"));
                                        banner_top_bg_color.set(String::from("auto"));
                                        banner_top_effect.set(String::from("solida"));
                                        banner_top_border_finish.set(String::from("nenhuma"));
                                        banner_top_border_width.set(3);
                                        banner_top_wave_amp.set(0);
                                        banner_top_word_rot.set(0);
                                        banner_top_word_cut.set(false);
                                        banner_top_ransom_note.set(false);
                                        image_crop_top.set(0);
                                        image_crop_bottom.set(0);
                                        image_crop_left.set(0);
                                        image_crop_right.set(0);
                                        dither_override.set(String::from("auto"));
                                        palette_override.set(String::from("auto"));
                                        show_advanced.set(false);
                                        threshold_slider.set(85);
                                        unsharp_slider.set(22);
                                        stretch_y_slider.set(15);
                                    },
                                    "↺ RESTAURAR PADRÃO DO PRESET"
                                }
                            }
                        } else if active_tab() == "intervencao" {
                            div { class: "control-box",
                                label { class: "section-tag", "4. MARCAS, DANOS & INTERVENÇÃO FÍSICA" }
                                div { class: "damage-selector-grid",
                                    {DamageEffect::all().iter().map(|dmg| {
                                        let dmg_id = dmg.as_str().to_string();
                                        let is_active = active_damages().contains(&dmg_id);
                                        let d_id = dmg_id.clone();
                                        rsx! {
                                            button {
                                                key: "{dmg_id}",
                                                class: if is_active { "btn-damage active" } else { "btn-damage" },
                                                onclick: move |_| {
                                                    let mut list = active_damages();
                                                    if list.contains(&d_id) {
                                                        list.retain(|x| x != &d_id);
                                                        // Remove seleção se era o item sendo editado
                                                        if selected_damage() == Some(d_id.clone()) {
                                                            selected_damage.set(None);
                                                        }
                                                    } else {
                                                        list.push(d_id.clone());
                                                        selected_damage.set(Some(d_id.clone()));
                                                    }
                                                    active_damages.set(list);
                                                },
                                                "{dmg.display_name()}"
                                            }
                                        }
                                    })}
                                }

                                // Subpainel de ajuste do dano selecionado
                                if !active_damages().is_empty() {
                                    div { class: "item-ajuste-panel",
                                        // Chips de seleção do dano a editar
                                        div { class: "item-ajuste-chips",
                                            {active_damages().into_iter().map(|d_str| {
                                                let d_str2 = d_str.clone();
                                                let is_selected = selected_damage() == Some(d_str.clone());
                                                rsx! {
                                                    button {
                                                        key: "{d_str}",
                                                        class: if is_selected { "chip-ajuste active" } else { "chip-ajuste" },
                                                        onclick: move |_| selected_damage.set(Some(d_str2.clone())),
                                                        "{d_str}"
                                                    }
                                                }
                                            })}
                                        }

                                        // Sliders para o dano selecionado
                                        if let Some(dmg_sel) = selected_damage() {
                                            {
                                                let params = damage_params().get(&dmg_sel).copied()
                                                    .unwrap_or_else(|| {
                                                        let def = DamageEffect::parse(&dmg_sel)
                                                            .map(DamageElement::default_for)
                                                            .unwrap_or_else(|| DamageElement::new(DamageEffect::Tiro, 0.5, 0.5, 1.0));
                                                        ((def.pos_x * 100.0).round() as u32,
                                                         (def.pos_y * 100.0).round() as u32,
                                                         100u32)
                                                    });
                                                let (px100, py100, sc100) = params;
                                                let d_x_minus = dmg_sel.clone();
                                                let d_x_input = dmg_sel.clone();
                                                let d_x_plus = dmg_sel.clone();
                                                let d_y_minus = dmg_sel.clone();
                                                let d_y_input = dmg_sel.clone();
                                                let d_y_plus = dmg_sel.clone();
                                                let d_sc_minus = dmg_sel.clone();
                                                let d_sc_input = dmg_sel.clone();
                                                let d_sc_plus = dmg_sel.clone();
                                                let d_reset = dmg_sel.clone();
                                                rsx! {
                                                    div { class: "item-ajuste-sliders",
                                                        div { class: "slider-group",
                                                            label { "POSIÇÃO X: {px100}%" }
                                                            div { class: "stepper-row",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    onclick: move |_| {
                                                                        let mut map = damage_params();
                                                                        let entry = map.entry(d_x_minus.clone()).or_insert((50, 50, 100));
                                                                        entry.0 = entry.0.saturating_sub(5);
                                                                        damage_params.set(map);
                                                                    },
                                                                    "−"
                                                                }
                                                                input {
                                                                    r#type: "range", min: "0", max: "100",
                                                                    value: "{px100}",
                                                                    class: "slider-punk-zoom",
                                                                    oninput: move |e| {
                                                                        if let Ok(v) = e.value().parse::<u32>() {
                                                                            let mut map = damage_params();
                                                                            let entry = map.entry(d_x_input.clone()).or_insert((50, 50, 100));
                                                                            entry.0 = v;
                                                                            damage_params.set(map);
                                                                        }
                                                                    }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    onclick: move |_| {
                                                                        let mut map = damage_params();
                                                                        let entry = map.entry(d_x_plus.clone()).or_insert((50, 50, 100));
                                                                        entry.0 = (entry.0 + 5).min(100);
                                                                        damage_params.set(map);
                                                                    },
                                                                    "+"
                                                                }
                                                            }
                                                        }
                                                        div { class: "slider-group",
                                                            label { "POSIÇÃO Y: {py100}%" }
                                                            div { class: "stepper-row",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    onclick: move |_| {
                                                                        let mut map = damage_params();
                                                                        let entry = map.entry(d_y_minus.clone()).or_insert((50, 50, 100));
                                                                        entry.1 = entry.1.saturating_sub(5);
                                                                        damage_params.set(map);
                                                                    },
                                                                    "−"
                                                                }
                                                                input {
                                                                    r#type: "range", min: "0", max: "100",
                                                                    value: "{py100}",
                                                                    class: "slider-punk-zoom",
                                                                    oninput: move |e| {
                                                                        if let Ok(v) = e.value().parse::<u32>() {
                                                                            let mut map = damage_params();
                                                                            let entry = map.entry(d_y_input.clone()).or_insert((50, 50, 100));
                                                                            entry.1 = v;
                                                                            damage_params.set(map);
                                                                        }
                                                                    }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    onclick: move |_| {
                                                                        let mut map = damage_params();
                                                                        let entry = map.entry(d_y_plus.clone()).or_insert((50, 50, 100));
                                                                        entry.1 = (entry.1 + 5).min(100);
                                                                        damage_params.set(map);
                                                                    },
                                                                    "+"
                                                                }
                                                            }
                                                        }
                                                        div { class: "slider-group",
                                                            label { "ESCALA: {sc100}% ({sc100 as f32 / 100.0:.1}x)" }
                                                            div { class: "stepper-row",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    onclick: move |_| {
                                                                        let mut map = damage_params();
                                                                        let entry = map.entry(d_sc_minus.clone()).or_insert((50, 50, 100));
                                                                        entry.2 = entry.2.saturating_sub(10).max(25);
                                                                        damage_params.set(map);
                                                                    },
                                                                    "−"
                                                                }
                                                                input {
                                                                    r#type: "range", min: "25", max: "400",
                                                                    value: "{sc100}",
                                                                    class: "slider-punk-zoom",
                                                                    oninput: move |e| {
                                                                        if let Ok(v) = e.value().parse::<u32>() {
                                                                            let mut map = damage_params();
                                                                            let entry = map.entry(d_sc_input.clone()).or_insert((50, 50, 100));
                                                                            entry.2 = v;
                                                                            damage_params.set(map);
                                                                        }
                                                                    }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    onclick: move |_| {
                                                                        let mut map = damage_params();
                                                                        let entry = map.entry(d_sc_plus.clone()).or_insert((50, 50, 100));
                                                                        entry.2 = (entry.2 + 10).min(400);
                                                                        damage_params.set(map);
                                                                    },
                                                                    "+"
                                                                }
                                                            }
                                                        }
                                                        button {
                                                            class: "btn-edit-action",
                                                            style: "margin-top:4px;",
                                                            onclick: move |_| {
                                                                let mut map = damage_params();
                                                                map.remove(&d_reset);
                                                                damage_params.set(map);
                                                            },
                                                            "↺ RESETAR POSIÇÃO"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "control-box",
                                label { class: "section-tag", "5. STICKERS & ADESIVOS PUNK" }
                                div { class: "damage-selector-grid",
                                    {StickerKind::all().iter().map(|stk| {
                                        let stk_id = stk.as_str().to_string();
                                        let is_active = active_stickers().contains(&stk_id);
                                        let s_id = stk_id.clone();
                                        rsx! {
                                            button {
                                                key: "{stk_id}",
                                                class: if is_active { "btn-damage active" } else { "btn-damage" },
                                                onclick: move |_| {
                                                    let mut list = active_stickers();
                                                    if list.contains(&s_id) {
                                                        list.retain(|x| x != &s_id);
                                                        if selected_sticker() == Some(s_id.clone()) {
                                                            selected_sticker.set(None);
                                                        }
                                                    } else {
                                                        list.push(s_id.clone());
                                                        selected_sticker.set(Some(s_id.clone()));
                                                    }
                                                    active_stickers.set(list);
                                                },
                                                "{stk.display_name()}"
                                            }
                                        }
                                    })}
                                }
                                div { style: "margin-top: 8px;",
                                    input {
                                        id: "custom-sticker-uploader",
                                        r#type: "file",
                                        accept: "image/*",
                                        style: "display: none;",
                                        onchange: on_custom_sticker_upload,
                                    }
                                    button {
                                        class: if custom_sticker_image().is_some() { "btn-edit-action active" } else { "btn-edit-action" },
                                        style: "width: 100%; border: 1px dashed var(--accent-red, #ff0033);",
                                        onclick: move |_| {
                                            let window = web_sys::window().unwrap();
                                            let document = window.document().unwrap();
                                            if let Some(input) = document.get_element_by_id("custom-sticker-uploader") {
                                                let input_el: HtmlInputElement = input.unchecked_into();
                                                input_el.click();
                                            }
                                        },
                                        if custom_sticker_image().is_some() { "📎 SUBSTITUIR RECORTE PRÓPRIO" } else { "📎 CARREGAR RECORTE / STICKER PRÓPRIO" }
                                    }
                                }
                                div { style: "margin-top: 8px;",
                                    button {
                                        class: if filter_stickers() { "btn-edit-action active" } else { "btn-edit-action" },
                                        style: "width: 100%;",
                                        onclick: move |_| {
                                            let curr = filter_stickers();
                                            filter_stickers.set(!curr);
                                        },
                                        if filter_stickers() { "🎨 PALETA ATIVA NOS STICKERS" } else { "🌈 CORES ORIGINAIS DOS STICKERS" }
                                    }
                                }

                                // Subpainel de ajuste do sticker selecionado
                                if !active_stickers().is_empty() {
                                    div { class: "item-ajuste-panel",
                                        div { class: "item-ajuste-chips",
                                            {active_stickers().into_iter().map(|s_str| {
                                                let s_str2 = s_str.clone();
                                                let is_selected = selected_sticker() == Some(s_str.clone());
                                                rsx! {
                                                    button {
                                                        key: "{s_str}",
                                                        class: if is_selected { "chip-ajuste active" } else { "chip-ajuste" },
                                                        onclick: move |_| selected_sticker.set(Some(s_str2.clone())),
                                                        "{s_str}"
                                                    }
                                                }
                                            })}
                                        }

                                        if let Some(stk_sel) = selected_sticker() {
                                            {
                                                let (px100, py100, sz, rot) = sticker_params().get(&stk_sel).copied()
                                                    .unwrap_or((50u32, 50u32, 100u32, 0i32));
                                                let (b_style, b_width) = sticker_borders().get(&stk_sel).cloned()
                                                    .unwrap_or_else(|| (String::from("diecut"), 4u32));
                                                let s_x_minus = stk_sel.clone();
                                                let s_x_input = stk_sel.clone();
                                                let s_x_plus = stk_sel.clone();
                                                let s_y_minus = stk_sel.clone();
                                                let s_y_input = stk_sel.clone();
                                                let s_y_plus = stk_sel.clone();
                                                let s_sz_minus = stk_sel.clone();
                                                let s_sz_input = stk_sel.clone();
                                                let s_sz_plus = stk_sel.clone();
                                                let s_rot_minus = stk_sel.clone();
                                                let s_rot_input = stk_sel.clone();
                                                let s_rot_plus = stk_sel.clone();
                                                let s_style = stk_sel.clone();
                                                let s_bw_minus = stk_sel.clone();
                                                let s_bw_input = stk_sel.clone();
                                                let s_bw_plus = stk_sel.clone();
                                                let s_reset_p = stk_sel.clone();
                                                let s_reset_b = stk_sel.clone();
                                                rsx! {
                                                    div { class: "item-ajuste-sliders",
                                                        div { class: "slider-group",
                                                            label { "POSIÇÃO X: {px100}%" }
                                                            div { class: "stepper-row",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    onclick: move |_| {
                                                                        let mut map = sticker_params();
                                                                        let entry = map.entry(s_x_minus.clone()).or_insert((50,50,100,0));
                                                                        entry.0 = entry.0.saturating_sub(5);
                                                                        sticker_params.set(map);
                                                                    },
                                                                    "−"
                                                                }
                                                                input {
                                                                    r#type: "range", min: "0", max: "100",
                                                                    value: "{px100}",
                                                                    class: "slider-punk-zoom",
                                                                    oninput: move |e| {
                                                                        if let Ok(v) = e.value().parse::<u32>() {
                                                                            let mut map = sticker_params();
                                                                            let entry = map.entry(s_x_input.clone()).or_insert((50,50,100,0));
                                                                            entry.0 = v;
                                                                            sticker_params.set(map);
                                                                        }
                                                                    }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    onclick: move |_| {
                                                                        let mut map = sticker_params();
                                                                        let entry = map.entry(s_x_plus.clone()).or_insert((50,50,100,0));
                                                                        entry.0 = (entry.0 + 5).min(100);
                                                                        sticker_params.set(map);
                                                                    },
                                                                    "+"
                                                                }
                                                            }
                                                        }
                                                        div { class: "slider-group",
                                                            label { "POSIÇÃO Y: {py100}%" }
                                                            div { class: "stepper-row",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    onclick: move |_| {
                                                                        let mut map = sticker_params();
                                                                        let entry = map.entry(s_y_minus.clone()).or_insert((50,50,100,0));
                                                                        entry.1 = entry.1.saturating_sub(5);
                                                                        sticker_params.set(map);
                                                                    },
                                                                    "−"
                                                                }
                                                                input {
                                                                    r#type: "range", min: "0", max: "100",
                                                                    value: "{py100}",
                                                                    class: "slider-punk-zoom",
                                                                    oninput: move |e| {
                                                                        if let Ok(v) = e.value().parse::<u32>() {
                                                                            let mut map = sticker_params();
                                                                            let entry = map.entry(s_y_input.clone()).or_insert((50,50,100,0));
                                                                            entry.1 = v;
                                                                            sticker_params.set(map);
                                                                        }
                                                                    }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    onclick: move |_| {
                                                                        let mut map = sticker_params();
                                                                        let entry = map.entry(s_y_plus.clone()).or_insert((50,50,100,0));
                                                                        entry.1 = (entry.1 + 5).min(100);
                                                                        sticker_params.set(map);
                                                                    },
                                                                    "+"
                                                                }
                                                            }
                                                        }
                                                        div { class: "slider-group",
                                                            label { "TAMANHO: {sz}px" }
                                                            div { class: "stepper-row",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    onclick: move |_| {
                                                                        let mut map = sticker_params();
                                                                        let entry = map.entry(s_sz_minus.clone()).or_insert((50,50,100,0));
                                                                        entry.2 = entry.2.saturating_sub(10).max(40);
                                                                        sticker_params.set(map);
                                                                    },
                                                                    "−"
                                                                }
                                                                input {
                                                                    r#type: "range", min: "40", max: "240",
                                                                    value: "{sz}",
                                                                    class: "slider-punk-zoom",
                                                                    oninput: move |e| {
                                                                        if let Ok(v) = e.value().parse::<u32>() {
                                                                            let mut map = sticker_params();
                                                                            let entry = map.entry(s_sz_input.clone()).or_insert((50,50,100,0));
                                                                            entry.2 = v;
                                                                            sticker_params.set(map);
                                                                        }
                                                                    }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    onclick: move |_| {
                                                                        let mut map = sticker_params();
                                                                        let entry = map.entry(s_sz_plus.clone()).or_insert((50,50,100,0));
                                                                        entry.2 = (entry.2 + 10).min(240);
                                                                        sticker_params.set(map);
                                                                    },
                                                                    "+"
                                                                }
                                                            }
                                                        }
                                                        div { class: "slider-group",
                                                            label { "ROTAÇÃO: {rot}°" }
                                                            div { class: "stepper-row",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    onclick: move |_| {
                                                                        let mut map = sticker_params();
                                                                        let entry = map.entry(s_rot_minus.clone()).or_insert((50,50,100,0));
                                                                        entry.3 = (entry.3 - 15).max(-180);
                                                                        sticker_params.set(map);
                                                                    },
                                                                    "−"
                                                                }
                                                                input {
                                                                    r#type: "range", min: "-180", max: "180",
                                                                    value: "{rot}",
                                                                    class: "slider-punk-zoom",
                                                                    oninput: move |e| {
                                                                        if let Ok(v) = e.value().parse::<i32>() {
                                                                            let mut map = sticker_params();
                                                                            let entry = map.entry(s_rot_input.clone()).or_insert((50,50,100,0));
                                                                            entry.3 = v;
                                                                            sticker_params.set(map);
                                                                        }
                                                                    }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "btn-stepper",
                                                                    onclick: move |_| {
                                                                        let mut map = sticker_params();
                                                                        let entry = map.entry(s_rot_plus.clone()).or_insert((50,50,100,0));
                                                                        entry.3 = (entry.3 + 15).min(180);
                                                                        sticker_params.set(map);
                                                                    },
                                                                    "+"
                                                                }
                                                            }
                                                        }
                                                        div { class: "slider-group",
                                                            label { "BORDA / RECORTE:" }
                                                            select {
                                                                class: "form-control",
                                                                value: "{b_style}",
                                                                onchange: move |e| {
                                                                    let mut map = sticker_borders();
                                                                    let entry = map.entry(s_style.clone()).or_insert((String::from("diecut"), 4));
                                                                    entry.0 = e.value();
                                                                    sticker_borders.set(map);
                                                                },
                                                                option { value: "diecut", "DIECUT BRANCO (ADESIVO)" }
                                                                option { value: "picote", "PICOTE PERFURADO" }
                                                                option { value: "serrilhada", "SERRILHADA" }
                                                                option { value: "preta", "LINHA PRETA" }
                                                                option { value: "nenhuma", "SEM BORDA" }
                                                            }
                                                        }
                                                        if b_style != "nenhuma" {
                                                            div { class: "slider-group",
                                                                label { "LARGURA DA BORDA: {b_width}px" }
                                                                div { class: "stepper-row",
                                                                    button {
                                                                        r#type: "button",
                                                                        class: "btn-stepper",
                                                                        onclick: move |_| {
                                                                            let mut map = sticker_borders();
                                                                            let entry = map.entry(s_bw_minus.clone()).or_insert((String::from("diecut"), 4));
                                                                            entry.1 = entry.1.saturating_sub(1).max(1);
                                                                            sticker_borders.set(map);
                                                                        },
                                                                        "−"
                                                                    }
                                                                    input {
                                                                        r#type: "range", min: "1", max: "16",
                                                                        value: "{b_width}",
                                                                        class: "slider-punk-zoom",
                                                                        oninput: move |e| {
                                                                            if let Ok(v) = e.value().parse::<u32>() {
                                                                                let mut map = sticker_borders();
                                                                                let entry = map.entry(s_bw_input.clone()).or_insert((String::from("diecut"), 4));
                                                                                entry.1 = v;
                                                                                sticker_borders.set(map);
                                                                            }
                                                                        }
                                                                    }
                                                                    button {
                                                                        r#type: "button",
                                                                        class: "btn-stepper",
                                                                        onclick: move |_| {
                                                                            let mut map = sticker_borders();
                                                                            let entry = map.entry(s_bw_plus.clone()).or_insert((String::from("diecut"), 4));
                                                                            entry.1 = (entry.1 + 1).min(16);
                                                                            sticker_borders.set(map);
                                                                        },
                                                                        "+"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        button {
                                                            class: "btn-edit-action",
                                                            style: "margin-top:4px;",
                                                            onclick: move |_| {
                                                                let mut map = sticker_params();
                                                                map.remove(&s_reset_p);
                                                                sticker_params.set(map);
                                                                let mut bmap = sticker_borders();
                                                                bmap.remove(&s_reset_b);
                                                                sticker_borders.set(bmap);
                                                            },
                                                            "↺ RESETAR AJUSTES"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "control-box",
                                label { class: "section-tag", "6. CAIXAS DE TEXTO EXTRA" }
                                div { class: "texto-extra-form",
                                    textarea {
                                        class: "text-input-punk",
                                        rows: "2",
                                        style: "resize: vertical; font-family: inherit;",
                                        value: "{form_texto}",
                                        placeholder: "TEXTO DA CAIXA (SUPORTA QUEBRA DE LINHA)...",
                                        oninput: move |e| form_texto.set(e.value()),
                                    }

                                    div { class: "texto-extra-sliders",
                                        div { class: "slider-group",
                                            div { class: "stepper-header-row",
                                                span { class: "stepper-title", "POSIÇÃO X: {form_pos_x}%" }
                                                button {
                                                    r#type: "button",
                                                    class: "btn-stepper-reset",
                                                    onclick: move |_| form_pos_x.set(50),
                                                    "↺ 50%"
                                                }
                                            }
                                            div { class: "stepper-row",
                                                button {
                                                    r#type: "button",
                                                    class: "btn-stepper",
                                                    onclick: move |_| {
                                                        let curr = form_pos_x();
                                                        if curr > 0 { form_pos_x.set(curr.saturating_sub(5)); }
                                                    },
                                                    "−"
                                                }
                                                input {
                                                    r#type: "range", min: "0", max: "100",
                                                    value: "{form_pos_x}",
                                                    class: "slider-punk-zoom",
                                                    oninput: move |e| {
                                                        if let Ok(v) = e.value().parse::<u32>() { form_pos_x.set(v); }
                                                    }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "btn-stepper",
                                                    onclick: move |_| {
                                                        let curr = form_pos_x();
                                                        if curr < 100 { form_pos_x.set((curr + 5).min(100)); }
                                                    },
                                                    "+"
                                                }
                                            }
                                        }
                                        div { class: "slider-group",
                                            div { class: "stepper-header-row",
                                                span { class: "stepper-title", "POSIÇÃO Y: {form_pos_y}%" }
                                                button {
                                                    r#type: "button",
                                                    class: "btn-stepper-reset",
                                                    onclick: move |_| form_pos_y.set(50),
                                                    "↺ 50%"
                                                }
                                            }
                                            div { class: "stepper-row",
                                                button {
                                                    r#type: "button",
                                                    class: "btn-stepper",
                                                    onclick: move |_| {
                                                        let curr = form_pos_y();
                                                        if curr > 0 { form_pos_y.set(curr.saturating_sub(5)); }
                                                    },
                                                    "−"
                                                }
                                                input {
                                                    r#type: "range", min: "0", max: "100",
                                                    value: "{form_pos_y}",
                                                    class: "slider-punk-zoom",
                                                    oninput: move |e| {
                                                        if let Ok(v) = e.value().parse::<u32>() { form_pos_y.set(v); }
                                                    }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "btn-stepper",
                                                    onclick: move |_| {
                                                        let curr = form_pos_y();
                                                        if curr < 100 { form_pos_y.set((curr + 5).min(100)); }
                                                    },
                                                    "+"
                                                }
                                            }
                                        }
                                        div { class: "slider-group",
                                            div { class: "stepper-header-row",
                                                span { class: "stepper-title", "TAMANHO: {form_tamanho}px" }
                                                button {
                                                    r#type: "button",
                                                    class: "btn-stepper-reset",
                                                    onclick: move |_| form_tamanho.set(48),
                                                    "↺ 48px"
                                                }
                                            }
                                            div { class: "stepper-row",
                                                button {
                                                    r#type: "button",
                                                    class: "btn-stepper",
                                                    onclick: move |_| {
                                                        let curr = form_tamanho();
                                                        if curr > 20 { form_tamanho.set(curr.saturating_sub(4).max(20)); }
                                                    },
                                                    "−"
                                                }
                                                input {
                                                    r#type: "range", min: "20", max: "120",
                                                    value: "{form_tamanho}",
                                                    class: "slider-punk-zoom",
                                                    oninput: move |e| {
                                                        if let Ok(v) = e.value().parse::<u32>() { form_tamanho.set(v); }
                                                    }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "btn-stepper",
                                                    onclick: move |_| {
                                                        let curr = form_tamanho();
                                                        if curr < 120 { form_tamanho.set((curr + 4).min(120)); }
                                                    },
                                                    "+"
                                                }
                                            }
                                        }
                                    }

                                    div { class: "texto-extra-controls",
                                        div { class: "select-group",
                                            label { "INCLINAÇÃO" }
                                            select {
                                                value: "{form_incl}",
                                                onchange: move |e| {
                                                    if let Ok(v) = e.value().parse::<i32>() { form_incl.set(v); }
                                                },
                                                option { value: "0",   "0° (Reto)" }
                                                option { value: "-5",  "-5° (Leve esquerda)" }
                                                option { value: "5",   "+5° (Leve direita)" }
                                                option { value: "-10", "-10° (Inclinado)" }
                                                option { value: "10",  "+10° (Inclinado)" }
                                                option { value: "-15", "-15° (Agressivo)" }
                                                option { value: "15",  "+15° (Agressivo)" }
                                            }
                                        }
                                        div { class: "select-group",
                                            label { "ACABAMENTO" }
                                            select {
                                                value: "{form_acabamento}",
                                                onchange: move |e| form_acabamento.set(e.value()),
                                                option { value: "tarja",      "TARJA (clássico punk)" }
                                                option { value: "xerox",      "XEROX (falhas de toner e riscos)" }
                                                option { value: "tape",       "TAPE (fita adesiva cinza)" }
                                                option { value: "serrilhado", "SERRILHADO (neon amarelo)" }
                                                option { value: "rasgo",      "RASGO (texto nu vermelho)" }
                                            }
                                        }
                                    }

                                    div { style: "display: flex; gap: 8px; margin-top: 8px;",
                                        button {
                                            class: "btn-gilete",
                                            style: "flex: 1; font-size: 0.85rem;",
                                            onclick: move |_| {
                                                let txt = form_texto().trim().to_string();
                                                if txt.is_empty() { return; }
                                                let mut lista = extra_texts();
                                                let entry = (
                                                    txt.clone(),
                                                    form_pos_x(),
                                                    form_pos_y(),
                                                    form_tamanho(),
                                                    form_incl(),
                                                    form_acabamento(),
                                                );
                                                if let Some(idx) = form_editando() {
                                                    if idx < lista.len() { lista[idx] = entry; }
                                                } else {
                                                    lista.push(entry);
                                                }
                                                extra_texts.set(lista);
                                                form_texto.set(String::new());
                                                form_pos_x.set(50);
                                                form_pos_y.set(50);
                                                form_tamanho.set(48);
                                                form_incl.set(0);
                                                form_acabamento.set(String::from("tarja"));
                                                form_editando.set(None);
                                            },
                                            if form_editando().is_some() { "✓ SALVAR EDIÇÃO" } else { "+ INSERIR CAIXA DE TEXTO" }
                                        }
                                        if form_editando().is_some() || !form_texto().trim().is_empty() {
                                            button {
                                                class: "btn-edit-action",
                                                style: "font-size: 0.85rem; padding: 0 12px; background: rgba(255, 0, 51, 0.15); border-color: #ff0033;",
                                                onclick: move |_| {
                                                    form_texto.set(String::new());
                                                    form_pos_x.set(50);
                                                    form_pos_y.set(50);
                                                    form_tamanho.set(48);
                                                    form_incl.set(0);
                                                    form_acabamento.set(String::from("tarja"));
                                                    form_editando.set(None);
                                                },
                                                "✕ LIMPAR"
                                            }
                                        }
                                    }
                                }

                                if !extra_texts().is_empty() {
                                    div { class: "texto-extra-lista",
                                        {let textos_lista = extra_texts();
                                        textos_lista.into_iter().enumerate().map(|(idx, (txt, px, py, tam, incl, acab))| {
                                            let label = format!("[{}] {} — {}% × {}% / {}px / {}° / {}",
                                                idx + 1, &txt, px, py, tam, incl, &acab);
                                            let txt_c  = txt.clone();
                                            let acab_c = acab.clone();
                                            rsx! {
                                                div { key: "{idx}", class: "texto-extra-item",
                                                    span { class: "texto-extra-label", "{label}" }
                                                    div { class: "texto-extra-actions",
                                                        button {
                                                            class: "btn-edit-action",
                                                            onclick: move |_| {
                                                                form_texto.set(txt_c.clone());
                                                                form_pos_x.set(px);
                                                                form_pos_y.set(py);
                                                                form_tamanho.set(tam);
                                                                form_incl.set(incl);
                                                                form_acabamento.set(acab_c.clone());
                                                                form_editando.set(Some(idx));
                                                            },
                                                            "✏️ EDITAR"
                                                        }
                                                        button {
                                                            class: "btn-edit-action",
                                                            style: "color: #ff0033;",
                                                            onclick: move |_| {
                                                                let mut lista = extra_texts();
                                                                if idx < lista.len() { lista.remove(idx); }
                                                                extra_texts.set(lista);
                                                                if form_editando() == Some(idx) {
                                                                    form_editando.set(None);
                                                                }
                                                            },
                                                            "✕ REMOVER"
                                                        }
                                                    }
                                                }
                                            }
                                        })}
                                    }
                                }
                            }
                        } else {
                            // Aba de Ajustes Finos
                            div { class: "advanced-panel",
                                button {
                                    class: if show_advanced() { "btn-edit-action active" } else { "btn-edit-action" },
                                    style: "width: 100%; margin-bottom: 8px;",
                                    onclick: move |_| {
                                        let curr = show_advanced();
                                        show_advanced.set(!curr);
                                    },
                                    if show_advanced() { "✓ AJUSTES DA PRENSA ATIVOS (SOBREPÕE O PRESET)" } else { "⚙️ ATIVAR CONTROLES MANUAIS DA PRENSA" }
                                }

                                div { class: "slider-group",
                                    div { class: "stepper-header-row",
                                        span { class: "stepper-title", "CORTE GUILHOTINA: {threshold_slider}%" }
                                        button {
                                            r#type: "button",
                                            class: "btn-stepper-reset",
                                            onclick: move |_| threshold_slider.set(85),
                                            "↺ 85%"
                                        }
                                    }
                                    div { class: "stepper-row",
                                        button {
                                            r#type: "button",
                                            class: "btn-stepper",
                                            onclick: move |_| {
                                                let curr = threshold_slider();
                                                if curr > 30 { threshold_slider.set(curr.saturating_sub(2).max(30)); }
                                            },
                                            "−"
                                        }
                                        input {
                                            r#type: "range",
                                            min: "30",
                                            max: "99",
                                            value: "{threshold_slider}",
                                            class: "slider-punk-zoom",
                                            oninput: move |e| {
                                                if let Ok(v) = e.value().parse::<u32>() {
                                                    threshold_slider.set(v);
                                                }
                                            }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "btn-stepper",
                                            onclick: move |_| {
                                                let curr = threshold_slider();
                                                if curr < 99 { threshold_slider.set((curr + 2).min(99)); }
                                            },
                                            "+"
                                        }
                                    }
                                }

                                div { class: "slider-group",
                                    div { class: "stepper-header-row",
                                        span { class: "stepper-title", "NITIDEZ UNSHARP: {unsharp_display}" }
                                        button {
                                            r#type: "button",
                                            class: "btn-stepper-reset",
                                            onclick: move |_| unsharp_slider.set(22),
                                            "↺ 2.2x"
                                        }
                                    }
                                    div { class: "stepper-row",
                                        button {
                                            r#type: "button",
                                            class: "btn-stepper",
                                            onclick: move |_| {
                                                let curr = unsharp_slider();
                                                if curr > 0 { unsharp_slider.set(curr.saturating_sub(2)); }
                                            },
                                            "−"
                                        }
                                        input {
                                            r#type: "range",
                                            min: "0",
                                            max: "50",
                                            value: "{unsharp_slider}",
                                            class: "slider-punk-zoom",
                                            oninput: move |e| {
                                                if let Ok(v) = e.value().parse::<u32>() {
                                                    unsharp_slider.set(v);
                                                }
                                            }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "btn-stepper",
                                            onclick: move |_| {
                                                let curr = unsharp_slider();
                                                if curr < 50 { unsharp_slider.set((curr + 2).min(50)); }
                                            },
                                            "+"
                                        }
                                    }
                                }

                                div { class: "slider-group",
                                    div { class: "stepper-header-row",
                                        span { class: "stepper-title", "DISTORÇÃO TIPOGRAFIA: {stretch_y_display}" }
                                        button {
                                            r#type: "button",
                                            class: "btn-stepper-reset",
                                            onclick: move |_| stretch_y_slider.set(15),
                                            "↺ 1.5x"
                                        }
                                    }
                                    div { class: "stepper-row",
                                        button {
                                            r#type: "button",
                                            class: "btn-stepper",
                                            onclick: move |_| {
                                                let curr = stretch_y_slider();
                                                if curr > 10 { stretch_y_slider.set(curr.saturating_sub(1).max(10)); }
                                            },
                                            "−"
                                        }
                                        input {
                                            r#type: "range",
                                            min: "10",
                                            max: "30",
                                            value: "{stretch_y_slider}",
                                            class: "slider-punk-zoom",
                                            oninput: move |e| {
                                                if let Ok(v) = e.value().parse::<u32>() {
                                                    stretch_y_slider.set(v);
                                                }
                                            }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "btn-stepper",
                                            onclick: move |_| {
                                                let curr = stretch_y_slider();
                                                if curr < 30 { stretch_y_slider.set((curr + 1).min(30)); }
                                            },
                                            "+"
                                        }
                                    }
                                }

                                div { class: "select-group",
                                    label { "ALGORITMO DE DITHER" }
                                    select {
                                        value: "{dither_override}",
                                        onchange: move |e| dither_override.set(e.value()),
                                        option { value: "auto", "Automático do Preset" }
                                        option { value: "atkinson", "Atkinson (75% Dispersão)" }
                                        option { value: "floyd-steinberg", "Floyd-Steinberg (Serpentina)" }
                                        option { value: "bayer8x8", "Bayer 8x8 (Digital Retrô)" }
                                        option { value: "bayer4x4", "Bayer 4x4" }
                                        option { value: "halftone", "Halftone Analógico 45°" }
                                    }
                                }

                                div { class: "select-group",
                                    label { "PALETA CROMÁTICA" }
                                    select {
                                        value: "{palette_override}",
                                        onchange: move |e| palette_override.set(e.value()),
                                        option { value: "auto", "Automático do Preset" }
                                        option { value: "classic-punk", "Navalha Clássica (#000, #FFF, #FF0033)" }
                                        option { value: "lapa-noturna", "Lapa Noturna (#0B0B0E, #FF007F, #00F0FF)" }
                                        option { value: "xerox-podre", "Xerox Podre (#1C1917, #F5F2EB)" }
                                        option { value: "risografia", "Risografia Crua (#0038A8, #FF48B0, #FFF200)" }
                                        option { value: "asfalto", "Asfalto & Ouro (#111111, #FFCC00)" }
                                        option { value: "vga256", "VGA 256 / Modo 13h (256 cores)" }
                                    }
                                }

                                div { class: "select-group",
                                    label { "AMPLIAÇÃO NEAREST-NEIGHBOR" }
                                    select {
                                        value: "{scale_select}",
                                        onchange: move |e| scale_select.set(e.value()),
                                        option { value: "1x", "1x (Resolução Nativa de Grão)" }
                                        option { value: "2x", "2x (Padrão Nítido)" }
                                        option { value: "3x", "3x (Super Tátil)" }
                                        option { value: "4x", "4x (Poster Gigante)" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Área do Canvas à direita (no mobile compactada no topo em modo controls, ou fullscreen)
                section {
                    class: if mobile_view() == "controls" && preview_collapsed() {
                        "panel-preview mobile-collapsed"
                    } else if mobile_view() == "controls" {
                        "panel-preview mobile-compact"
                    } else {
                        "panel-preview mobile-fullscreen"
                    },

                    if mobile_view() == "controls" && preview_collapsed() {
                        div { class: "mobile-collapsed-bar",
                            button {
                                class: "btn-expand-bar",
                                onclick: move |_| preview_collapsed.set(false),
                                "👁️ MOSTRAR CARTAZ AO VIVO (EXPANDIR) ▼"
                            }
                        }
                    } else {
                        // Barra superior do preview compacto no mobile
                        if mobile_view() == "controls" {
                            div { class: "mobile-compact-bar",
                                span { class: "mobile-compact-label", "👁️ CARTAZ AO VIVO" }
                                div { class: "mobile-compact-actions",
                                    button {
                                        class: "btn-compact-opt",
                                        onclick: move |_| preview_collapsed.set(true),
                                        title: "Ocultar preview para ganhar espaço",
                                        "▲ OCULTAR"
                                    }
                                    button {
                                        class: "btn-compact-opt active",
                                        onclick: move |_| mobile_view.set("preview".into()),
                                        title: "Expandir para tela cheia",
                                        "⤢ AMPLIAR"
                                    }
                                }
                            }
                        } else {
                            // Ações de preview no mobile fullscreen
                            div { class: "mobile-preview-actions",
                                button {
                                    class: "btn-edit-action",
                                    onclick: move |_| mobile_view.set("controls".into()),
                                    "⚡ VOLTAR AOS CONTROLES"
                                }
                                button {
                                    class: "btn-download-sm",
                                    onclick: download_png_preview,
                                    "💾 BAIXAR PÔSTER (PNG)"
                                }
                            }
                        }

                        div { class: "canvas-wrapper",
                            if is_processing() {
                                div {
                                    class: "processing-indicator",
                                    "[ RENDERIZANDO... ]"
                                }
                            }
                            if rendered_png_uri().is_empty() {
                                div {
                                    style: "color: #ff0033; font-family: monospace; font-size: 0.9rem; padding: 40px; text-align: center;",
                                    "[ INICIANDO MOTOR... ]"
                                }
                            } else {
                                img {
                                    class: if is_processing() { "poster-display dim" } else { "poster-display" },
                                    src: "{rendered_png_uri}",
                                    alt: "Pôster Navalha gerado em tempo real",
                                    onclick: move |_| {
                                        if mobile_view() == "controls" {
                                            mobile_view.set("preview".into());
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }

            // Doca flutuante inferior no mobile
            div { class: "mobile-bottom-dock",
                if mobile_view() == "controls" {
                    button {
                        class: "dock-btn secondary",
                        onclick: move |_| mobile_view.set("preview".into()),
                        "⤢ TELA CHEIA"
                    }
                } else {
                    button {
                        class: "dock-btn secondary",
                        onclick: move |_| mobile_view.set("controls".into()),
                        "⚡ VOLTAR A EDITAR"
                    }
                }
                button {
                    class: "dock-btn primary",
                    onclick: download_png_dock,
                    "💾 BAIXAR PÔSTER (PNG)"
                }
            }

            // Rodapé com linha de comando em tempo real
            footer { class: "navalha-footer",
                div { class: "cli-mirror-box",
                    span { class: "cli-prompt", "$" }
                    code { class: "cli-code", "{cli_cmd}" }
                    button {
                        class: "btn-copy-cli",
                        onclick: copy_cmd,
                        "{copy_status}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_collection_count() {
        assert_eq!(BACKGROUND_COLLECTION.len(), 20, "O catálogo de backgrounds deve conter exatamente 20 imagens CC");
    }

    #[test]
    fn test_background_collection_unique() {
        let mut paths = std::collections::HashSet::new();
        let mut names = std::collections::HashSet::new();
        for (path, name) in BACKGROUND_COLLECTION {
            assert!(paths.insert(path), "Caminho duplicado no catálogo: {}", path);
            assert!(names.insert(name), "Nome de personalidade duplicado: {}", name);
        }
    }

    #[test]
    fn test_background_files_exist_on_disk() {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for (rel_path, name) in BACKGROUND_COLLECTION {
            let full_path = manifest_dir.join(rel_path);
            assert!(
                full_path.exists(),
                "Arquivo de background não encontrado em disco para {}: {:?}",
                name,
                full_path
            );
            let metadata = std::fs::metadata(&full_path).unwrap();
            assert!(metadata.len() > 1024, "Arquivo de imagem vazio ou corrompido: {:?}", full_path);
        }
    }

    #[test]
    fn test_pick_random_background_index_bounds() {
        let idx = pick_random_background_index();
        assert!(idx < BACKGROUND_COLLECTION.len());
    }
}


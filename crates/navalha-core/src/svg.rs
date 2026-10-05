use crate::press::PosterComposition;

pub fn generate_poster_svg(composition: &PosterComposition, image_base64_png: Option<&str>) -> String {
    let w = composition.canvas_width;
    let h = composition.canvas_height;
    let bg = composition.background_color.to_hex();

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" style="background-color: {bg};">
  <rect width="100%" height="100%" fill="{bg}" />
"#
    );

    // Camada 1: Texto de fundo
    if let Some(ref bg_text) = composition.background_text {
        let col = bg_text.color.to_hex();
        let fs = bg_text.font_size * bg_text.stretch_y;
        let scale_x = bg_text.stretch_x;
        let tx = if bg_text.pos_x <= 1.0 { bg_text.pos_x * w as f32 } else { bg_text.pos_x };
        let ty = if bg_text.pos_y <= 1.0 { bg_text.pos_y * h as f32 } else { bg_text.pos_y };
        let anchor = if bg_text.center_x { "middle" } else { "start" };

        svg.push_str(&format!(
            r#"  <text x="{tx}" y="{ty}" fill="{col}" font-family="Impact, 'Bebas Neue', sans-serif" font-weight="900" font-size="{fs}" text-anchor="{anchor}" transform="scale({scale_x}, 1.0)">{}</text>
"#,
            escape_xml(&bg_text.text)
        ));
    }

    // Camada 2: Imagem do sujeito embutida como PNG data URI
    if let Some(png_b64) = image_base64_png {
        svg.push_str(&format!(
            r#"  <image href="data:image/png;base64,{png_b64}" x="0" y="0" width="{w}" height="{h}" preserveAspectRatio="xMidYMid meet" style="image-rendering: pixelated;" />
"#
        ));
    }

    // Camada 3: Textos em tarja
    for fg_text in &composition.foreground_texts {
        let tx = if fg_text.pos_x <= 1.0 { fg_text.pos_x * w as f32 } else { fg_text.pos_x };
        let ty = if fg_text.pos_y <= 1.0 { fg_text.pos_y * h as f32 } else { fg_text.pos_y };
        let fs = fg_text.font_size * fg_text.stretch_y;
        let text_col = fg_text.color.to_hex();

        let raw_lines: Vec<&str> = fg_text.text.lines().collect();
        let lines = if raw_lines.is_empty() {
            vec![fg_text.text.as_str()]
        } else {
            raw_lines
        };
        let max_line_len = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1);
        let line_height = fs * 1.05;

        let rot = fg_text.rotation;
        let has_transform = rot != 0;
        if has_transform {
            svg.push_str(&format!(r#"  <g transform="rotate({rot} {tx} {ty})">"#));
            svg.push('\n');
        }

        if let Some(tarja) = fg_text.bg_tarja {
            let tarja_col = tarja.to_hex();
            let pad = fg_text.tarja_padding as f32;
            let est_w = max_line_len as f32 * fg_text.font_size * 0.6 * fg_text.stretch_x;
            let est_h = fs * 1.1 + (lines.len().saturating_sub(1) as f32) * line_height;
            let rx = if fg_text.center_x { tx - (est_w / 2.0) - pad } else { tx - pad };
            let ry = ty - fs * 0.85 - pad;
            let tw = est_w + pad * 2.0;
            let th = est_h + pad * 2.0;

            let border_finish = if fg_text.border_finish == crate::press::TarjaBorderFinish::Nenhuma && fg_text.border_width > 0 {
                crate::press::TarjaBorderFinish::Solida
            } else {
                fg_text.border_finish
            };
            let stroke_attr = if border_finish != crate::press::TarjaBorderFinish::Nenhuma {
                let bw = if fg_text.border_width == 0 { 3 } else { fg_text.border_width };
                let bc = fg_text.border_color.unwrap_or(fg_text.color).to_hex();
                let dash = match border_finish {
                    crate::press::TarjaBorderFinish::Tracejada => " stroke-dasharray=\"12,6\"",
                    crate::press::TarjaBorderFinish::Picote => " stroke-dasharray=\"4,4\"",
                    crate::press::TarjaBorderFinish::FitaDurex => " stroke-dasharray=\"2,2\"",
                    _ => "",
                };
                format!(" stroke=\"{bc}\" stroke-width=\"{bw}\"{dash}")
            } else {
                String::new()
            };

            svg.push_str(&format!(
                r#"  <rect x="{rx}" y="{ry}" width="{tw}" height="{th}" fill="{tarja_col}"{stroke_attr} />
"#
            ));

            if fg_text.tarja_effect == crate::press::TarjaEffect::Xerox {
                let bg_hex = composition.background_color.to_hex();
                let scan_rel = [0.22, 0.51, 0.76, 0.89];
                for rel in scan_rel {
                    let sy = ry + th * rel;
                    svg.push_str(&format!(
                        r#"  <line x1="{rx}" y1="{sy}" x2="{}" y2="{sy}" stroke="{bg_hex}" stroke-width="2" stroke-dasharray="12,4,6,8" />
"#,
                        rx + tw
                    ));
                }
            }
        }

        let anchor = if fg_text.center_x { "middle" } else { "start" };
        let mut tspans = String::new();
        for (idx, line) in lines.iter().enumerate() {
            let dy = if idx == 0 { "0" } else { "1.05em" };
            tspans.push_str(&format!(
                r#"<tspan x="{tx}" dy="{dy}">{}</tspan>"#,
                escape_xml(line)
            ));
        }
        let font_fam = fg_text.font_family.css_font_family();
        svg.push_str(&format!(
            r#"  <text x="{tx}" y="{ty}" fill="{text_col}" font-family="{font_fam}" font-weight="900" font-size="{fs}" text-anchor="{anchor}">{tspans}</text>
"#
        ));

        if has_transform {
            svg.push_str("  </g>\n");
        }
    }

    // Camada 4: Riscos e molduras punk
    let v_col = composition.overlay.vector_color.to_hex();
    if composition.overlay.crop_frame {
        svg.push_str(&format!(
            r#"  <rect x="24" y="24" width="{}" height="{}" fill="none" stroke="{v_col}" stroke-width="3" />
"#,
            w.saturating_sub(48),
            h.saturating_sub(48)
        ));
    }

    if composition.overlay.registration_marks {
        let cs = 18;
        let marks = [
            (12, 12),
            (w as i32 - 12, 12),
            (12, h as i32 - 12),
            (w as i32 - 12, h as i32 - 12),
        ];
        for (cx, cy) in marks {
            svg.push_str(&format!(
                r#"  <path d="M {} {} L {} {} M {} {} L {} {}" stroke="{v_col}" stroke-width="2" />
"#,
                cx - cs, cy, cx + cs, cy, cx, cy - cs, cx, cy + cs
            ));
        }
    }

    // Camada 5: Stickers e adesivos com recorte e bordas variadas
    for sticker in &composition.stickers {
        let (cx, cy) = if sticker.pos_x <= 1.0 {
            (sticker.pos_x * w as f32, sticker.pos_y * h as f32)
        } else {
            (sticker.pos_x, sticker.pos_y)
        };
        let half = sticker.size as f32 / 2.0;
        let rot = sticker.rotation;
        let bw = sticker.border_width.max(2);
        let border_r = half + bw as f32;

        svg.push_str(&format!(
            r#"  <g transform="translate({cx:.1}, {cy:.1}) rotate({rot})">
"#
        ));

        // Bordas de recorte do sticker
        match sticker.border {
            crate::press::StickerBorder::DieCutBranco => {
                svg.push_str(&format!(
                    r##"    <circle cx="0" cy="0" r="{border_r}" fill="white" stroke="#777777" stroke-width="1" />
"##
                ));
            }
            crate::press::StickerBorder::LinhaPreta => {
                svg.push_str(&format!(
                    r##"    <circle cx="0" cy="0" r="{border_r}" fill="none" stroke="black" stroke-width="{bw}" />
"##
                ));
            }
            crate::press::StickerBorder::Picote => {
                svg.push_str(&format!(
                    r##"    <circle cx="0" cy="0" r="{border_r}" fill="white" stroke="#222" stroke-width="2" stroke-dasharray="4,4" />
"##
                ));
            }
            crate::press::StickerBorder::Serrilhada => {
                svg.push_str(&format!(
                    r##"    <circle cx="0" cy="0" r="{border_r}" fill="white" stroke="#ff0033" stroke-width="2" stroke-dasharray="2,2" />
"##
                ));
            }
            crate::press::StickerBorder::Nenhuma => {}
        }

        // Elemento gráfico interno
        if let Some(ref custom) = sticker.custom_image {
            let mut buf = std::io::Cursor::new(Vec::new());
            if custom.write_to(&mut buf, image::ImageFormat::Png).is_ok() {
                let b64 = bytes_to_base64(buf.get_ref());
                svg.push_str(&format!(
                    r##"    <image x="{}" y="{}" width="{}" height="{}" href="data:image/png;base64,{}" />
"##,
                    -half, -half, half * 2.0, half * 2.0, b64
                ));
            } else {
                svg.push_str(&format!(
                    r##"    <circle cx="0" cy="0" r="{}" fill="#222" stroke="white" stroke-width="2" />
"##,
                    half * 0.75
                ));
            }
        } else {
            match sticker.kind {
                crate::press::StickerKind::Anarchy => {
                    svg.push_str(&format!(
                        r##"    <circle cx="0" cy="0" r="{}" fill="none" stroke="#ff0033" stroke-width="4" />
    <path d="M 0 {} L {} {} M {} {} L 0 {} M {} {} L {} {}" stroke="#ff0033" stroke-width="4" />
"##,
                        half * 0.8,
                        -half * 0.7, -half * 0.6, half * 0.7,
                        half * 0.6, half * 0.7, -half * 0.7,
                        -half * 0.45, 0.0, half * 0.45, 0.0
                    ));
                }
                crate::press::StickerKind::RazorBlade => {
                    svg.push_str(&format!(
                        r##"    <rect x="{}" y="{}" width="{}" height="{}" fill="#e6e6e6" stroke="#222222" stroke-width="2" rx="4" />
    <line x1="{}" y1="0" x2="{}" y2="0" stroke="#111" stroke-width="3" />
"##,
                        -half * 0.85, -half * 0.5, half * 1.7, half * 1.0,
                        -half * 0.6, half * 0.6
                    ));
                }
                crate::press::StickerKind::Skull => {
                    svg.push_str(&format!(
                        r##"    <circle cx="0" cy="{}" r="{}" fill="#f0f0f0" stroke="#111" stroke-width="2" />
    <circle cx="{}" cy="{}" r="{}" fill="#111" />
    <circle cx="{}" cy="{}" r="{}" fill="#111" />
"##,
                        -half * 0.2, half * 0.55,
                        -half * 0.2, -half * 0.2, half * 0.12,
                        half * 0.2, -half * 0.2, half * 0.12
                    ));
                }
                crate::press::StickerKind::Biohazard => {
                    svg.push_str(&format!(
                        r##"    <circle cx="0" cy="0" r="{}" fill="#ffe600" stroke="#111" stroke-width="3" />
    <circle cx="0" cy="0" r="{}" fill="#111" />
"##,
                        half * 0.85, half * 0.2
                    ));
                }
                crate::press::StickerKind::SmileyAcid => {
                    svg.push_str(&format!(
                        r##"    <circle cx="0" cy="0" r="{}" fill="#ffee00" stroke="#111" stroke-width="3" />
    <circle cx="{}" cy="{}" r="{}" fill="#111" />
    <circle cx="{}" cy="{}" r="{}" fill="#111" />
    <path d="M {} {} Q 0 {} {} {}" fill="none" stroke="#111" stroke-width="3" stroke-linecap="round" />
"##,
                        half * 0.85,
                        -half * 0.25, -half * 0.18, half * 0.1,
                        half * 0.25, -half * 0.18, half * 0.1,
                        -half * 0.45, half * 0.18, half * 0.55, half * 0.45, half * 0.18
                    ));
                }
                crate::press::StickerKind::Custom => {
                    svg.push_str(&format!(
                        r##"    <circle cx="0" cy="0" r="{}" fill="#222" stroke="#ffee00" stroke-width="2" />
    <path d="M 0 {} L {} {} L {} {} L {} {} Z" fill="#ffee00" />
"##,
                        half * 0.75,
                        -half * 0.4,
                        half * 0.35, half * 0.3,
                        0.0, half * 0.1,
                        -half * 0.35, half * 0.3
                    ));
                }
                _ => {
                    svg.push_str(&format!(
                        r##"    <circle cx="0" cy="0" r="{}" fill="#222" stroke="white" stroke-width="2" />
"##,
                        half * 0.75
                    ));
                }
            }
        }

        svg.push_str("  </g>\n");
    }

    // Camada 6: Intervenção manual e desenho livre
    for stroke in &composition.manual_drawings {
        let col = stroke.color.to_hex();
        let sw = stroke.width;
        match stroke.tool {
            crate::press::DrawingTool::Freehand => {
                if stroke.points.len() == 1 {
                    let p = stroke.points[0];
                    let cx = if p.0 <= 1.0 { p.0 * w as f32 } else { p.0 };
                    let cy = if p.1 <= 1.0 { p.1 * h as f32 } else { p.1 };
                    let r = (sw / 2.0).max(1.0);
                    svg.push_str(&format!(
                        r#"  <circle cx="{cx:.1}" cy="{cy:.1}" r="{r:.1}" fill="{col}" />
"#
                    ));
                } else if stroke.points.len() > 1 {
                    let pts: Vec<String> = stroke.points.iter().map(|p| {
                        let px = if p.0 <= 1.0 { p.0 * w as f32 } else { p.0 };
                        let py = if p.1 <= 1.0 { p.1 * h as f32 } else { p.1 };
                        format!("{px:.1},{py:.1}")
                    }).collect();
                    let pts_str = pts.join(" ");
                    svg.push_str(&format!(
                        r#"  <polyline points="{pts_str}" fill="none" stroke="{col}" stroke-width="{sw}" stroke-linecap="round" stroke-linejoin="round" />
"#
                    ));
                }
            }
            crate::press::DrawingTool::Line => {
                if stroke.points.len() >= 2 {
                    let p0 = stroke.points[0];
                    let p1 = stroke.points[1];
                    let x0 = if p0.0 <= 1.0 { p0.0 * w as f32 } else { p0.0 };
                    let y0 = if p0.1 <= 1.0 { p0.1 * h as f32 } else { p0.1 };
                    let x1 = if p1.0 <= 1.0 { p1.0 * w as f32 } else { p1.0 };
                    let y1 = if p1.1 <= 1.0 { p1.1 * h as f32 } else { p1.1 };
                    svg.push_str(&format!(
                        r#"  <line x1="{x0:.1}" y1="{y0:.1}" x2="{x1:.1}" y2="{y1:.1}" stroke="{col}" stroke-width="{sw}" stroke-linecap="round" />
"#
                    ));
                }
            }
            crate::press::DrawingTool::Cross => {
                if !stroke.points.is_empty() {
                    let p0 = stroke.points[0];
                    let p1 = stroke.points.get(1).copied().unwrap_or(p0);
                    let mut x0 = if p0.0 <= 1.0 { p0.0 * w as f32 } else { p0.0 };
                    let mut y0 = if p0.1 <= 1.0 { p0.1 * h as f32 } else { p0.1 };
                    let mut x1 = if p1.0 <= 1.0 { p1.0 * w as f32 } else { p1.0 };
                    let mut y1 = if p1.1 <= 1.0 { p1.1 * h as f32 } else { p1.1 };
                    if (x1 - x0).abs() < 5.0 && (y1 - y0).abs() < 5.0 {
                        let sz = (sw * 4.0).max(18.0);
                        x0 -= sz;
                        y0 -= sz;
                        x1 += sz;
                        y1 += sz;
                    }
                    svg.push_str(&format!(
                        r#"  <line x1="{x0:.1}" y1="{y0:.1}" x2="{x1:.1}" y2="{y1:.1}" stroke="{col}" stroke-width="{sw}" stroke-linecap="round" />
  <line x1="{x0:.1}" y1="{y1:.1}" x2="{x1:.1}" y2="{y0:.1}" stroke="{col}" stroke-width="{sw}" stroke-linecap="round" />
"#
                    ));
                }
            }
            crate::press::DrawingTool::Arrow => {
                if stroke.points.len() >= 2 {
                    let p0 = stroke.points[0];
                    let p1 = stroke.points[1];
                    let x0 = if p0.0 <= 1.0 { p0.0 * w as f32 } else { p0.0 };
                    let y0 = if p0.1 <= 1.0 { p0.1 * h as f32 } else { p0.1 };
                    let x1 = if p1.0 <= 1.0 { p1.0 * w as f32 } else { p1.0 };
                    let y1 = if p1.1 <= 1.0 { p1.1 * h as f32 } else { p1.1 };
                    let dx = x1 - x0;
                    let dy = y1 - y0;
                    let angle = dy.atan2(dx);
                    let wing_len = (sw * 3.5).max(16.0);
                    let a1 = angle + std::f32::consts::PI - (std::f32::consts::PI / 6.0);
                    let a2 = angle + std::f32::consts::PI + (std::f32::consts::PI / 6.0);
                    let wx1 = x1 + wing_len * a1.cos();
                    let wy1 = y1 + wing_len * a1.sin();
                    let wx2 = x1 + wing_len * a2.cos();
                    let wy2 = y1 + wing_len * a2.sin();
                    svg.push_str(&format!(
                        r#"  <line x1="{x0:.1}" y1="{y0:.1}" x2="{x1:.1}" y2="{y1:.1}" stroke="{col}" stroke-width="{sw}" stroke-linecap="round" />
  <line x1="{x1:.1}" y1="{y1:.1}" x2="{wx1:.1}" y2="{wy1:.1}" stroke="{col}" stroke-width="{sw}" stroke-linecap="round" />
  <line x1="{x1:.1}" y1="{y1:.1}" x2="{wx2:.1}" y2="{wy2:.1}" stroke="{col}" stroke-width="{sw}" stroke-linecap="round" />
"#
                    ));
                }
            }
        }
    }

    svg.push_str("</svg>\n");
    svg
}

fn bytes_to_base64(bytes: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARSET[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARSET[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARSET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARSET[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

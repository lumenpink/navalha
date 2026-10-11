//! Módulo para injeção e leitura de metadados em chunks textuais PNG (`tEXt`).
//!
//! Padrão PNG:
//! - Chunks consistem em: [4 bytes length][4 bytes type][data][4 bytes crc32].
//! - Para `tEXt`: [keyword (1..79 bytes latin1)][null byte 0x00][text value (latin1/utf8)].
//! - Chunks de metadados são inseridos entre o cabeçalho PNG (IHDR) e o chunk de encerramento (IEND).

use std::io;

/// Calcula o CRC-32 no padrão IEEE 802.3 (polinômio 0xEDB88320 / standard PNG).
#[inline]
pub fn crc32(buf: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in buf {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// Cria um chunk PNG `tEXt` com chave e valor UTF-8 / Latin-1.
pub fn create_text_chunk(keyword: &str, text: &str) -> Vec<u8> {
    let mut chunk_data = Vec::with_capacity(keyword.len() + 1 + text.len());
    chunk_data.extend_from_slice(keyword.as_bytes());
    chunk_data.push(0x00); // Separador nulo exigido pela especificação PNG
    chunk_data.extend_from_slice(text.as_bytes());

    let len = chunk_data.len() as u32;
    let mut chunk_full = Vec::with_capacity(12 + chunk_data.len());
    chunk_full.extend_from_slice(&len.to_be_bytes());
    chunk_full.extend_from_slice(b"tEXt");
    chunk_full.extend_from_slice(&chunk_data);

    // CRC cobre o tipo do chunk ("tEXt") e os dados
    let mut crc_buf = Vec::with_capacity(4 + chunk_data.len());
    crc_buf.extend_from_slice(b"tEXt");
    crc_buf.extend_from_slice(&chunk_data);
    let crc_val = crc32(&crc_buf);
    chunk_full.extend_from_slice(&crc_val.to_be_bytes());

    chunk_full
}

/// Injeta metadados textuais (`tEXt`) em um fluxo de bytes PNG já codificado.
///
/// Os chunks são inseridos logo antes do marcador `IEND`, preservando a integridade
/// total de visualização em qualquer software padrão.
pub fn inject_png_metadata(png_bytes: &[u8], metadata: &[(&str, &str)]) -> io::Result<Vec<u8>> {
    const PNG_MAGIC: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if png_bytes.len() < 8 || &png_bytes[0..8] != PNG_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Arquivo não possui assinatura mágica de PNG",
        ));
    }

    // Procura o chunk IEND (últimos 12 bytes geralmente, mas varremos com segurança)
    let iend_marker = b"IEND";
    let mut iend_offset = None;

    let mut cursor = 8;
    while cursor + 8 <= png_bytes.len() {
        let length = u32::from_be_bytes([
            png_bytes[cursor],
            png_bytes[cursor + 1],
            png_bytes[cursor + 2],
            png_bytes[cursor + 3],
        ]) as usize;
        let chunk_type = &png_bytes[cursor + 4..cursor + 8];
        if chunk_type == iend_marker {
            iend_offset = Some(cursor);
            break;
        }
        cursor += 12 + length;
    }

    let insert_pos = match iend_offset {
        Some(pos) => pos,
        None => {
            // Se por algum motivo o chunk IEND não for encontrado estruturado, insere antes do final se contiver b"IEND"
            if let Some(pos) = png_bytes.windows(4).rposition(|w| w == iend_marker) {
                pos.saturating_sub(4)
            } else {
                png_bytes.len()
            }
        }
    };

    let mut out = Vec::with_capacity(png_bytes.len() + metadata.len() * 128);
    out.extend_from_slice(&png_bytes[..insert_pos]);

    for &(k, v) in metadata {
        let chunk = create_text_chunk(k, v);
        out.extend_from_slice(&chunk);
    }

    out.extend_from_slice(&png_bytes[insert_pos..]);
    Ok(out)
}

/// Extrai todos os pares de metadados (`keyword`, `text`) contidos em chunks `tEXt` de um PNG.
pub fn extract_png_metadata(png_bytes: &[u8]) -> Vec<(String, String)> {
    const PNG_MAGIC: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if png_bytes.len() < 8 || &png_bytes[0..8] != PNG_MAGIC {
        return Vec::new();
    }

    let mut results = Vec::new();
    let mut cursor = 8;

    while cursor + 8 <= png_bytes.len() {
        let length = u32::from_be_bytes([
            png_bytes[cursor],
            png_bytes[cursor + 1],
            png_bytes[cursor + 2],
            png_bytes[cursor + 3],
        ]) as usize;
        let chunk_type = &png_bytes[cursor + 4..cursor + 8];
        let data_start = cursor + 8;
        let data_end = data_start + length;

        if data_end > png_bytes.len() {
            break;
        }

        if chunk_type == b"tEXt" {
            let chunk_data = &png_bytes[data_start..data_end];
            if let Some(null_idx) = chunk_data.iter().position(|&b| b == 0x00) {
                let key = String::from_utf8_lossy(&chunk_data[..null_idx]).to_string();
                let val = String::from_utf8_lossy(&chunk_data[null_idx + 1..]).to_string();
                results.push((key, val));
            }
        }

        cursor = data_end + 4; // Pula os 4 bytes de CRC
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_standard() {
        // "123456789" -> CRC32 é 0xCBF43926
        let res = crc32(b"123456789");
        assert_eq!(res, 0xCBF43926);
    }

    #[test]
    fn test_inject_and_extract_metadata() {
        // Gera um PNG sintético mínimo
        let mut fake_png = Vec::new();
        fake_png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
        // IHDR
        let ihdr_data = [0u8; 13];
        fake_png.extend_from_slice(&(13u32).to_be_bytes());
        fake_png.extend_from_slice(b"IHDR");
        fake_png.extend_from_slice(&ihdr_data);
        fake_png.extend_from_slice(&crc32(b"IHDR\0\0\0\0\0\0\0\0\0\0\0\0\0").to_be_bytes());
        // IEND
        fake_png.extend_from_slice(&0u32.to_be_bytes());
        fake_png.extend_from_slice(b"IEND");
        fake_png.extend_from_slice(&crc32(b"IEND").to_be_bytes());

        let metadata = [
            ("Software", "Navalha v0.1.0"),
            ("Command", "navalha poster foto.jpg --frase PUNK -e punk-riot"),
            ("License", "CC BY-SA 4.0"),
        ];

        let injected = inject_png_metadata(&fake_png, &metadata).expect("Injeção com sucesso");
        let extracted = extract_png_metadata(&injected);

        assert_eq!(extracted.len(), 3);
        assert_eq!(extracted[0], ("Software".to_string(), "Navalha v0.1.0".to_string()));
        assert_eq!(extracted[1], ("Command".to_string(), "navalha poster foto.jpg --frase PUNK -e punk-riot".to_string()));
        assert_eq!(extracted[2], ("License".to_string(), "CC BY-SA 4.0".to_string()));
    }
}

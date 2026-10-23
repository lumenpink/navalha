# Especificação 41: Otimização de Tamanho Binário e Linking Estático musl

- **ID do Ticket:** #41
- **Status:** Aprovado
- **Autora:** Lumen
- **Alinhamento:** GOST 1.5.0 (Rust)

## 1. Contexto & Motivação
O Navalha (`navalha`) é a ferramenta de corte, redimensionamento, criação de figurinhas de WhatsApp e processamento visual de imagens em Rust.
Para garantir máxima velocidade de processamento visual com binários ultracompactos e 100% portáteis:
1. Declara-se o perfil `[profile.release]` com `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"` e `strip = true`.
2. Configura-se `.cargo/config.toml` forçando o linking estático da C runtime (`target-feature=+crt-static`) para targets musl.

## 2. Requisitos Técnicos
- `[profile.release]` configurado com `opt-level = 3` no `Cargo.toml`.
- `.cargo/config.toml` contendo `rustflags = ["-C", "target-feature=+crt-static"]` para `x86_64-unknown-linux-musl` e `aarch64-unknown-linux-musl`.

## 3. Critérios de Aceitação
- Teste automatizado validando as diretivas em `Cargo.toml` e `.cargo/config.toml`.
- Binários resultantes sem símbolos de depuração (`stripped`) e estáticos.

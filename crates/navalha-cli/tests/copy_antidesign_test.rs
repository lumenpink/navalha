use std::fs;
use std::path::PathBuf;

#[test]
fn test_navalha_copy_antidesign_sem_jargoes_wasm() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();

    let site_index = fs::read_to_string(root.join("site/index.html")).expect("site/index.html");
    let web_index = fs::read_to_string(root.join("crates/navalha-web/index.html"))
        .expect("crates/navalha-web/index.html");
    let doc_index =
        fs::read_to_string(root.join("site/doc/index.html")).expect("site/doc/index.html");

    // 1. Preservar o termo anti-design
    assert!(
        site_index.to_lowercase().contains("anti-design"),
        "site/index.html deve celebrar anti-design"
    );
    assert!(
        web_index.to_lowercase().contains("anti-design"),
        "navalha-web/index.html deve celebrar anti-design"
    );

    // 2. Não deve conter explicações técnicas de WebAssembly / Wasm / processamento de memória nas descrições públicas
    assert!(
        !site_index.contains("WebAssembly"),
        "site/index.html não deve conter jargão WebAssembly"
    );
    assert!(
        !web_index.contains("Wasm"),
        "crates/navalha-web/index.html não deve conter jargão Wasm"
    );
    assert!(
        !doc_index.contains("Rust e WebAssembly"),
        "site/doc/index.html não deve conter 'Rust e WebAssembly'"
    );
    assert!(
        !doc_index.contains("na memória do seu navegador"),
        "site/doc/index.html não deve conter 'na memória do seu navegador'"
    );

    // 3. Preservar homenagens históricas intactas (ex: Neon Cunha)
    assert!(
        doc_index.contains("Neon Cunha")
            && doc_index.contains("autodeterminação corporal, a soberania e a dignidade humana"),
        "Homenagem histórica à Neon Cunha deve ser preservada intacta"
    );
}

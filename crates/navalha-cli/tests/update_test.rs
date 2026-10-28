use navalha_cli::update::{
    UpdateConfig, UpdateState, get_config_path, get_distribution_endpoints, get_download_url,
    get_state_path, is_version_pinned, load_update_config, load_update_state, resolve_channel,
    save_default_channel, save_update_config, save_update_state, semver_is_newer,
};
use std::env;
use tempfile::tempdir;

#[test]
fn test_xdg_path_resolution_with_custom_env() {
    let tmp = tempdir().unwrap();
    let config_dir = tmp.path().join("my_config");
    let state_dir = tmp.path().join("my_state");

    unsafe {
        env::set_var("XDG_CONFIG_HOME", &config_dir);
        env::set_var("XDG_STATE_HOME", &state_dir);
    }

    let config_path = get_config_path();
    let state_path = get_state_path();

    assert_eq!(config_path, config_dir.join("navalha").join("config.toml"));
    assert_eq!(
        state_path,
        state_dir.join("navalha").join("update_check.json")
    );

    unsafe {
        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("XDG_STATE_HOME");
    }
}

#[test]
fn test_pinning_logic_and_safeguards() {
    let cfg = UpdateConfig::default();
    assert!(!is_version_pinned(&cfg, "0.1.0"));

    let cfg_pinned = UpdateConfig {
        pinned: true,
        ..Default::default()
    };
    assert!(is_version_pinned(&cfg_pinned, "0.1.0"));

    let cfg_pin_ver = UpdateConfig {
        pinned: false,
        pin: "0.1.0".to_string(),
        ..Default::default()
    };
    assert!(is_version_pinned(&cfg_pin_ver, "0.1.0"));

    let cfg_pin_diff = UpdateConfig {
        pinned: false,
        pin: "0.0.9".to_string(),
        ..Default::default()
    };
    assert!(is_version_pinned(&cfg_pin_diff, "0.1.0"));
}

#[test]
fn test_state_cache_roundtrip() {
    let tmp = tempdir().unwrap();
    let state_file = tmp.path().join("update_check.json");

    let state = UpdateState {
        last_check_timestamp: 1729411200,
        latest_remote_version: "0.2.0".to_string(),
        channel: "nightly".to_string(),
    };

    save_update_state(&state, Some(&state_file)).unwrap();
    let loaded = load_update_state(Some(&state_file));

    assert_eq!(loaded.last_check_timestamp, 1729411200);
    assert_eq!(loaded.latest_remote_version, "0.2.0");
    assert_eq!(loaded.channel, "nightly");
}

#[test]
fn test_update_config_full_schema() {
    let tmp = tempdir().unwrap();
    let cfg_file = tmp.path().join("config.toml");

    let cfg = UpdateConfig {
        channel: "nightly".to_string(),
        pinned: true,
        pin: "0.1.0".to_string(),
        auto_check: true,
        auto_update: false,
        check_interval_hours: 12,
    };

    save_update_config(&cfg, Some(&cfg_file)).unwrap();
    let loaded = load_update_config(Some(&cfg_file));

    assert_eq!(loaded.channel, "nightly");
    assert!(loaded.pinned);
    assert_eq!(loaded.pin, "0.1.0");
    assert!(loaded.auto_check);
    assert!(!loaded.auto_update);
    assert_eq!(loaded.check_interval_hours, 12);
}

#[test]
fn test_distribution_endpoints_banca() {
    let (stable, nightly, ver) = get_distribution_endpoints("stable");
    assert_eq!(
        stable,
        "https://banca.2lp.in/downloads/navalha/stable-latest/navalha"
    );
    assert_eq!(
        nightly,
        "https://banca.2lp.in/downloads/navalha/nightly-latest/navalha"
    );
    assert_eq!(ver, "https://banca.2lp.in/downloads/navalha/version");

    assert!(!stable.contains("codeberg.org"));
    assert!(!nightly.contains("codeberg.org"));
    assert!(!ver.contains("codeberg.org"));

    let (_, _, nightly_ver) = get_distribution_endpoints("nightly");
    assert_eq!(
        nightly_ver,
        "https://banca.2lp.in/downloads/navalha/nightly-version"
    );
}

#[test]
fn test_semver_precedence() {
    assert!(semver_is_newer("0.2.0", "0.1.0"));
    assert!(!semver_is_newer("0.1.0", "0.1.0"));
    assert!(!semver_is_newer("0.1.0", "0.2.0"));
    assert!(semver_is_newer("0.1.1", "0.1.0"));
    assert!(semver_is_newer("1.0.0", "0.9.9"));
    assert!(semver_is_newer(
        "0.1.0-dev.2+20261019",
        "0.1.0-dev.1+20261019"
    ));
}

#[test]
fn test_resolve_channel_precedence() {
    let tmp = tempdir().unwrap();
    let config_file = tmp.path().join("config.toml");

    // 1. Default without flag, env or config -> "stable"
    let ch = resolve_channel(None, None, Some(&config_file));
    assert_eq!(ch, "stable");

    // 2. Persisted config -> "nightly"
    save_default_channel("nightly", Some(&config_file)).unwrap();
    let ch = resolve_channel(None, None, Some(&config_file));
    assert_eq!(ch, "nightly");

    // 3. Env var overrides config
    let ch = resolve_channel(None, Some("stable"), Some(&config_file));
    assert_eq!(ch, "stable");

    // 4. CLI flag overrides everything
    let ch = resolve_channel(Some("nightly"), Some("stable"), Some(&config_file));
    assert_eq!(ch, "nightly");
}

#[test]
fn test_get_download_url_and_pinning() {
    assert_eq!(
        get_download_url("stable", None),
        "https://banca.2lp.in/downloads/navalha/stable-latest/navalha"
    );
    assert_eq!(
        get_download_url("nightly", None),
        "https://banca.2lp.in/downloads/navalha/nightly-latest/navalha"
    );
    assert_eq!(
        get_download_url("stable", Some("0.2.0")),
        "https://banca.2lp.in/downloads/navalha/releases/v0.2.0/navalha"
    );
    assert_eq!(
        get_download_url("stable", Some("v0.3.0")),
        "https://banca.2lp.in/downloads/navalha/releases/v0.3.0/navalha"
    );
}

#[test]
fn test_semver_edge_cases_with_prefixes() {
    assert!(semver_is_newer("v0.2.0", "0.1.0"));
    assert!(semver_is_newer("0.2.0", "v0.1.0"));
    assert!(!semver_is_newer("v0.1.0", "0.1.0"));
    assert!(!semver_is_newer("0.1.0", "v0.1.0"));
    assert!(semver_is_newer("0.2.0", "0.1.99"));
}

#[test]
fn test_mutation_fortification_updater_edge_cases() {
    let tmp = tempdir().unwrap();
    let cfg_file = tmp.path().join("sub").join("config.toml");

    // Salvar configuração cria automaticamente diretórios intermediários
    let mut cfg = UpdateConfig {
        channel: "nightly".to_string(),
        pinned: false,
        pin: "".to_string(),
        auto_check: false,
        auto_update: true,
        check_interval_hours: 48,
    };
    save_update_config(&cfg, Some(&cfg_file)).unwrap();
    assert!(cfg_file.exists());

    let loaded = load_update_config(Some(&cfg_file));
    assert_eq!(loaded.channel, "nightly");
    assert!(!loaded.pinned);
    assert!(!loaded.auto_check);
    assert!(loaded.auto_update);
    assert_eq!(loaded.check_interval_hours, 48);

    // Pinning mutantes: pin vazio e pinned falso -> desativado
    assert!(!is_version_pinned(&cfg, "0.1.0"));

    // Pinning mutantes: pin preenchido mesmo com pinned false -> ativado
    cfg.pin = "0.1.0".to_string();
    assert!(is_version_pinned(&cfg, "0.1.0"));

    // Semver mutantes
    assert!(!semver_is_newer("0.1.0", "0.1.0"));
    assert!(!semver_is_newer("invalid-a", "invalid-b"));
    assert!(semver_is_newer("0.2.0-dev.10", "0.2.0-dev.5"));
    assert!(semver_is_newer("1.0.0", "invalid"));
}

#[test]
fn test_jornaleira_delegation_and_offline_startup() {
    use navalha_cli::update::{
        is_startup_check_discontinued, jornaleira_notice, run_startup_check,
    };

    // 1. A checagem em background no startup deve estar formalmente descontinuada
    assert!(is_startup_check_discontinued());

    // 2. A diretiva da Jornaleira deve orientar o usuário sobre a gestão central
    let notice = jornaleira_notice();
    assert!(notice.starts_with("★ GOSPLAN:"));
    assert!(notice.contains("distribuição de binários e atualizações"));
    assert!(notice.contains("Jornaleira"));
    assert!(notice.contains("jornaleira upgrade navalha"));
    assert!(notice.len() > 30);

    // 3. Execução de run_startup_check não deve falhar nem criar arquivo de estado espúrio
    let tmp = tempdir().unwrap();
    let state_file = tmp.path().join("fake_state.json");
    unsafe {
        std::env::set_var("XDG_STATE_HOME", tmp.path());
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());
    }
    // Testar com diversas versões e garantir que nenhuma cria arquivos ou gera side-effects
    run_startup_check("0.1.0");
    run_startup_check("");
    run_startup_check("99.99.99");
    assert!(!state_file.exists());
    assert!(!tmp.path().join("navalha").exists());
    unsafe {
        std::env::remove_var("XDG_STATE_HOME");
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}

#[test]
fn test_mutation_fortification_jornaleira_constants() {
    use navalha_cli::update::{JORNALEIRA_NOTICE, jornaleira_notice};
    // Garantir que a constante canônica coincide exatamente com a função pública
    assert_eq!(JORNALEIRA_NOTICE, jornaleira_notice());
    assert!(!JORNALEIRA_NOTICE.is_empty());
}

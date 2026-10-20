use navalha_cli::update::{
    get_distribution_endpoints, resolve_channel, save_default_channel, semver_is_newer,
};
use tempfile::tempdir;

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
    use navalha_cli::update::get_download_url;

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

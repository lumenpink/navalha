use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const APP_NAME: &str = "navalha";

fn default_channel() -> String {
    "stable".to_string()
}

fn default_check_interval() -> u64 {
    24
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateConfig {
    #[serde(default = "default_channel")]
    pub channel: String,
    #[serde(default)]
    pub auto_check: bool,
    #[serde(default)]
    pub auto_update: bool,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub pin: String,
    #[serde(default = "default_check_interval")]
    pub check_interval_hours: u64,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            channel: default_channel(),
            auto_check: true,
            auto_update: false,
            pinned: false,
            pin: String::new(),
            check_interval_hours: default_check_interval(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct NavalhaConfig {
    pub update: Option<UpdateConfig>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct UpdateState {
    pub last_check_timestamp: u64,
    pub latest_remote_version: String,
    pub channel: String,
}

/// Retorna o caminho do arquivo de configuração do Navalha segundo o padrão XDG
pub fn get_config_path() -> PathBuf {
    if let Some(xdg) = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|s| !s.trim().is_empty())
    {
        return PathBuf::from(xdg.trim())
            .join(APP_NAME)
            .join("config.toml");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join(APP_NAME)
            .join("config.toml");
    }
    PathBuf::from(".navalha_config.toml")
}

/// Retorna o caminho legado de configuração para retrocompatibilidade
pub fn get_default_config_path() -> PathBuf {
    get_config_path()
}

/// Retorna o caminho do arquivo de estado de runtime do Navalha segundo o padrão XDG
pub fn get_state_path() -> PathBuf {
    if let Some(xdg) = std::env::var("XDG_STATE_HOME")
        .ok()
        .filter(|s| !s.trim().is_empty())
    {
        return PathBuf::from(xdg.trim())
            .join(APP_NAME)
            .join("update_check.json");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("state")
            .join(APP_NAME)
            .join("update_check.json");
    }
    PathBuf::from(".navalha_update_check.json")
}

/// Carrega o arquivo de configuração persistido (NavalhaConfig)
pub fn load_config(path: Option<&Path>) -> NavalhaConfig {
    let p = path.map(PathBuf::from).unwrap_or_else(get_config_path);
    if !p.exists() {
        return NavalhaConfig::default();
    }
    match fs::read_to_string(&p) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => NavalhaConfig::default(),
    }
}

/// Carrega especificamente a seção [update] da configuração
pub fn load_update_config(path: Option<&Path>) -> UpdateConfig {
    let cfg = load_config(path);
    cfg.update.unwrap_or_default()
}

/// Salva as configurações de atualização no arquivo de configuração
pub fn save_update_config(
    update_cfg: &UpdateConfig,
    path: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let p = path.map(PathBuf::from).unwrap_or_else(get_config_path);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut cfg = load_config(Some(&p));
    cfg.update = Some(update_cfg.clone());

    let toml_str = toml::to_string_pretty(&cfg)?;
    fs::write(&p, toml_str)?;
    Ok(())
}

/// Salva o canal padrão no arquivo de configuração (mantido para retrocompatibilidade)
pub fn save_default_channel(
    channel: &str,
    path: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut update_cfg = load_update_config(path);
    update_cfg.channel = channel.to_string();
    save_update_config(&update_cfg, path)
}

/// Carrega o cache de estado de checagem de runtime
pub fn load_update_state(path: Option<&Path>) -> UpdateState {
    let p = path.map(PathBuf::from).unwrap_or_else(get_state_path);
    if !p.exists() {
        return UpdateState::default();
    }
    match fs::read_to_string(&p) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => UpdateState::default(),
    }
}

/// Salva o estado de checagem de runtime em disco
pub fn save_update_state(
    state: &UpdateState,
    path: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let p = path.map(PathBuf::from).unwrap_or_else(get_state_path);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    let json_str = serde_json::to_string_pretty(state)?;
    fs::write(&p, json_str)?;
    Ok(())
}

/// Verifica se a versão corrente está travada (pinning)
pub fn is_version_pinned(config: &UpdateConfig, _current_ver: &str) -> bool {
    if config.pinned {
        return true;
    }
    let pin = config.pin.trim();
    if !pin.is_empty() {
        return true;
    }
    false
}

/// Resolve o canal ativo seguindo a precedência: Flag > Env > Config > "stable"
pub fn resolve_channel(
    flag: Option<&str>,
    env_val: Option<&str>,
    config_path: Option<&Path>,
) -> String {
    if let Some(f) = flag.map(str::trim).filter(|s| !s.is_empty()) {
        return f.to_lowercase();
    }

    if let Some(e) = env_val.map(str::trim).filter(|s| !s.is_empty()) {
        return e.to_lowercase();
    }

    if let Ok(e) = std::env::var("NAVALHA_CHANNEL") {
        let trimmed = e.trim();
        if !trimmed.is_empty() {
            return trimmed.to_lowercase();
        }
    }

    let cfg = load_update_config(config_path);
    let ch = cfg.channel.trim();
    if !ch.is_empty() {
        return ch.to_lowercase();
    }

    "stable".to_string()
}

/// Retorna os endpoints soberanos oficiais de distribuição na Banca: (stable_url, nightly_url, version_url)
pub fn get_distribution_endpoints(channel: &str) -> (String, String, String) {
    let stable = "https://banca.2lp.in/downloads/navalha/stable-latest/navalha".to_string();
    let nightly = "https://banca.2lp.in/downloads/navalha/nightly-latest/navalha".to_string();
    let ver = if channel == "nightly" {
        "https://banca.2lp.in/downloads/navalha/nightly-version".to_string()
    } else {
        "https://banca.2lp.in/downloads/navalha/version".to_string()
    };
    (stable, nightly, ver)
}

/// Retorna a URL de download correspondente à seleção
pub fn get_download_url(channel: &str, pin: Option<&str>) -> String {
    if let Some(p) = pin {
        let clean = p.trim().trim_start_matches('v');
        format!(
            "https://banca.2lp.in/downloads/navalha/releases/v{}/navalha",
            clean
        )
    } else if channel == "nightly" {
        "https://banca.2lp.in/downloads/navalha/nightly-latest/navalha".to_string()
    } else {
        "https://banca.2lp.in/downloads/navalha/stable-latest/navalha".to_string()
    }
}

/// Compara se a versão remota é mais recente que a atual
pub fn semver_is_newer(remote: &str, current: &str) -> bool {
    let r_clean = remote.trim().trim_start_matches('v');
    let c_clean = current.trim().trim_start_matches('v');

    if r_clean == c_clean {
        return false;
    }

    if let (Ok(r_ver), Ok(c_ver)) = (
        semver::Version::parse(r_clean),
        semver::Version::parse(c_clean),
    ) {
        return r_ver > c_ver;
    }

    // Fallback: se houver sufixos de commit ou timestamp
    r_clean > c_clean
}

/// Obtém a versão compilada atual do Navalha
pub fn get_current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Verificação periódica não-bloqueante no startup do Navalha
pub fn run_startup_check(current_version: &str) {
    let cfg = load_update_config(None);
    if !cfg.auto_check || is_version_pinned(&cfg, current_version) {
        return;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let state = load_update_state(None);
    let interval_secs = cfg.check_interval_hours.saturating_mul(3600);

    if now.saturating_sub(state.last_check_timestamp) < interval_secs {
        if !state.latest_remote_version.is_empty()
            && semver_is_newer(&state.latest_remote_version, current_version)
        {
            eprintln!(
                "★ [navalha] Nova versão disponível: v{} -> v{} (execute 'navalha update' para atualizar)",
                current_version, state.latest_remote_version
            );
        }
        return;
    }

    // Timeout ultracurto para não onerar o startup
    let (_, _, ver_url) = get_distribution_endpoints(&cfg.channel);
    let ureq_cfg = ureq::config::Config::builder()
        .timeout_global(Some(Duration::from_millis(1500)))
        .build();
    let agent = ureq::Agent::new_with_config(ureq_cfg);

    let mut remote_ver = String::new();
    if let Ok(mut body) = agent.get(&ver_url).call().map(|r| r.into_body())
        && let Ok(text) = body.read_to_string()
    {
        remote_ver = text.trim().to_string();
    }

    if !remote_ver.is_empty() {
        let new_state = UpdateState {
            last_check_timestamp: now,
            latest_remote_version: remote_ver.clone(),
            channel: cfg.channel.clone(),
        };
        let _ = save_update_state(&new_state, None);

        if semver_is_newer(&remote_ver, current_version) {
            eprintln!(
                "★ [navalha] Nova versão disponível: v{} -> v{} (execute 'navalha update' para atualizar)",
                current_version, remote_ver
            );
        }
    }
}

/// Consulta o endpoint da Banca para checagem de atualizações
pub fn cmd_check_update(channel_override: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let channel = resolve_channel(channel_override, None, None);
    let current_ver = get_current_version();
    let (_, _, ver_url) = get_distribution_endpoints(&channel);

    println!("⚡ Navalha Updater // Banca Soberana 2lp.in");
    println!("Canal de Distribuição : {}", channel);
    println!("Versão Local Atual    : v{}", current_ver);
    println!("Endpoint de Consulta  : {}", ver_url);

    let ureq_cfg = ureq::config::Config::builder()
        .timeout_global(Some(Duration::from_secs(3)))
        .build();
    let agent = ureq::Agent::new_with_config(ureq_cfg);

    let mut remote_ver = String::new();
    if let Ok(mut body) = agent.get(&ver_url).call().map(|r| r.into_body())
        && let Ok(text) = body.read_to_string()
    {
        remote_ver = text.trim().to_string();
    }

    if remote_ver.is_empty() {
        println!("Versão Remota         : indisponível (servidor offline ou rota não publicada)");
        println!("\nNão foi possível validar atualizações no momento.");
        return Ok(());
    }

    println!("Versão Remota         : v{}", remote_ver);

    if semver_is_newer(&remote_ver, current_ver) {
        println!(
            "\n★ Nova versão disponível: v{} -> v{}",
            current_ver, remote_ver
        );
        println!("Para atualizar agora, execute:");
        println!("  navalha update --channel {}", channel);
    } else {
        println!(
            "\n✔ O Navalha já está atualizado na versão mais recente do canal {}.",
            channel
        );
    }

    Ok(())
}

/// Executa a atualização atômica do binário com salvaguardas
pub fn cmd_update(
    channel_override: Option<&str>,
    pin_override: Option<&str>,
    force: bool,
    dry_run: bool,
    save_channel: bool,
    target_override: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let channel = resolve_channel(channel_override, None, None);
    let current_ver = get_current_version();
    let cfg = load_update_config(None);

    if is_version_pinned(&cfg, current_ver) && !force {
        eprintln!("✖ Atualização abortada: Versão está fixada (pin: '{}', pinned: {}).", cfg.pin, cfg.pinned);
        eprintln!("Use --force (-f) para forçar a atualização ou 'navalha config unpin' para liberar.");
        return Err("Versão fixada impede atualização sem --force".into());
    }

    if save_channel {
        save_default_channel(&channel, None)?;
        println!("✔ Canal padrão salvo com sucesso: '{}'", channel);
    }

    let pin = pin_override.or_else(|| if !cfg.pin.trim().is_empty() { Some(cfg.pin.as_str()) } else { None });
    let download_url = get_download_url(&channel, pin);

    println!("⚡ Navalha Auto-Update");
    println!("Canal Selecionado : {}", channel);
    if let Some(p) = pin {
        println!("Versão Fixada     : v{}", p);
    }
    println!("Versão Atual      : v{}", current_ver);
    println!("Baixando de       : {}", download_url);

    if dry_run {
        println!("✔ Modo de simulação ativado (--dry-run). Nenhuma modificação aplicada ao binário.");
        return Ok(());
    }

    let target_path = if let Some(t) = target_override {
        t.to_path_buf()
    } else {
        std::env::current_exe()?
    };

    let ureq_cfg = ureq::config::Config::builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .build();
    let agent = ureq::Agent::new_with_config(ureq_cfg);

    let resp = agent
        .get(&download_url)
        .call()
        .map_err(|e| format!("Falha ao conectar com {}: {}", download_url, e))?;

    let bytes = resp
        .into_body()
        .read_to_vec()
        .map_err(|e| format!("Falha ao baixar payload do executável: {}", e))?;

    if bytes.is_empty() {
        return Err("Arquivo baixado está vazio.".into());
    }

    let target_dir = target_path
        .parent()
        .ok_or("Não foi possível identificar o diretório do binário alvo.")?;

    let mut temp_file = tempfile::Builder::new()
        .prefix(".navalha-update-")
        .tempfile_in(target_dir)?;

    temp_file.write_all(&bytes)?;
    temp_file.flush()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(temp_file.path(), perms)?;
    }

    temp_file.persist(&target_path)?;

    println!(
        "✔ Navalha atualizado com sucesso em '{}'!",
        target_path.display()
    );
    Ok(())
}

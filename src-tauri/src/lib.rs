mod sync;

use anyhow::{anyhow, Context, Result};
use git2::{
    Cred, CredentialType, FetchOptions, IndexAddOption, PushOptions, RemoteCallbacks, Repository,
    Signature,
};
use keyring::Entry;
use serde::Deserialize;
use std::{fs, path::Path, sync::OnceLock};
#[cfg(target_os = "android")]
use std::path::PathBuf;
#[cfg(target_os = "android")]
use tauri::Manager;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub remote_url: String,
    pub username: String,
    #[serde(default)]
    pub token: String,
    pub branch: String,
    pub folder: String,
}

fn credential_key(remote_url: &str, username: &str) -> String {
    format!("{}:{}", remote_url, username)
}

fn ensure_keyring_store() -> Result<()> {
    static INIT: OnceLock<std::result::Result<(), String>> = OnceLock::new();
    match INIT.get_or_init(|| {
        keyring::cli::use_native_store(false)
            .map_err(|e| format!("не удалось инициализировать системное хранилище ключей: {e:#}"))
    }) {
        Ok(()) => Ok(()),
        Err(message) => Err(anyhow!(message.clone())),
    }
}

#[cfg(target_os = "android")]
static ANDROID_CA_BUNDLE_INIT: OnceLock<std::result::Result<(), String>> = OnceLock::new();

#[cfg(target_os = "android")]
fn build_android_ca_bundle(cache_dir: &Path) -> Result<PathBuf> {
    let candidates = [
        "/system/etc/security/cacerts",
        "/apex/com.android.conscrypt/cacerts",
    ];
    let source = candidates
        .iter()
        .map(Path::new)
        .find(|p| p.is_dir())
        .ok_or_else(|| anyhow!("системное хранилище CA-сертификатов Android не найдено"))?;

    let mut files: Vec<PathBuf> = fs::read_dir(source)
        .with_context(|| format!("не удалось прочитать {source:?}"))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .and_then(|e| e.parse::<u32>().ok())
                .is_some()
        })
        .collect();
    files.sort();
    if files.is_empty() {
        return Err(anyhow!("в {source:?} не найдено CA-сертификатов"));
    }

    let mut bundle = Vec::new();
    for file in files {
        let data = fs::read(&file)
            .with_context(|| format!("не удалось прочитать CA-сертификат {file:?}"))?;
        bundle.extend_from_slice(&data);
        if !data.ends_with(b"\n") {
            bundle.push(b'\n');
        }
    }

    let bundle_path = cache_dir.join("git-folder-sync-android-ca-bundle.pem");
    fs::write(&bundle_path, bundle)
        .with_context(|| format!("не удалось создать CA bundle {bundle_path:?}"))?;
    Ok(bundle_path)
}

#[cfg(target_os = "android")]
fn ensure_android_ca_bundle_ready() -> Result<()> {
    match ANDROID_CA_BUNDLE_INIT.get() {
        Some(Ok(())) => Ok(()),
        Some(Err(message)) => Err(anyhow!(message.clone())),
        None => Err(anyhow!("CA bundle Android ещё не был инициализирован")),
    }
}

#[cfg(not(target_os = "android"))]
fn ensure_android_ca_bundle_ready() -> Result<()> {
    Ok(())
}

fn stored_token(remote_url: &str, username: &str) -> Result<String> {
    ensure_keyring_store()?;
    let entry = Entry::new("git-folder-sync", &credential_key(remote_url, username))?;
    entry.get_password().context("PAT не найден в защищённом хранилище")
}

fn save_token(remote_url: &str, username: &str, token: &str) -> Result<()> {
    ensure_keyring_store()?;
    if token.trim().is_empty() {
        return Err(anyhow!("Токен пустой"));
    }
    let entry = Entry::new("git-folder-sync", &credential_key(remote_url, username))?;
    entry.set_password(token)?;
    Ok(())
}

fn effective_config(mut cfg: Config) -> Result<Config> {
    if cfg.remote_url.trim().is_empty() {
        return Err(anyhow!("Git remote URL не указан"));
    }
    if cfg.branch.trim().is_empty() {
        cfg.branch = "main".into();
    }
    if cfg.token.is_empty() {
        cfg.token = stored_token(&cfg.remote_url, &cfg.username)?;
    }
    Ok(cfg)
}

#[tauri::command]
fn save_credentials(config: Config) -> Result<String, String> {
    save_token(&config.remote_url, &config.username, &config.token)
        .map(|_| "Токен сохранён в защищённом хранилище ОС.".into())
        .map_err(|e| format!("{e:#}"))
}

#[tauri::command]
fn delete_credentials(config: Config) -> Result<String, String> {
    ensure_keyring_store().map_err(|e| format!("{e:#}"))?;
    let entry = Entry::new("git-folder-sync", &credential_key(&config.remote_url, &config.username))
        .map_err(|e| format!("{e:#}"))?;
    match entry.delete_credential() {
        Ok(_) => Ok("Токен удалён.".into()),
        Err(keyring::Error::NoEntry) => Ok("Сохранённого токена не было.".into()),
        Err(e) => Err(format!("{e:#}")),
    }
}

fn callbacks(cfg: &Config) -> RemoteCallbacks<'static> {
    let user = cfg.username.clone();
    let token = cfg.token.clone();
    let mut cb = RemoteCallbacks::new();
    cb.credentials(move |_url, username_from_url, allowed| {
        if allowed.contains(CredentialType::USER_PASS_PLAINTEXT) {
            Cred::userpass_plaintext(
                if user.is_empty() { username_from_url.unwrap_or("git") } else { &user },
                &token,
            )
        } else {
            Err(git2::Error::from_str("HTTPS username/password credentials unavailable"))
        }
    });
    cb
}

fn open_repo(cfg: &Config) -> Result<Repository> {
    Repository::open(&cfg.folder).context("Папка не является Git-репозиторием")
}

#[tauri::command]
fn clone_or_initialize(config: Config) -> Result<String, String> {
    let config = effective_config(config).map_err(|e| format!("{e:#}"))?;
    ensure_android_ca_bundle_ready().map_err(|e| format!("{e:#}"))?;
    (|| -> Result<String> {
        let p = Path::new(&config.folder);
        std::fs::create_dir_all(p)?;

        if p.join(".git").exists() {
            let repo = Repository::open(p)?;
            let remote = repo.find_remote("origin").or_else(|_| {
                repo.remote("origin", &config.remote_url)?;
                repo.find_remote("origin")
            })?;
            if remote.url()? != config.remote_url {
                repo.remote_set_url("origin", &config.remote_url)?;
            }
            return Ok("Git-репозиторий уже существует.".into());
        }

        let is_empty = std::fs::read_dir(p)?.next().is_none();
        if is_empty {
            let mut fo = FetchOptions::new();
            fo.remote_callbacks(callbacks(&config));
            let mut builder = git2::build::RepoBuilder::new();
            builder.fetch_options(fo);
            builder.branch(&config.branch);
            builder.clone(&config.remote_url, p)?;
            return Ok("Репозиторий клонирован.".into());
        }

        let repo = Repository::init(p)?;
        repo.set_head(&format!("refs/heads/{}", config.branch))?;
        repo.remote("origin", &config.remote_url)?;
        Ok("Локальный репозиторий создан. Нажмите Push, чтобы отправить файлы.".into())
    })()
    .map_err(|e| format!("{e:#}"))
}

fn working_tree_dirty(repo: &Repository) -> Result<bool> {
    Ok(!repo.statuses(None)?.is_empty())
}

#[tauri::command]
fn push_repo(config: Config) -> Result<String, String> {
    let config = effective_config(config).map_err(|e| format!("{e:#}"))?;
    ensure_android_ca_bundle_ready().map_err(|e| format!("{e:#}"))?;
    (|| -> Result<String> {
        let repo = open_repo(&config)?;
        let branch_ref = format!("refs/heads/{}", config.branch);
        let head_before = repo
            .find_reference(&branch_ref)
            .ok()
            .and_then(|r| r.target());

        let mut index = repo.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        if let Some(head_id) = head_before {
            let head_commit = repo.find_commit(head_id)?;
            if head_commit.tree_id() == tree_id {
                return Ok("Изменений нет: новый commit не создавался.".into());
            }
        }

        let sig = Signature::now("Git Folder Sync", "git-folder-sync@localhost")?;
        let message = "Sync from Git Folder Sync";
        if let Some(head_id) = head_before {
            let parent = repo.find_commit(head_id)?;
            repo.commit(Some(&branch_ref), &sig, &sig, message, &tree, &[&parent])?;
        } else {
            repo.commit(Some(&branch_ref), &sig, &sig, message, &tree, &[])?;
        }
        repo.set_head(&branch_ref)?;

        let mut remote = repo.find_remote("origin").or_else(|_| {
            repo.remote("origin", &config.remote_url)?;
            repo.find_remote("origin")
        })?;
        let mut po = PushOptions::new();
        po.remote_callbacks(callbacks(&config));
        let refspec = format!("refs/heads/{}:refs/heads/{}", config.branch, config.branch);
        remote.push(&[refspec.as_str()], Some(&mut po))?;
        Ok("Push выполнен успешно.".into())
    })()
    .map_err(|e| format!("{e:#}"))
}

#[tauri::command]
fn pull_repo(config: Config) -> Result<String, String> {
    let config = effective_config(config).map_err(|e| format!("{e:#}"))?;
    ensure_android_ca_bundle_ready().map_err(|e| format!("{e:#}"))?;
    (|| -> Result<String> {
        let repo = open_repo(&config)?;
        if working_tree_dirty(&repo)? {
            return Err(anyhow!("Локальная папка содержит несохранённые изменения. Сначала выполните Push или приведите её в чистое состояние."));
        }

        let mut remote = repo.find_remote("origin")?;
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(callbacks(&config));
        remote.fetch(&[config.branch.as_str()], Some(&mut fo), None)?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        let analysis = repo.merge_analysis(&[&fetch_commit])?;

        if analysis.0.is_up_to_date() {
            return Ok("Уже актуально.".into());
        }
        if analysis.0.is_fast_forward() {
            let refname = format!("refs/heads/{}", config.branch);
            let mut r = repo.find_reference(&refname)?;
            r.set_target(fetch_commit.id(), "Fast-forward pull")?;
            repo.set_head(&refname)?;
            repo.checkout_head(None)?;
            return Ok("Pull выполнен (fast-forward).".into());
        }
        Err(anyhow!("Локальная и удалённая ветки разошлись. Автоматическое слияние отключено."))
    })()
    .map_err(|e| format!("{e:#}"))
}

#[tauri::command]
fn repo_status(config: Config) -> Result<String, String> {
    let config = effective_config(config).map_err(|e| format!("{e:#}"))?;
    (|| -> Result<String> {
        let repo = open_repo(&config)?;
        let statuses = repo.statuses(None)?;
        if statuses.is_empty() {
            return Ok("Чисто: изменений нет.".into());
        }
        let mut out = String::new();
        for s in statuses.iter() {
            out.push_str(&format!("{:?}  {}\n", s.status(), s.path().unwrap_or("?")));
        }
        Ok(out)
    })()
    .map_err(|e| format!("{e:#}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());

    #[cfg(target_os = "android")]
    let builder = builder.plugin(
            tauri::plugin::Builder::<tauri::Wry, ()>::new("android-saf")
                .setup(|_app, api| {
                    api.register_android_plugin("com.gitfoldersync.saf", "AndroidSafPlugin")?;
                    Ok(())
                })
                .build(),
        );

    builder
        .setup(|app| {
            #[cfg(target_os = "android")]
            {
                let initialization: std::result::Result<(), String> = (|| {
                    let cache_dir = app
                        .path()
                        .app_cache_dir()
                        .map_err(|e| format!("не удалось получить приватный каталог кэша приложения: {e}"))?;
                    fs::create_dir_all(&cache_dir).map_err(|e| {
                        format!("не удалось создать приватный каталог кэша {cache_dir:?}: {e}")
                    })?;

                    let bundle_path = build_android_ca_bundle(&cache_dir)
                        .map_err(|e| format!("{e:#}"))?;
                    // SAFETY: this libgit2 global option is set during Tauri setup,
                    // before the event loop and before any app Git command can run.
                    unsafe {
                        git2::opts::set_ssl_cert_file(&bundle_path)
                            .map_err(|e| format!("не удалось настроить CA bundle для HTTPS: {e}"))?;
                    }
                    Ok(())
                })();
                let _ = ANDROID_CA_BUNDLE_INIT.set(initialization);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_credentials,
            delete_credentials,
            clone_or_initialize,
            push_repo,
            pull_repo,
            repo_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");
}

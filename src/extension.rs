use zed_extension_api::{self as zed, Result};

struct MinijinjaExtension {
    cached_binary_path: Option<String>,
}

impl zed::Extension for MinijinjaExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        if let Some(path) = &self.cached_binary_path {
            if std::fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(zed::Command {
                    command: path.clone(),
                    args: vec![],
                    env: vec![],
                });
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "uros-5/jinja-lsp",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == "jinja-lsp.zip")
            .ok_or_else(|| format!("No jinja-lsp.zip asset found in release {}", release.version))?;

        let version_dir = format!("jinja-lsp-{}", release.version);
        std::fs::create_dir_all(&version_dir)
            .map_err(|e| format!("failed to create directory: {}", e))?;

        let binary_name = match (platform, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => "jinja-lsp-darwin-arm64",
            (zed::Os::Mac, _) => "jinja-lsp-darwin-x64",
            (zed::Os::Linux, _) => "jinja-lsp-linux-x64",
            (zed::Os::Windows, zed::Architecture::Aarch64) => "jinja-lsp-windows-arm64.exe",
            (zed::Os::Windows, _) => "jinja-lsp-windows-x64.exe",
        };
        let binary_path = format!("{version_dir}/{binary_name}");

        if !std::fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            // Download the file. Note that depending on the release format, this could be
            // a tar.gz, zip, or raw executable. If it's a tar.gz/zip we need to extract.
            // If the name has .tar.gz, we use DownloadedFileType::GzipTar
            let file_type = if asset.name.ends_with(".tar.gz") {
                zed::DownloadedFileType::GzipTar
            } else if asset.name.ends_with(".zip") {
                zed::DownloadedFileType::Zip
            } else {
                zed::DownloadedFileType::Uncompressed
            };
            
            zed::download_file(&asset.download_url, &version_dir, file_type)
                .map_err(|e| format!("failed to download file: {e}"))?;

            zed::make_file_executable(&binary_path)
                .map_err(|e| format!("failed to make file executable: {}", e))?;

            // Cleanup old versions
            if let Ok(entries) = std::fs::read_dir(".") {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && entry.file_name().to_string_lossy() != version_dir {
                        let _ = std::fs::remove_dir_all(&path);
                    }
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());

        Ok(zed::Command {
            command: binary_path,
            args: vec![],
            env: Default::default(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings = zed::settings::LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_else(|| {
                zed::serde_json::json!({
                    "templates": "./templates",
                    "backend": ["./src"],
                    "lang": "rust"
                })
            });

        Ok(Some(settings))
    }
}

zed::register_extension!(MinijinjaExtension);

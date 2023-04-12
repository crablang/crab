use anyhow::Error;
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Archive;

const DEFAULT_TARGET: &str = "x86_64-unknown-linux-gnu";

macro_rules! pkg_type {
    ( $($variant:ident = $component:literal $(; preview = true $(@$is_preview:tt)? )? ),+ $(,)? ) => {
        #[derive(Debug, Hash, Eq, PartialEq, Clone)]
        pub(crate) enum PkgType {
            $($variant,)+
        }

        impl PkgType {
            pub(crate) fn is_preview(&self) -> bool {
                match self {
                    $( $( $($is_preview)? PkgType::$variant => true, )? )+
                    _ => false,
                }
            }

            /// First part of the tarball name.
            pub(crate) fn tarball_component_name(&self) -> &str {
                match self {
                    $( PkgType::$variant => $component,)+
                }
            }

            pub(crate) fn all() -> &'static [PkgType] {
                &[ $(PkgType::$variant),+ ]
            }
        }
    }
}

pkg_type! {
    CrabLang = "crablang",
    CrabLangSrc = "crablang-src",
    CrabLangc = "crablangc",
    CrabLangcDev = "crablangc-dev",
    CrabLangcDocs = "crablangc-docs",
    ReproducibleArtifacts = "reproducible-artifacts",
    CrabLangMingw = "crablang-mingw",
    CrabLangStd = "crablang-std",
    Cargo = "cargo",
    HtmlDocs = "crablang-docs",
    CrabLangAnalysis = "crablang-analysis",
    Rls = "rls"; preview = true,
    CrabLangAnalyzer = "crablang-analyzer"; preview = true,
    Clippy = "clippy"; preview = true,
    CrabLangfmt = "crablangfmt"; preview = true,
    LlvmTools = "llvm-tools"; preview = true,
    Miri = "miri"; preview = true,
    JsonDocs = "crablang-docs-json"; preview = true,
}

impl PkgType {
    /// Component name in the manifest. In particular, this includes the `-preview` suffix where appropriate.
    pub(crate) fn manifest_component_name(&self) -> String {
        if self.is_preview() {
            format!("{}-preview", self.tarball_component_name())
        } else {
            self.tarball_component_name().to_string()
        }
    }

    /// Whether this package has the same version as CrabLang itself, or has its own `version` and
    /// `git-commit-hash` files inside the tarball.
    fn should_use_crablang_version(&self) -> bool {
        match self {
            PkgType::Cargo => false,
            PkgType::Rls => false,
            PkgType::CrabLangAnalyzer => false,
            PkgType::Clippy => false,
            PkgType::CrabLangfmt => false,
            PkgType::LlvmTools => false,
            PkgType::Miri => false,

            PkgType::CrabLang => true,
            PkgType::CrabLangStd => true,
            PkgType::CrabLangSrc => true,
            PkgType::CrabLangc => true,
            PkgType::JsonDocs => true,
            PkgType::HtmlDocs => true,
            PkgType::CrabLangcDev => true,
            PkgType::CrabLangcDocs => true,
            PkgType::ReproducibleArtifacts => true,
            PkgType::CrabLangMingw => true,
            PkgType::CrabLangAnalysis => true,
        }
    }

    pub(crate) fn targets(&self) -> &[&str] {
        use crate::{HOSTS, MINGW, TARGETS};
        use PkgType::*;

        match self {
            CrabLang => HOSTS, // doesn't matter in practice, but return something to avoid panicking
            CrabLangc => HOSTS,
            CrabLangcDev => HOSTS,
            ReproducibleArtifacts => HOSTS,
            CrabLangcDocs => HOSTS,
            Cargo => HOSTS,
            CrabLangMingw => MINGW,
            CrabLangStd => TARGETS,
            HtmlDocs => HOSTS,
            JsonDocs => HOSTS,
            CrabLangSrc => &["*"],
            Rls => HOSTS,
            CrabLangAnalyzer => HOSTS,
            Clippy => HOSTS,
            Miri => HOSTS,
            CrabLangfmt => HOSTS,
            CrabLangAnalysis => TARGETS,
            LlvmTools => TARGETS,
        }
    }

    /// Whether this package is target-independent or not.
    fn target_independent(&self) -> bool {
        *self == PkgType::CrabLangSrc
    }

    /// Whether to package these target-specific docs for another similar target.
    pub(crate) fn use_docs_fallback(&self) -> bool {
        match self {
            PkgType::JsonDocs | PkgType::HtmlDocs => true,
            _ => false,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct VersionInfo {
    pub(crate) version: Option<String>,
    pub(crate) git_commit: Option<String>,
    pub(crate) present: bool,
}

pub(crate) struct Versions {
    channel: String,
    dist_path: PathBuf,
    versions: HashMap<PkgType, VersionInfo>,
}

impl Versions {
    pub(crate) fn new(channel: &str, dist_path: &Path) -> Result<Self, Error> {
        Ok(Self { channel: channel.into(), dist_path: dist_path.into(), versions: HashMap::new() })
    }

    pub(crate) fn channel(&self) -> &str {
        &self.channel
    }

    pub(crate) fn version(&mut self, mut package: &PkgType) -> Result<VersionInfo, Error> {
        if package.should_use_crablang_version() {
            package = &PkgType::CrabLang;
        }

        match self.versions.get(package) {
            Some(version) => Ok(version.clone()),
            None => {
                let version_info = self.load_version_from_tarball(package)?;
                if *package == PkgType::CrabLang && version_info.version.is_none() {
                    panic!("missing version info for toolchain");
                }
                self.versions.insert(package.clone(), version_info.clone());
                Ok(version_info)
            }
        }
    }

    fn load_version_from_tarball(&mut self, package: &PkgType) -> Result<VersionInfo, Error> {
        let tarball_name = self.tarball_name(package, DEFAULT_TARGET)?;
        let tarball = self.dist_path.join(tarball_name);

        let file = match File::open(&tarball) {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                // Missing tarballs do not return an error, but return empty data.
                println!("warning: missing tarball {}", tarball.display());
                return Ok(VersionInfo::default());
            }
            Err(err) => return Err(err.into()),
        };
        let mut tar = Archive::new(GzDecoder::new(file));

        let mut version = None;
        let mut git_commit = None;
        for entry in tar.entries()? {
            let mut entry = entry?;

            let dest;
            match entry.path()?.components().nth(1).and_then(|c| c.as_os_str().to_str()) {
                Some("version") => dest = &mut version,
                Some("git-commit-hash") => dest = &mut git_commit,
                _ => continue,
            }
            let mut buf = String::new();
            entry.read_to_string(&mut buf)?;
            *dest = Some(buf);

            // Short circuit to avoid reading the whole tar file if not necessary.
            if version.is_some() && git_commit.is_some() {
                break;
            }
        }

        Ok(VersionInfo { version, git_commit, present: true })
    }

    pub(crate) fn archive_name(
        &self,
        package: &PkgType,
        target: &str,
        extension: &str,
    ) -> Result<String, Error> {
        let component_name = package.tarball_component_name();
        let version = match self.channel.as_str() {
            "stable" => self.crablangc_version().into(),
            "beta" => "beta".into(),
            "nightly" => "nightly".into(),
            _ => format!("{}-dev", self.crablangc_version()),
        };

        if package.target_independent() {
            Ok(format!("{}-{}.{}", component_name, version, extension))
        } else {
            Ok(format!("{}-{}-{}.{}", component_name, version, target, extension))
        }
    }

    pub(crate) fn tarball_name(&self, package: &PkgType, target: &str) -> Result<String, Error> {
        self.archive_name(package, target, "tar.gz")
    }

    pub(crate) fn crablangc_version(&self) -> &str {
        const CRABLANGC_VERSION: &str = include_str!("../../../version");
        CRABLANGC_VERSION.trim()
    }
}

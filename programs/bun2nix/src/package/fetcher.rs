//! This module holds the implementation for data about a given nix fetcher type

use std::{fmt::Debug, hash::Hash};

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::{
    Options,
    error::{Error, Result},
};

#[derive(Template, Debug, Serialize, Deserialize, Clone, Eq, Ord, PartialEq, PartialOrd, Hash)]
/// # Package Fetcher
///
/// Nix-translated fetcher for a given package
pub enum Fetcher {
    /// A package which must be retrieved with nix's `pkgs.fetchurl`
    #[template(path = "fetchurl.nix_template")]
    FetchUrl {
        /// The url to fetch the package from
        url: String,
        /// The hash of the downloaded results
        /// This can be derived from the bun lockfile
        hash: String,
    },
    /// A package which must be retrieved with nix's `pkgs.fetchgit`
    #[template(path = "fetchgit.nix_template")]
    FetchGit {
        /// The url to fetch the package from
        url: String,
        /// The commit ref to fetch
        rev: String,
        /// The hash of the downloaded results
        /// This must be calculated via nix-prefetch
        hash: String,
    },
    /// A package which must be retrieved with nix's `pkgs.fetchFromGitHub`
    #[template(path = "fetchgithub.nix_template")]
    FetchGitHub {
        /// The owner of the repo to fetch from
        owner: String,
        /// The repo to fetch
        repo: String,
        /// The git ref to fetch
        rev: String,
        /// The hash of the downloaded results
        /// This must be calculated via nix-prefetch
        hash: String,
    },
    /// A package which must be retrieved with nix's `pkgs.fetchtarball`
    #[template(path = "fetchtarball.nix_template")]
    FetchTarball {
        /// The url to fetch the package from
        url: String,
        /// The hash of the downloaded results
        /// This must be calculated via nix-prefetch
        hash: String,
    },
    /// A package can be a path copied to the store directly
    #[template(path = "copy-to-store.nix_template")]
    CopyToStore {
        /// The path from the root to copy to the store
        path: String,
    },
}

/// The default NPM registry URL
pub const DEFAULT_REGISTRY: &str = "https://registry.npmjs.org/";

impl Fetcher {
    /// # From NPM Package Name
    ///
    /// Initialize a fetcher from an npm identifier and
    /// it's hash, optionally using a custom registry path
    ///
    /// ## Arguments
    /// * `ident` - The package identifier (e.g., "@types/node@1.0.0")
    /// * `hash` - The integrity hash of the package
    /// * `registry_path` - Optional registry path from bun.lock. Can be:
    ///   - None or empty: uses the default npmjs.org registry
    ///   - Full tarball URL (ends with .tgz): used directly
    ///   - Base registry URL: package path is appended
    pub fn new_npm_package(ident: &str, hash: String, registry_path: Option<&str>) -> Result<Self> {
        let url = Self::to_npm_url(ident, registry_path)?;

        Ok(Self::FetchUrl { url, hash })
    }

    /// # NPM url converter
    ///
    /// Produce a url needed to fetch from the npm api from a package
    ///
    /// ## Usage
    ///```rust
    /// use bun2nix::package::Fetcher;
    ///
    /// // Default registry
    /// let npm_identifier = "@alloc/quick-lru@5.2.0";
    ///
    /// assert_eq!(
    ///     Fetcher::to_npm_url(npm_identifier, None).unwrap(),
    ///     "https://registry.npmjs.org/@alloc/quick-lru/-/quick-lru-5.2.0.tgz"
    /// );
    ///
    /// // Custom registry (base URL)
    /// assert_eq!(
    ///     Fetcher::to_npm_url(npm_identifier, Some("https://npm.pkg.github.com/")).unwrap(),
    ///     "https://npm.pkg.github.com/@alloc/quick-lru/-/quick-lru-5.2.0.tgz"
    /// );
    ///
    /// // Unscoped package with custom registry
    /// assert_eq!(
    ///     Fetcher::to_npm_url("lodash@4.17.21", Some("https://npm.example.com")).unwrap(),
    ///     "https://npm.example.com/lodash/-/lodash-4.17.21.tgz"
    /// );
    ///
    /// // Full tarball URL (used directly)
    /// assert_eq!(
    ///     Fetcher::to_npm_url("lodash@4.17.21", Some("https://npm.pkg.github.com/lodash/-/lodash-4.17.21.tgz")).unwrap(),
    ///     "https://npm.pkg.github.com/lodash/-/lodash-4.17.21.tgz"
    /// );
    /// ```
    pub fn to_npm_url(ident: &str, registry_path: Option<&str>) -> Result<String> {
        // If registry_path is a full tarball URL, use it directly
        if let Some(path) = registry_path {
            if !path.is_empty() && path.ends_with(".tgz") {
                return Ok(path.to_string());
            }
        }

        // Determine the base registry URL
        let base_url = match registry_path {
            Some(url) if !url.is_empty() => {
                // Ensure the registry URL ends with a slash
                if url.ends_with('/') {
                    url.to_string()
                } else {
                    format!("{}/", url)
                }
            }
            _ => DEFAULT_REGISTRY.to_string(),
        };

        // Construct the tarball URL from the package identifier
        let Some((user, name_and_ver)) = ident.split_once("/") else {
            let Some((name, ver)) = ident.split_once("@") else {
                return Err(Error::NoAtInPackageIdentifier);
            };

            return Ok(format!(
                "{}{}/-/{}-{}.tgz",
                base_url, name, name, ver
            ));
        };

        let Some((name, ver)) = name_and_ver.split_once("@") else {
            return Err(Error::NoAtInPackageIdentifier);
        };

        Ok(format!(
            "{}{}/{}/-/{}-{}.tgz",
            base_url, user, name, name, ver
        ))
    }
}

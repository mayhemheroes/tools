mod package_json;

use crate::diagnostics::ProjectDiagnostic;
use crate::node_js_project::package_json::PackageJson;
use crate::{Manifest, Project};
use rome_fs::{FileSystem, OpenOptions};
use std::path::{Path, PathBuf};

/// A Node.js project.
pub struct NodeJsProject {
    /// The path where the project
    manifest_path: PathBuf,
    /// The `package.json` manifest
    manifest: PackageJson,
}

impl Project for NodeJsProject {
    type Manifest = PackageJson;

    fn load_project(&mut self, fs: impl FileSystem) -> Result<(), ProjectDiagnostic> {
        let options = OpenOptions::default().read(true);
        let mut file = fs.open_with_options(self.manifest_path.as_path(), options)?;
        let manifest = Self::Manifest::load_manifest(self.manifest_path.as_path(), &mut file)?;
        self.manifest = manifest;
        Ok(())
    }

    fn project_path(&self) -> &Path {
        self.manifest_path.as_path()
    }

    fn manifest(&self) -> Option<&Self::Manifest> {
        Some(&self.manifest)
    }
}

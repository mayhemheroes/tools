mod diagnostics;
mod license;
mod node_js_project;

use std::fmt::Debug;
use std::path::Path;

use crate::diagnostics::ProjectDiagnostic;
pub use node_js_project::NodeJsProject;
use rome_fs::{File, FileSystem};

pub trait Manifest: Default + Debug {
    /// It loads the manifest. It accepts the path where the manifest should be
    fn load_manifest(file_path: &Path, file: &mut Box<dyn File>)
        -> Result<Self, ProjectDiagnostic>;
}

/// An internal representation of a project.
pub trait Project {
    type Manifest: Manifest;

    /// Use this function to prepare the project, like loading the manifest.
    fn load_project(&mut self, fs: impl FileSystem) -> Result<(), ProjectDiagnostic>;

    /// The home directory of the project
    fn project_path(&self) -> &Path;

    /// A generic manifest of a project
    fn manifest(&self) -> Option<&Self::Manifest> {
        None
    }
}

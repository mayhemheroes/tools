use crate::diagnostics::ProjectDiagnostic;
use crate::Manifest;
use node_semver::{SemverError, Version};
use rome_deserialize::json::{
    deserialize_from_json_str, has_only_known_keys, JsonDeserialize, VisitJsonNode,
};
use rome_deserialize::{DeserializationDiagnostic, VisitNode};
use rome_fs::File;
use rome_json_syntax::{AnyJsonValue, JsonLanguage, JsonRoot, JsonStringValue, JsonSyntaxNode};
use rome_rowan::{AstNode, SyntaxNode};
use rome_text_size::{TextRange, TextSize};
use rustc_hash::FxHashMap;
use std::ops::Add;
use std::path::Path;

#[derive(Debug, Default)]
pub struct PackageJson {
    version: Option<Version>,
    name: Option<String>,
    description: Option<String>,
    dependencies: Dependencies,
    dev_dependencies: Dependencies,
    optional_dependencies: Dependencies,
    license: Option<String>,
}

impl Manifest for PackageJson {
    fn load_manifest(
        file_path: &Path,
        file: &mut Box<dyn File>,
    ) -> Result<Self, ProjectDiagnostic> {
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        let deserialized = deserialize_from_json_str::<PackageJson>(buffer.as_str());
        if deserialized.has_errors() {
            return Err(ProjectDiagnostic::new_failed_deserialization(
                file_path.display().to_string().as_str(),
                deserialized.into_diagnostics(),
            ));
        }

        Ok(deserialized.into_deserialized())
    }
}

#[derive(Debug, Default)]
pub struct Dependencies(FxHashMap<String, Version>);

impl PackageJson {
    const KNOWN_FIELDS: &'static [&'static str] = &[
        "name",
        "description",
        "dependencies",
        "devDependencies",
        "optionalDependencies",
        "license",
        "version",
    ];
}

impl JsonDeserialize for PackageJson {
    fn deserialize_from_ast(
        root: JsonRoot,
        visitor: &mut impl VisitJsonNode,
        diagnostics: &mut Vec<DeserializationDiagnostic>,
    ) -> Option<()> {
        let value = root.value().ok()?;
        match value {
            AnyJsonValue::JsonObjectValue(node) => {
                for element in node.json_member_list() {
                    let element = element.ok()?;
                    let member_name = element.name().ok()?;
                    let member_value = element.value().ok()?;
                    visitor.visit_map(member_name.syntax(), member_value.syntax(), diagnostics)?;
                }
                Some(())
            }
            _ => {
                diagnostics.push(
                    DeserializationDiagnostic::new("The manifest should be an object")
                        .with_range(root.range()),
                );
                None
            }
        }
    }
}

impl VisitJsonNode for PackageJson {}

impl VisitNode<JsonLanguage> for PackageJson {
    fn visit_member_name(
        &mut self,
        node: &JsonSyntaxNode,
        diagnostics: &mut Vec<DeserializationDiagnostic>,
    ) -> Option<()> {
        has_only_known_keys(node, PackageJson::KNOWN_FIELDS, diagnostics)
    }

    fn visit_map(
        &mut self,
        key: &JsonSyntaxNode,
        value: &JsonSyntaxNode,
        diagnostics: &mut Vec<DeserializationDiagnostic>,
    ) -> Option<()> {
        let (name, value) = self.get_key_and_value(key, value, diagnostics)?;
        let name_text = name.text();
        match name_text {
            "version" => {
                let version = self.map_to_string(&value, name_text, diagnostics)?;
                self.version = parse_to_version(&version, value.syntax(), diagnostics);
            }
            "name" => {
                self.name = self.map_to_string(&value, name_text, diagnostics);
            }
            "license" => {
                self.license = self.map_to_string(&value, name_text, diagnostics);
            }
            "description" => {
                self.description = self.map_to_string(&value, name_text, diagnostics);
            }
            "dependencies" => {
                let mut dependencies = Dependencies::default();
                self.map_to_object(&value, name_text, &mut dependencies, diagnostics)?;
                self.dependencies = dependencies;
            }
            "devDependencies" => {
                let mut dev_dependencies = Dependencies::default();
                self.map_to_object(&value, name_text, &mut dev_dependencies, diagnostics)?;
                self.dev_dependencies = dev_dependencies;
            }
            "optionalDependencies" => {
                let mut optional_dependencies = Dependencies::default();
                self.map_to_object(&value, name_text, &mut optional_dependencies, diagnostics)?;
                self.optional_dependencies = optional_dependencies;
            }
            _ => {}
        }
        Some(())
    }
}

impl VisitJsonNode for Dependencies {}
impl VisitNode<JsonLanguage> for Dependencies {
    fn visit_member_name(
        &mut self,
        _node: &SyntaxNode<JsonLanguage>,
        _diagnostics: &mut Vec<DeserializationDiagnostic>,
    ) -> Option<()> {
        Some(())
    }
    fn visit_map(
        &mut self,
        key: &JsonSyntaxNode,
        value: &JsonSyntaxNode,
        diagnostics: &mut Vec<DeserializationDiagnostic>,
    ) -> Option<()> {
        let (name, value) = self.get_key_and_value(key, value, diagnostics)?;
        let name_text = name.text();

        let value = JsonStringValue::cast_ref(value.syntax()).or_else(|| {
            diagnostics.push(DeserializationDiagnostic::new_incorrect_type_for_value(
                name_text,
                "string",
                value.range(),
            ));
            None
        })?;
        let version = value.inner_string_text().ok()?;
        let version = parse_to_version(version.text(), value.syntax(), diagnostics);
        if let Some(version) = version {
            self.0.insert(name_text.to_string(), version);
        }

        Some(())
    }
}

fn parse_to_version(
    version: &str,
    value: &JsonSyntaxNode,
    diagnostics: &mut Vec<DeserializationDiagnostic>,
) -> Option<Version> {
    let result: Result<Version, SemverError> = version.parse();
    match result {
        Ok(version) => Some(version),
        Err(err) => {
            let (start, end) = err.location();
            let start_range = value.text_trimmed_range().start();
            let end_range = value.text_trimmed_range().end();
            let range = TextRange::new(
                start_range.add(TextSize::from(start as u32)),
                end_range.add(TextSize::from(end as u32)),
            );
            diagnostics
                .push(DeserializationDiagnostic::new(err.kind().to_string()).with_range(range));
            None
        }
    }
}

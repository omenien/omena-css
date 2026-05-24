use std::{fs, path::PathBuf, time::SystemTime};

use super::*;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn extracts_vite_object_aliases_from_define_config() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("omena_bridge_vite_alias_define_config")?;
    let config_path = root.join("vite.config.ts");
    let source = r#"
        import { defineConfig } from "vite";
        export default defineConfig({
          resolve: {
            alias: {
              "@styles": "./src/styles",
              "@root": path.resolve(__dirname, "src")
            }
          }
        });
    "#;

    let summary =
        summarize_omena_bridge_bundler_path_aliases_for_config(config_path.as_path(), source);

    assert_eq!(summary.unrecognized, Vec::new());
    assert_eq!(summary.aliases.len(), 2);
    assert_eq!(summary.aliases[0].pattern, "@styles");
    assert_eq!(
        summary.aliases[0].target_path,
        root.join("src/styles").to_string_lossy()
    );
    assert_eq!(summary.aliases[1].pattern, "@root");
    assert_eq!(
        summary.aliases[1].target_path,
        root.join("src").to_string_lossy()
    );
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn extracts_webpack_array_aliases_from_module_exports() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("omena_bridge_webpack_alias_array")?;
    let config_path = root.join("webpack.config.js");
    let source = r#"
        module.exports = {
          resolve: {
            alias: [
              { find: "@theme", replacement: "./src/theme" }
            ]
          }
        };
    "#;

    let summary =
        summarize_omena_bridge_bundler_path_aliases_for_config(config_path.as_path(), source);

    assert_eq!(summary.unrecognized, Vec::new());
    assert_eq!(summary.aliases.len(), 1);
    assert_eq!(summary.aliases[0].pattern, "@theme");
    assert_eq!(
        summary.aliases[0].target_path,
        root.join("src/theme").to_string_lossy()
    );
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn preserves_webpack_array_alias_declaration_order() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("omena_bridge_webpack_alias_array_order")?;
    let config_path = root.join("webpack.config.js");
    let source = r#"
        module.exports = {
          resolve: {
            alias: [
              { find: "@theme", replacement: "./src/first" },
              { find: "@theme", replacement: "./src/second" }
            ]
          }
        };
    "#;

    let summary =
        summarize_omena_bridge_bundler_path_aliases_for_config(config_path.as_path(), source);

    assert_eq!(summary.unrecognized, Vec::new());
    assert_eq!(summary.aliases.len(), 2);
    assert_eq!(summary.aliases[0].pattern, "@theme");
    assert_eq!(
        summary.aliases[0].target_path,
        root.join("src/first").to_string_lossy()
    );
    assert_eq!(summary.aliases[1].pattern, "@theme");
    assert_eq!(
        summary.aliases[1].target_path,
        root.join("src/second").to_string_lossy()
    );
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn extracts_vite_aliases_from_top_level_config_identifier() -> TestResult {
    let root = temp_dir("omena_bridge_vite_alias_config_identifier")?;
    let config_path = root.join("vite.config.ts");
    let source = r#"
        import { defineConfig } from "vite";
        const config = {
          resolve: {
            alias: {
              "@shared": "./src/shared"
            }
          }
        };
        export default defineConfig(config);
    "#;

    let summary =
        summarize_omena_bridge_bundler_path_aliases_for_config(config_path.as_path(), source);

    assert_eq!(summary.unrecognized, Vec::new());
    assert_eq!(summary.aliases.len(), 1);
    assert_eq!(summary.aliases[0].pattern, "@shared");
    assert_eq!(
        summary.aliases[0].target_path,
        root.join("src/shared").to_string_lossy()
    );
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn extracts_object_aliases_from_top_level_alias_identifier() -> TestResult {
    let root = temp_dir("omena_bridge_vite_alias_object_identifier")?;
    let config_path = root.join("vite.config.ts");
    let source = r#"
        const aliases = {
          "@tokens": "./src/tokens"
        };
        export default {
          resolve: {
            alias: aliases
          }
        };
    "#;

    let summary =
        summarize_omena_bridge_bundler_path_aliases_for_config(config_path.as_path(), source);

    assert_eq!(summary.unrecognized, Vec::new());
    assert_eq!(summary.aliases.len(), 1);
    assert_eq!(summary.aliases[0].pattern, "@tokens");
    assert_eq!(
        summary.aliases[0].target_path,
        root.join("src/tokens").to_string_lossy()
    );
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn extracts_array_aliases_from_top_level_alias_identifier() -> TestResult {
    let root = temp_dir("omena_bridge_webpack_alias_array_identifier")?;
    let config_path = root.join("webpack.config.js");
    let source = r#"
        const aliases = [
          { find: "@icons", replacement: "./src/icons" }
        ];
        module.exports = {
          resolve: {
            alias: aliases
          }
        };
    "#;

    let summary =
        summarize_omena_bridge_bundler_path_aliases_for_config(config_path.as_path(), source);

    assert_eq!(summary.unrecognized, Vec::new());
    assert_eq!(summary.aliases.len(), 1);
    assert_eq!(summary.aliases[0].pattern, "@icons");
    assert_eq!(
        summary.aliases[0].target_path,
        root.join("src/icons").to_string_lossy()
    );
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn marks_dynamic_alias_identifier_unrecognized() -> TestResult {
    let root = temp_dir("omena_bridge_alias_dynamic_identifier")?;
    let config_path = root.join("vite.config.ts");
    let source = r#"
        const aliases = buildAliases();
        export default {
          resolve: {
            alias: aliases
          }
        };
    "#;

    let summary =
        summarize_omena_bridge_bundler_path_aliases_for_config(config_path.as_path(), source);

    assert_eq!(summary.aliases, Vec::new());
    assert_eq!(summary.unrecognized.len(), 1);
    assert_eq!(summary.unrecognized[0].reason, "dynamic-alias-container");
    assert_eq!(summary.unrecognized[0].text, "aliases");
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn marks_dynamic_alias_entries_unrecognized() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("omena_bridge_vite_alias_dynamic")?;
    let config_path = root.join("vite.config.ts");
    let source = r#"
        export default {
          resolve: {
            alias: [{ find: /^@dynamic/, replacement: dynamicTarget }]
          }
        };
    "#;

    let summary =
        summarize_omena_bridge_bundler_path_aliases_for_config(config_path.as_path(), source);

    assert_eq!(summary.aliases, Vec::new());
    assert!(
        summary
            .unrecognized
            .iter()
            .any(|entry| entry.reason == "regex-alias-find")
    );
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn marks_dynamic_exported_config_unrecognized_without_top_level_fallback()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("omena_bridge_vite_dynamic_export")?;
    let config_path = root.join("vite.config.ts");
    let source = r#"
        const unrelated = {
          resolve: {
            alias: { "@wrong": "./src/wrong" }
          }
        };
        export default defineConfig(({ mode }) => ({
          resolve: {
            alias: { "@styles": "./src/styles" }
          }
        }));
    "#;

    let summary =
        summarize_omena_bridge_bundler_path_aliases_for_config(config_path.as_path(), source);

    assert_eq!(summary.aliases, Vec::new());
    assert_eq!(summary.unrecognized.len(), 1);
    assert_eq!(summary.unrecognized[0].reason, "dynamic-config-export");
    assert!(summary.unrecognized[0].text.contains("defineConfig"));
    assert!(!summary.unrecognized[0].text.contains("@wrong"));
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn marks_dynamic_module_exports_config_unrecognized() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("omena_bridge_webpack_dynamic_export")?;
    let config_path = root.join("webpack.config.cjs");
    let source = r#"
        module.exports = (env) => ({
          resolve: {
            alias: { "@theme": env.themePath }
          }
        });
    "#;

    let summary =
        summarize_omena_bridge_bundler_path_aliases_for_config(config_path.as_path(), source);

    assert_eq!(summary.aliases, Vec::new());
    assert_eq!(summary.unrecognized.len(), 1);
    assert_eq!(summary.unrecognized[0].reason, "dynamic-config-export");
    assert!(summary.unrecognized[0].text.contains("env"));
    let _ = fs::remove_dir_all(root);
    Ok(())
}

fn temp_dir(prefix: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let suffix = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}_{suffix}"));
    fs::create_dir_all(path.as_path())?;
    Ok(path)
}

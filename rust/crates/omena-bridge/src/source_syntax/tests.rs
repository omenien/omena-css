use super::*;

#[test]
fn builds_target_aware_source_syntax_index_for_css_modules_binding_inputs() {
    let source = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
const variants = { primary: "item--primary", icon: "item__icon" };
export function App({ tone }: { tone: "warm" | "cool" }) {
  return <div className={clsx("alert", cx("wrapper", variants.primary, `tone-${tone}`))} />;
}"#;

    let index = summarize_omena_bridge_source_syntax_index(
        source,
        vec![SourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }],
        vec!["bind".to_string()],
    );

    assert_eq!(index.product, "omena-bridge.source-syntax-index");
    assert!(index.class_string_literals.is_empty());
    assert!(index.selector_references.iter().any(|reference| {
        selector_reference_name(source, reference) == "wrapper"
            && reference.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
    }));
    assert!(index.selector_references.iter().any(|reference| {
        reference.selector_name.as_deref() == Some("item--primary")
            && reference.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
    }));
    assert!(index.selector_references.iter().any(|reference| {
        selector_reference_name(source, reference) == "alert"
            && reference.target_style_uri.as_deref().is_none()
    }));
    assert!(index.type_fact_targets.iter().any(|target| {
        &source[target.byte_span.start..target.byte_span.end] == "tone"
            && target.prefix == "tone-"
            && target.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
    }));
}

#[test]
fn collects_html_like_template_literal_class_attributes() {
    let source = r#"<main class="root active">
  <section class={dynamic}></section>
  <span class=""></span>
  <script type="module">
    const ignored = "class=\"from-script\"";
  </script>
</main>
"#;
    let index = summarize_omena_bridge_source_syntax_index_for_source_language(
        "Page.html",
        source,
        Some("html"),
        Vec::new(),
        Vec::new(),
    );

    let names = index
        .selector_references
        .iter()
        .map(|reference| selector_reference_name(source, reference))
        .collect::<Vec<_>>();

    assert_eq!(names, vec!["root", "active"]);
    assert!(!names.contains(&"{dynamic}"));
    assert!(!names.contains(&"from-script"));
}

#[test]
fn collects_server_template_literal_class_attributes() {
    let source = r#"{% if enabled %}
<main class="root active">
  <script>const ignored = "class=\"from-script\"";</script>
  <style>.ignored::before { content: "class=\"from-style\""; }</style>
</main>
{% endif %}
"#;
    let index = summarize_omena_bridge_source_syntax_index_for_source_language(
        "card.liquid",
        source,
        Some("liquid"),
        Vec::new(),
        Vec::new(),
    );

    let names = index
        .selector_references
        .iter()
        .map(|reference| selector_reference_name(source, reference))
        .collect::<Vec<_>>();

    assert_eq!(names, vec!["root", "active"]);
    assert!(!names.contains(&"from-script"));
    assert!(!names.contains(&"from-style"));
}

#[test]
fn masks_liquid_template_class_interpolations_without_shifting_literal_spans() -> Result<(), String>
{
    let source = r#"{% if enabled %}
<main class="card {{ modifier }} active"></main>
<section class="{{ modifier }}"></section>
<aside class="card--{{ m }} stable"></aside>
{% endif %}
"#;
    let index = summarize_omena_bridge_source_syntax_index_for_source_language(
        "card.liquid",
        source,
        Some("liquid"),
        Vec::new(),
        Vec::new(),
    );

    let names = index
        .selector_references
        .iter()
        .map(|reference| selector_reference_name(source, reference))
        .collect::<Vec<_>>();

    assert_eq!(names, vec!["card", "active", "stable"]);
    assert!(!names.contains(&"modifier"));
    assert!(!names.contains(&"card--"));
    for expected in ["card", "active", "stable"] {
        let reference = index
            .selector_references
            .iter()
            .find(|reference| selector_reference_name(source, reference) == expected)
            .ok_or_else(|| format!("literal selector reference is emitted for {expected}"))?;
        assert_eq!(
            &source[reference.byte_span.start..reference.byte_span.end],
            expected
        );
    }
    Ok(())
}

#[test]
fn masks_server_template_delimiter_families_in_literal_class_attributes() {
    let cases = [
        (
            "page.twig",
            Some("twig"),
            r#"<main class="card {{ modifier }} active item--{{ m }}"></main>"#,
            vec!["card", "active"],
        ),
        (
            "page.njk",
            Some("nunjucks"),
            r#"<main class="card {% if active %} active {% endif %} item--{{ m }}"></main>"#,
            vec!["card", "active"],
        ),
        (
            "page.django-html",
            Some("django-html"),
            r#"<main class="card {# ignored #} active item--{{ m }}"></main>"#,
            vec!["card", "active"],
        ),
        (
            "page.jinja",
            Some("jinja"),
            r#"<main class="card {{ modifier }} active item--{{ m }}"></main>"#,
            vec!["card", "active"],
        ),
        (
            "page.erb",
            Some("erb"),
            r#"<main class="card <%= modifier %> active item--<%= m %>"></main>"#,
            vec!["card", "active"],
        ),
        (
            "page.ejs",
            Some("ejs"),
            r#"<main class="card <%- modifier %> active item--<%= m %>"></main>"#,
            vec!["card", "active"],
        ),
        (
            "page.html.eex",
            Some("html-eex"),
            r#"<main class="card <%= modifier %> active item--<%= m %>"></main>"#,
            vec!["card", "active"],
        ),
        (
            "page.heex",
            Some("heex"),
            r#"<main class="card <%= modifier %> active item--<%= m %>"></main>"#,
            vec!["card", "active"],
        ),
        (
            "page.hbs",
            Some("handlebars"),
            r#"<main class="card {{{modifier}}} active item--{{m}}"></main>"#,
            vec!["card", "active"],
        ),
        (
            "page.njk",
            None,
            r#"<main class="card {{ modifier }} active item--{{ m }}"></main>"#,
            vec!["card", "active"],
        ),
    ];

    for (source_path, source_language, source, expected) in cases {
        let index = summarize_omena_bridge_source_syntax_index_for_source_language(
            source_path,
            source,
            source_language,
            Vec::new(),
            Vec::new(),
        );
        let names = index
            .selector_references
            .iter()
            .map(|reference| selector_reference_name(source, reference))
            .collect::<Vec<_>>();

        assert_eq!(names, expected, "{source_path}");
    }
}

#[test]
fn collects_template_style_binding_class_expressions() {
    let source = r#"<template>
  <section class={styles.root}></section>
  <section :class="styles['item--primary']"></section>
  <section v-bind:class="styles.icon"></section>
</template>
"#;
    let index = summarize_omena_bridge_source_syntax_index_for_source_language(
        "Card.vue",
        source,
        Some("vue"),
        vec![SourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri: "file:///workspace/Card.vue".to_string(),
        }],
        Vec::new(),
    );

    for expected in ["root", "item--primary", "icon"] {
        assert!(index.selector_references.iter().any(|reference| {
            selector_reference_name(source, reference) == expected
                && reference.target_style_uri.as_deref() == Some("file:///workspace/Card.vue")
        }));
    }
}

#[test]
fn collects_markdown_inline_html_classes_without_scanning_prose_or_code() {
    let source = r#"# Notes

The prose says class="from-prose" but it is not an HTML block.

<main class="root active">
  <section
    class="card"
  ></section>
</main>

    <span class="from-indented-code"></span>

```html
<span class="from-fence"></span>
```
"#;
    let index = summarize_omena_bridge_source_syntax_index_for_source_language(
        "Notes.md",
        source,
        Some("markdown"),
        Vec::new(),
        Vec::new(),
    );

    let names = index
        .selector_references
        .iter()
        .map(|reference| selector_reference_name(source, reference))
        .collect::<Vec<_>>();

    assert_eq!(names, vec!["root", "active", "card"]);
}

#[test]
fn collects_variant_recipe_universes_and_domain_references() -> Result<(), String> {
    let source = r#"import { cva } from "class-variance-authority";
const button = cva("btn", {
  variants: {
    intent: {
      primary: "btn-primary",
      secondary: ["btn-secondary"],
    },
  },
});
button({ intent: "primary" });
button({ intent: "ghost" });
"#;

    let index = summarize_omena_bridge_source_syntax_index(source, Vec::new(), Vec::new());

    let universe = index
        .class_value_universes
        .iter()
        .find(|universe| universe.owner_name == "button")
        .ok_or_else(|| "cva recipe should create a class value universe".to_string())?;
    assert_eq!(universe.plugin_id, "cva-recipe-domain");
    assert_eq!(universe.domain, "cva-recipe");
    assert!(universe.class_names.contains(&"btn".to_string()));
    assert!(universe.class_names.contains(&"btn-primary".to_string()));
    assert!(universe.class_names.contains(&"btn-secondary".to_string()));
    assert!(universe.axes.iter().any(|axis| {
        axis.axis_name == "intent"
            && axis.values == vec!["primary".to_string(), "secondary".to_string()]
    }));

    let referenced_options = index
        .domain_class_references
        .iter()
        .filter(|reference| reference.owner_name == "button" && reference.axis_name == "intent")
        .filter_map(|reference| reference.option_name.as_deref())
        .collect::<Vec<_>>();
    assert_eq!(referenced_options, vec!["primary", "ghost"]);
    Ok(())
}

#[test]
fn shadowed_local_does_not_resolve_variant_recipe_call() -> Result<(), String> {
    let source = r#"import { cva } from "class-variance-authority";
const button = cva("btn", {
  variants: {
    intent: {
      primary: "btn-primary",
    },
  },
});
export function View(button: (input: unknown) => string) {
  button({ intent: "primary" });
}"#;

    let index = summarize_omena_bridge_source_syntax_index(source, Vec::new(), Vec::new());

    assert!(
        index.domain_class_references.is_empty(),
        "shadowed button call must not resolve to the recipe binding"
    );
    assert!(
        index
            .class_value_universes
            .iter()
            .any(|universe| universe.owner_name == "button"),
        "the recipe declaration should still produce its universe"
    );
    Ok(())
}

#[test]
fn renamed_variant_recipe_import_still_resolves_calls_by_identity() -> Result<(), String> {
    let source = r#"import { cva as makeRecipe } from "class-variance-authority";
const renamedButton = makeRecipe("btn", {
  variants: {
    intent: {
      primary: "btn-primary",
    },
  },
});
renamedButton({ intent: "primary" });
"#;

    let index = summarize_omena_bridge_source_syntax_index(source, Vec::new(), Vec::new());

    let universe = index
        .class_value_universes
        .iter()
        .find(|universe| universe.owner_name == "renamedButton")
        .ok_or_else(|| "renamed recipe should create a class value universe".to_string())?;
    assert_eq!(universe.plugin_id, "cva-recipe-domain");
    assert!(index.domain_class_references.iter().any(|reference| {
        reference.owner_name == "renamedButton"
            && reference.axis_name == "intent"
            && reference.option_name.as_deref() == Some("primary")
    }));
    Ok(())
}

#[test]
fn collects_style_property_accesses_from_oxc_ast() {
    let source = r#"import styles from "./App.module.scss";
const text = "styles.fake";
export function View() {
  return <div className={styles.root} data-token={styles["item--primary"]} />;
}"#;

    let index = summarize_omena_bridge_source_syntax_index(
        source,
        vec![SourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }],
        Vec::new(),
    );

    let access_names = index
        .style_property_accesses
        .iter()
        .map(|access| &source[access.byte_span.start..access.byte_span.end])
        .collect::<Vec<_>>();

    assert_eq!(access_names, vec!["root", "item--primary"]);
    assert!(index.style_property_accesses.iter().all(|access| {
        access.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
    }));
    assert!(
        !index
            .selector_references
            .iter()
            .any(|reference| selector_reference_name(source, reference) == "fake")
    );
}

#[test]
fn symbol_resolver_distinguishes_import_binding_from_shadowing_parameter() -> Result<(), String> {
    let source = r#"import styles from "./App.module.scss";
function render(styles: Record<string, string>) {
  return styles.button;
}"#;
    let allocator = Allocator::default();
    let ParserReturn {
        program, panicked, ..
    } = Parser::new(
        &allocator,
        source,
        source_type_for_language("source.tsx", None),
    )
    .parse();
    if panicked {
        return Err("fixture parse panicked".to_string());
    }
    let semantic = SemanticBuilder::new().build(&program).semantic;
    let scoping = semantic.scoping();

    let import_symbol = program
        .body
        .iter()
        .find_map(|statement| {
            let Statement::ImportDeclaration(import) = statement else {
                return None;
            };
            import.specifiers.as_ref()?.iter().find_map(|specifier| {
                let ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) = specifier
                else {
                    return None;
                };
                (specifier.local.name.as_str() == "styles")
                    .then(|| binding_identifier_symbol_id(&specifier.local))
                    .flatten()
            })
        })
        .ok_or_else(|| "import styles symbol should exist".to_string())?;

    let (parameter_symbol, object_reference_symbol) = program
        .body
        .iter()
        .find_map(|statement| {
            let Statement::FunctionDeclaration(function) = statement else {
                return None;
            };
            let parameter = function.params.items.first()?;
            let parameter_symbol =
                binding_identifier_symbol_id(binding_pattern_identifier(&parameter.pattern)?)?;
            let body = function.body.as_ref()?;
            let return_statement = body.statements.iter().find_map(|statement| {
                let Statement::ReturnStatement(statement) = statement else {
                    return None;
                };
                statement.argument.as_ref()
            })?;
            let Expression::StaticMemberExpression(member) = return_statement else {
                return None;
            };
            let Expression::Identifier(identifier) = &member.object else {
                return None;
            };
            let object_reference_symbol = reference_symbol_id(scoping, identifier)?;
            Some((parameter_symbol, object_reference_symbol))
        })
        .ok_or_else(|| {
            "shadowing parameter and styles.button reference should exist".to_string()
        })?;

    assert_ne!(import_symbol, parameter_symbol);
    assert_eq!(object_reference_symbol, parameter_symbol);
    Ok(())
}

#[test]
fn shadowed_parameter_does_not_bind_import_styles_property_access() {
    let source = r#"import styles from "./App.module.scss";
export function View(styles: Record<string, string>) {
  return <div className={styles.root} />;
}"#;

    let index = summarize_omena_bridge_source_syntax_index(
        source,
        vec![SourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }],
        Vec::new(),
    );

    assert!(
        index.style_property_accesses.iter().all(|access| {
            &source[access.byte_span.start..access.byte_span.end] != "root"
                || access.target_style_uri.as_deref() != Some("file:///workspace/App.module.scss")
        }),
        "shadowed parameter styles.root must not bind to the import"
    );
}

#[test]
fn unresolved_style_reference_does_not_fall_back_to_import_name() {
    let source = r#"export function View() {
  return <div className={styles.root} />;
}"#;

    let index = summarize_omena_bridge_source_syntax_index(
        source,
        vec![SourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }],
        Vec::new(),
    );

    assert!(
        index.style_property_accesses.is_empty(),
        "unresolved styles reference must stay Unknown instead of using a name fallback"
    );
}

#[test]
fn renamed_style_import_still_binds_property_access_by_identity() {
    let source = r#"import moduleStyles from "./App.module.scss";
export function View() {
  return <div className={moduleStyles.root} />;
}"#;

    let index = summarize_omena_bridge_source_syntax_index(
        source,
        vec![SourceImportedStyleBindingV0 {
            binding: "moduleStyles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }],
        Vec::new(),
    );

    assert!(index.style_property_accesses.iter().any(|access| {
        &source[access.byte_span.start..access.byte_span.end] == "root"
            && access.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
    }));
}

#[test]
fn collects_inline_style_declarations_from_jsx_style_prop() {
    let source = r#"import styles from "./App.module.scss";
const token = "dynamic";
export function View() {
  return <div className={styles.root} style={{ color: "red", borderColor: `blue`, "--brand": token }} />;
}"#;

    let index = summarize_omena_bridge_source_syntax_index(
        source,
        vec![SourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }],
        Vec::new(),
    );

    let declarations = index
        .inline_style_declarations
        .iter()
        .map(|declaration| {
            (
                declaration.property_name.as_str(),
                declaration.value.as_deref(),
                declaration.cascade_tier,
                declaration.static_value,
                declaration.target_style_uri.as_deref(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        declarations,
        vec![
            (
                "color",
                Some("\"red\""),
                "authorInlineStyle",
                true,
                Some("file:///workspace/App.module.scss")
            ),
            (
                "border-color",
                Some("`blue`"),
                "authorInlineStyle",
                true,
                Some("file:///workspace/App.module.scss")
            ),
            (
                "--brand",
                None,
                "authorInlineStyle",
                false,
                Some("file:///workspace/App.module.scss")
            ),
        ]
    );
}

#[test]
fn walks_anonymous_arrow_default_export_body_for_style_property_accesses() {
    // Regression for RFC-0007 #53: `collect_export_default_declaration` dropped every
    // expression-kind default export except member/call, so `export default () => <JSX/>` was
    // never walked and its `styles.foo` references were lost -> unusedSelector false positives.
    // Cover the three failing forms; `.ghost` is never referenced and must stay absent so the
    // fix does not silence true positives.
    for body in [
        "() => <i className={styles.used} />",
        "() => { return <i className={styles.used} />; }",
        "() => (<i className={styles.used} />)",
    ] {
        let source = format!("import styles from \"./App.module.scss\";\nexport default {body};");

        let index = summarize_omena_bridge_source_syntax_index(
            &source,
            vec![SourceImportedStyleBindingV0 {
                binding: "styles".to_string(),
                style_uri: "file:///workspace/App.module.scss".to_string(),
            }],
            Vec::new(),
        );

        assert!(
            index
                .style_property_accesses
                .iter()
                .any(|access| &source[access.byte_span.start..access.byte_span.end] == "used"),
            "anon-arrow default export should collect styles.used: {body}",
        );
        assert!(
            !index
                .selector_references
                .iter()
                .any(|reference| selector_reference_name(&source, reference) == "ghost"),
            "no phantom references should appear: {body}",
        );
    }
}

#[test]
fn collects_classnames_bind_utility_bindings_from_oxc_ast() {
    let source = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind((styles));
export const view = <div className={cx("root")} />;"#;

    let index = summarize_omena_bridge_source_syntax_index(
        source,
        vec![SourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }],
        vec!["bind".to_string()],
    );

    assert!(index.selector_references.iter().any(|reference| {
        selector_reference_name(source, reference) == "root"
            && reference.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
    }));
}

#[test]
fn shadowed_local_does_not_bind_classnames_bind_import() {
    let source = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
export function View(cx: (...args: string[]) => string) {
  return <div className={cx("root")} />;
}"#;

    let index = summarize_omena_bridge_source_syntax_index(
        source,
        vec![SourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }],
        vec!["bind".to_string()],
    );

    assert!(
        index.selector_references.iter().all(|reference| {
            selector_reference_name(source, reference) != "root"
                || reference.target_style_uri.as_deref()
                    != Some("file:///workspace/App.module.scss")
        }),
        "shadowed cx call must not bind to the classnames/bind utility"
    );
}

#[test]
fn renamed_classnames_bind_import_and_style_import_still_bind_by_identity() {
    let source = r#"import renamedBind from "classnames/bind";
import moduleStyles from "./App.module.scss";
const cx = renamedBind.bind(moduleStyles);
export const view = <div className={cx("root")} />;"#;

    let index = summarize_omena_bridge_source_syntax_index(
        source,
        vec![SourceImportedStyleBindingV0 {
            binding: "moduleStyles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }],
        vec!["renamedBind".to_string()],
    );

    assert!(index.selector_references.iter().any(|reference| {
        selector_reference_name(source, reference) == "root"
            && reference.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
    }));
}

#[test]
fn summarizes_classnames_bind_utility_binding_identity_for_binding_index() {
    let source = r#"import renamedBind from "classnames/bind";
import moduleStyles from "./App.module.scss";
const cx = renamedBind.bind(moduleStyles);
const localClass = "root";
export const view = <div className={cx(localClass, moduleStyles.icon)} />;"#;

    let index = summarize_omena_bridge_source_binding_index(
        source,
        vec![SourceImportedStyleBindingV0 {
            binding: "moduleStyles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }],
        vec!["renamedBind".to_string()],
    );

    assert_eq!(index.product, "omena-bridge.source-binding-index");
    let renamed_bind_decl_start = source.find("renamedBind").unwrap_or(usize::MAX);
    assert_ne!(renamed_bind_decl_start, usize::MAX);
    let module_styles_decl_start = source.find("moduleStyles").unwrap_or(usize::MAX);
    assert_ne!(module_styles_decl_start, usize::MAX);
    let cx_decl_start = source.find("cx =").unwrap_or(usize::MAX);
    assert_ne!(cx_decl_start, usize::MAX);
    let local_decl_start = source.find("localClass =").unwrap_or(usize::MAX);
    assert_ne!(local_decl_start, usize::MAX);
    let view_decl_start = source.find("view =").unwrap_or(usize::MAX);
    assert_ne!(view_decl_start, usize::MAX);
    let source_scope = ParserByteSpanV0 {
        start: 0,
        end: source.len(),
    };
    assert_eq!(
        index.binding_scopes,
        vec![SourceBindingScopeFactV0 {
            kind: "sourceFile",
            byte_span: source_scope,
        }]
    );
    assert!(index.scope_parent_edges.is_empty());
    assert_eq!(
        index.binding_decls,
        vec![
            SourceBindingDeclFactV0 {
                kind: "import",
                name: "moduleStyles".to_string(),
                byte_span: ParserByteSpanV0 {
                    start: module_styles_decl_start,
                    end: module_styles_decl_start + "moduleStyles".len(),
                },
                import_path: Some("./App.module.scss".to_string()),
            },
            SourceBindingDeclFactV0 {
                kind: "import",
                name: "renamedBind".to_string(),
                byte_span: ParserByteSpanV0 {
                    start: renamed_bind_decl_start,
                    end: renamed_bind_decl_start + "renamedBind".len(),
                },
                import_path: Some("classnames/bind".to_string()),
            },
            SourceBindingDeclFactV0 {
                kind: "localVar",
                name: "cx".to_string(),
                byte_span: ParserByteSpanV0 {
                    start: cx_decl_start,
                    end: cx_decl_start + "cx".len(),
                },
                import_path: None,
            },
            SourceBindingDeclFactV0 {
                kind: "localVar",
                name: "localClass".to_string(),
                byte_span: ParserByteSpanV0 {
                    start: local_decl_start,
                    end: local_decl_start + "localClass".len(),
                },
                import_path: None,
            },
            SourceBindingDeclFactV0 {
                kind: "localVar",
                name: "view".to_string(),
                byte_span: ParserByteSpanV0 {
                    start: view_decl_start,
                    end: view_decl_start + "view".len(),
                },
                import_path: None,
            },
        ]
    );
    assert_eq!(
        index.scope_contains_decls,
        vec![
            SourceScopeContainsDeclFactV0 {
                scope_kind: "sourceFile",
                scope_byte_span: source_scope,
                decl_kind: "import",
                decl_name: "moduleStyles".to_string(),
                decl_byte_span: ParserByteSpanV0 {
                    start: module_styles_decl_start,
                    end: module_styles_decl_start + "moduleStyles".len(),
                },
                import_path: Some("./App.module.scss".to_string()),
            },
            SourceScopeContainsDeclFactV0 {
                scope_kind: "sourceFile",
                scope_byte_span: source_scope,
                decl_kind: "import",
                decl_name: "renamedBind".to_string(),
                decl_byte_span: ParserByteSpanV0 {
                    start: renamed_bind_decl_start,
                    end: renamed_bind_decl_start + "renamedBind".len(),
                },
                import_path: Some("classnames/bind".to_string()),
            },
            SourceScopeContainsDeclFactV0 {
                scope_kind: "sourceFile",
                scope_byte_span: source_scope,
                decl_kind: "localVar",
                decl_name: "cx".to_string(),
                decl_byte_span: ParserByteSpanV0 {
                    start: cx_decl_start,
                    end: cx_decl_start + "cx".len(),
                },
                import_path: None,
            },
            SourceScopeContainsDeclFactV0 {
                scope_kind: "sourceFile",
                scope_byte_span: source_scope,
                decl_kind: "localVar",
                decl_name: "localClass".to_string(),
                decl_byte_span: ParserByteSpanV0 {
                    start: local_decl_start,
                    end: local_decl_start + "localClass".len(),
                },
                import_path: None,
            },
            SourceScopeContainsDeclFactV0 {
                scope_kind: "sourceFile",
                scope_byte_span: source_scope,
                decl_kind: "localVar",
                decl_name: "view".to_string(),
                decl_byte_span: ParserByteSpanV0 {
                    start: view_decl_start,
                    end: view_decl_start + "view".len(),
                },
                import_path: None,
            },
        ]
    );
    assert_eq!(
        index.style_import_bindings,
        vec![SourceBindingStyleImportFactV0 {
            local_name: "moduleStyles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }]
    );
    assert_eq!(
        index.declares_style_imports,
        vec![SourceDeclaresStyleImportFactV0 {
            decl_name: "moduleStyles".to_string(),
            styles_local_name: "moduleStyles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }]
    );
    assert_eq!(
        index.style_import_resolves_modules,
        vec![SourceStyleImportResolvesModuleFactV0 {
            styles_local_name: "moduleStyles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }]
    );
    let local_start = source.rfind("localClass").unwrap_or(usize::MAX);
    assert_ne!(local_start, usize::MAX);
    let icon_start = source.find("icon").unwrap_or(usize::MAX);
    assert_ne!(icon_start, usize::MAX);
    assert_eq!(
        index.expression_targets_modules,
        vec![
            SourceExpressionTargetsModuleFactV0 {
                byte_span: ParserByteSpanV0 {
                    start: local_start,
                    end: local_start + "localClass".len(),
                },
                target_style_uri: "file:///workspace/App.module.scss".to_string(),
            },
            SourceExpressionTargetsModuleFactV0 {
                byte_span: ParserByteSpanV0 {
                    start: icon_start,
                    end: icon_start + "icon".len(),
                },
                target_style_uri: "file:///workspace/App.module.scss".to_string(),
            },
        ]
    );
    assert_eq!(
        index.class_expression_nodes,
        vec![
            SourceClassExpressionNodeFactV0 {
                kind: "symbolRef",
                byte_span: ParserByteSpanV0 {
                    start: local_start,
                    end: local_start + "localClass".len(),
                },
                target_style_uri: "file:///workspace/App.module.scss".to_string(),
            },
            SourceClassExpressionNodeFactV0 {
                kind: "styleAccess",
                byte_span: ParserByteSpanV0 {
                    start: icon_start,
                    end: icon_start + "icon".len(),
                },
                target_style_uri: "file:///workspace/App.module.scss".to_string(),
            },
        ]
    );
    assert_eq!(
        index.classnames_bind_utility_bindings,
        vec![SourceClassnamesBindUtilityBindingFactV0 {
            local_name: "cx".to_string(),
            styles_local_name: "moduleStyles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
            classnames_import_name: "renamedBind".to_string(),
        }]
    );
    assert_eq!(
        index.declares_utility_bindings,
        vec![SourceDeclaresUtilityBindingFactV0 {
            decl_name: "cx".to_string(),
            utility_local_name: "cx".to_string(),
            utility_kind: "classnamesBind",
        }]
    );
    assert_eq!(
        index.utility_uses_style_imports,
        vec![SourceUtilityUsesStyleImportFactV0 {
            utility_local_name: "cx".to_string(),
            styles_local_name: "moduleStyles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }]
    );
    assert_eq!(
        index.style_access_uses_style_imports,
        vec![SourceStyleAccessUsesStyleImportFactV0 {
            byte_span: ParserByteSpanV0 {
                start: icon_start,
                end: icon_start + "icon".len(),
            },
            decl_name: "moduleStyles".to_string(),
            styles_local_name: "moduleStyles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }]
    );
    assert_eq!(
        index.symbol_ref_uses_decls,
        vec![SourceSymbolRefUsesDeclFactV0 {
            byte_span: ParserByteSpanV0 {
                start: local_start,
                end: local_start + "localClass".len(),
            },
            raw_reference: "localClass".to_string(),
            root_name: "localClass".to_string(),
            decl_name: "localClass".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }]
    );
}

#[test]
fn does_not_treat_object_shorthand_aliases_as_static_class_values() {
    let source = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
export function View({ primary }: { primary: "medium" | "small" }) {
  const variants = { primary };
  return <div className={cx(variants.primary)} />;
}"#;

    let index = summarize_omena_bridge_source_syntax_index(
        source,
        vec![SourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }],
        vec!["bind".to_string()],
    );

    assert!(!index.selector_references.iter().any(|reference| {
        selector_reference_name(source, reference) == "primary"
            && reference.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
    }));
    assert!(index.type_fact_targets.iter().any(|target| {
        &source[target.byte_span.start..target.byte_span.end] == "variants.primary"
            && target.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
    }));
}

#[test]
fn merges_class_value_reassignments_into_symbol_selector_references() {
    let source = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
export function View({ enabled }: { enabled: boolean }) {
  let size = "card";
  if (enabled) {
    size = "card--active";
  }
  return <div className={cx(size)} />;
}"#;

    let index = summarize_omena_bridge_source_syntax_index(
        source,
        vec![SourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }],
        vec!["bind".to_string()],
    );

    let size_references = index
        .selector_references
        .iter()
        .filter(|reference| &source[reference.byte_span.start..reference.byte_span.end] == "size")
        .collect::<Vec<_>>();
    assert!(size_references.iter().any(|reference| {
        reference.selector_name.as_deref() == Some("card")
            && reference.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
    }));
    assert!(size_references.iter().any(|reference| {
        reference.selector_name.as_deref() == Some("card--active")
            && reference.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
    }));
}

#[test]
fn keeps_template_prefix_selector_references_as_atomic_flat_class_prefixes() {
    let source = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
export function View({ fontSize }: { fontSize: 10 | 12 }) {
  return <div className={cx(`font-size-${fontSize}`)} />;
}"#;

    let index = summarize_omena_bridge_source_syntax_index(
        source,
        vec![SourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }],
        vec!["bind".to_string()],
    );
    let reference_names = index
        .selector_references
        .iter()
        .filter(|reference| {
            reference.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
        })
        .map(|reference| selector_reference_name(source, reference))
        .collect::<Vec<_>>();

    assert!(reference_names.contains(&"font-size-"));
    assert!(!reference_names.contains(&"font"));
    assert!(!reference_names.contains(&"-size"));
    assert!(index.type_fact_targets.iter().any(|target| {
        &source[target.byte_span.start..target.byte_span.end] == "fontSize"
            && target.prefix == "font-size-"
            && target.suffix.is_empty()
            && target.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
    }));
}

#[test]
fn walks_expression_like_oxc_argument_array_and_property_key_variants() {
    let source = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
const items = [1];
const nodes = [<span className={cx("arrayItem")} />];
const keyed = { [items.length ? <span className={cx("keyedItem")} /> : "fallback"]: true };
export const view = <>{items.map(() => <a className={cx("callbackLink")} />)}{nodes}</>;"#;

    let index = summarize_omena_bridge_source_syntax_index(
        source,
        vec![SourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri: "file:///workspace/App.module.scss".to_string(),
        }],
        vec!["bind".to_string()],
    );
    let names = index
        .selector_references
        .iter()
        .filter(|reference| {
            reference.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
        })
        .map(|reference| selector_reference_name(source, reference))
        .collect::<Vec<_>>();

    assert!(
        names.contains(&"arrayItem"),
        "array literal JSX should be walked"
    );
    assert!(
        names.contains(&"keyedItem"),
        "computed property key expression should be walked"
    );
    assert!(
        names.contains(&"callbackLink"),
        "callback argument JSX should be walked"
    );
}

#[test]
fn collects_class_name_string_literals_from_oxc_ast() {
    let source = r#"const text = "className=\"fake\"";
export const view = <div className="root item--primary" data-token="ignored" />;"#;

    let index = summarize_omena_bridge_source_syntax_index(source, Vec::new(), Vec::new());

    let literal_values = index
        .class_string_literals
        .iter()
        .map(|span| &source[span.start..span.end])
        .collect::<Vec<_>>();

    assert_eq!(literal_values, vec!["root item--primary"]);
    assert!(
        index
            .selector_references
            .iter()
            .any(|reference| { selector_reference_name(source, reference) == "root" })
    );
    assert!(
        index
            .selector_references
            .iter()
            .any(|reference| { selector_reference_name(source, reference) == "item--primary" })
    );
}

#[test]
fn source_recovery_scanners_keep_multibyte_escape_boundaries()
-> Result<(), Box<dyn std::error::Error>> {
    let source = r#"const escaped = "\비";
const view = <div className={cx("root", active && `상태-${tone}`)} />;"#;
    let escaped_quote = source
        .find(r#""\비""#)
        .ok_or_else(|| std::io::Error::other("escaped fixture exists"))?;
    let escaped_end = skip_js_string_literal(source, escaped_quote, source.len())
        .ok_or_else(|| std::io::Error::other("escaped string should be skipped"))?;
    assert!(source.is_char_boundary(escaped_end));

    let expression_start = source
        .find("cx(")
        .ok_or_else(|| std::io::Error::other("cx call exists"))?
        + "cx(".len();
    let expression_end = js_call_end(source, expression_start - 1)
        .ok_or_else(|| std::io::Error::other("cx call ends"))?;
    let segments = split_top_level_js_segments(source, expression_start, expression_end, b',');
    assert_eq!(segments.len(), 2);
    for (start, end) in segments {
        assert!(source.is_char_boundary(start));
        assert!(source.is_char_boundary(end));
    }

    let operator = find_top_level_js_operator(source, expression_start, expression_end, "&&")
        .ok_or_else(|| {
            std::io::Error::other(
                "conditional operator should be found without slicing inside UTF-8",
            )
        })?;
    assert!(source.is_char_boundary(operator));
    Ok(())
}

#[test]
fn collects_vue_sfc_use_css_module_bindings_from_projected_script() {
    let source = r#"<template><div :class="$style.ignored" /></template>
<script setup lang="ts">
import { useCssModule as useModule } from "vue";
const styles = useModule();
const text = ".not-style";
</script>
<style module>
.root {}
</style>
"#;

    let bindings = collect_omena_bridge_vue_style_module_bindings("Card.vue", source, Some("vue"));

    assert_eq!(bindings, vec!["styles"]);
}

#[test]
fn indexes_vue_sfc_use_css_module_property_accesses_against_vue_style_uri() {
    let source = r#"<template><div /></template>
<script setup lang="ts">
import { useCssModule } from "vue";
const styles = useCssModule();
export const root = styles.root;
</script>
<style module>
.root { color: red; }
</style>
"#;
    let index = summarize_omena_bridge_source_syntax_index_for_source_language(
        "file:///workspace/Card.vue",
        source,
        Some("vue"),
        vec![SourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri: "file:///workspace/Card.vue".to_string(),
        }],
        Vec::new(),
    );

    assert!(index.selector_references.iter().any(|reference| {
        selector_reference_name(source, reference) == "root"
            && reference.target_style_uri.as_deref() == Some("file:///workspace/Card.vue")
    }));
}

#[test]
fn indexes_html_script_property_accesses_against_imported_style_uri() {
    let source = r#"<main>ignored</main>
<script type="module">
import styles from "./Page.module.scss";
export const root = styles.root;
</script>
"#;
    let index = summarize_omena_bridge_source_syntax_index_for_source_language(
        "file:///workspace/Page.html",
        source,
        Some("html"),
        vec![SourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri: "file:///workspace/Page.module.scss".to_string(),
        }],
        Vec::new(),
    );

    assert!(index.selector_references.iter().any(|reference| {
        selector_reference_name(source, reference) == "root"
            && reference.target_style_uri.as_deref() == Some("file:///workspace/Page.module.scss")
    }));
    assert!(
        !index
            .selector_references
            .iter()
            .any(|reference| selector_reference_name(source, reference) == "ignored")
    );
}

fn selector_reference_name<'a>(
    source: &'a str,
    reference: &'a SourceSelectorReferenceFactV0,
) -> &'a str {
    reference
        .selector_name
        .as_deref()
        .unwrap_or(&source[reference.byte_span.start..reference.byte_span.end])
}

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

fn selector_reference_name<'a>(
    source: &'a str,
    reference: &'a SourceSelectorReferenceFactV0,
) -> &'a str {
    reference
        .selector_name
        .as_deref()
        .unwrap_or(&source[reference.byte_span.start..reference.byte_span.end])
}

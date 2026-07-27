#![no_main]

use std::collections::BTreeSet;

use libfuzzer_sys::fuzz_target;
use omena_cascade::{
    ElementSignature, SelectorMatchVerdict, selector_co_match_verdict, selector_match_witness,
};

fuzz_target!(|data: &[u8]| {
    let Some((selector, element)) = decode_case(data) else {
        return;
    };
    let ceiling = selector_co_match_verdict(selector, selector);
    let direct = selector_match_witness(selector, &element).verdict;

    if ceiling == SelectorMatchVerdict::Maybe {
        assert_eq!(
            direct,
            SelectorMatchVerdict::Maybe,
            "direct matcher exceeded the co-match precision ceiling for {selector:?}"
        );
    }
});

fn decode_case(data: &[u8]) -> Option<(&str, ElementSignature)> {
    let text = std::str::from_utf8(data).ok()?;
    let mut fields = text.trim().split('|');
    let selector = fields.next()?.trim();
    if selector.is_empty() {
        return None;
    }

    let tag = optional_field(fields.next());
    let id = optional_field(fields.next());
    let classes = string_set(fields.next());
    let attributes = string_set(fields.next());
    let pseudo_states = string_set(fields.next());
    let exactness = fields
        .next()
        .and_then(|value| value.trim().parse::<u8>().ok())
        .unwrap_or(0);

    Some((
        selector,
        ElementSignature {
            tag,
            id,
            classes,
            attributes,
            pseudo_states,
            classes_are_exact: exactness & 1 != 0,
            attributes_are_exact: exactness & 2 != 0,
            pseudo_states_are_exact: exactness & 4 != 0,
            tag_is_exact: exactness & 8 != 0,
            id_is_exact: exactness & 16 != 0,
        },
    ))
}

fn optional_field(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn string_set(value: Option<&str>) -> BTreeSet<String> {
    value
        .into_iter()
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

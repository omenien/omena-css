use std::collections::BTreeMap;

use super::{
    less_mixin_arguments::static_less_mixin_pattern_argument_matches,
    model::{
        StaticScssFunctionArgument, StaticScssFunctionCall, StaticScssFunctionDeclaration,
        StaticScssFunctionParameter, StaticScssMixinDeclaration, StaticScssMixinIncludeCall,
    },
};

pub(super) fn bind_static_scss_function_arguments(
    declaration: &StaticScssFunctionDeclaration,
    call: &StaticScssFunctionCall,
) -> Option<Vec<(String, String)>> {
    bind_static_scss_callable_arguments(&declaration.parameters, &call.arguments)
}

pub(super) fn bind_static_scss_mixin_arguments(
    declaration: &StaticScssMixinDeclaration,
    call: &StaticScssMixinIncludeCall,
) -> Option<Vec<(String, String)>> {
    bind_static_scss_callable_arguments(&declaration.parameters, &call.arguments)
}

pub(super) fn bind_static_scss_callable_arguments(
    parameters: &[StaticScssFunctionParameter],
    arguments: &[StaticScssFunctionArgument],
) -> Option<Vec<(String, String)>> {
    let mut bindings = BTreeMap::<String, String>::new();
    let mut positional_index = 0usize;
    let mut saw_named_argument = false;

    for argument in arguments {
        if let Some(argument_name) = argument.name.as_ref() {
            saw_named_argument = true;
            if !parameters.iter().any(|parameter| {
                parameter.pattern_value.is_none() && parameter.name == *argument_name
            }) || bindings
                .insert(argument_name.clone(), argument.value.clone())
                .is_some()
            {
                return None;
            }
            continue;
        }

        if saw_named_argument {
            return None;
        }
        let parameter = parameters.get(positional_index)?;
        if let Some(pattern_value) = parameter.pattern_value.as_deref() {
            if !static_less_mixin_pattern_argument_matches(pattern_value, argument.value.as_str()) {
                return None;
            }
            positional_index += 1;
            continue;
        }
        if parameter.variadic {
            bindings
                .entry(parameter.name.clone())
                .and_modify(|value| {
                    value.push_str(", ");
                    value.push_str(argument.value.as_str());
                })
                .or_insert_with(|| argument.value.clone());
            continue;
        }
        if bindings
            .insert(parameter.name.clone(), argument.value.clone())
            .is_some()
        {
            return None;
        }
        positional_index += 1;
    }

    for (index, parameter) in parameters.iter().enumerate() {
        if parameter.pattern_value.is_some() {
            if index >= positional_index {
                return None;
            }
            continue;
        }
        if bindings.contains_key(parameter.name.as_str()) {
            continue;
        }
        if parameter.variadic {
            return None;
        }
        let default_value = parameter.default_value.as_ref()?;
        bindings.insert(parameter.name.clone(), default_value.clone());
    }

    parameters
        .iter()
        .filter(|parameter| parameter.pattern_value.is_none())
        .map(|parameter| {
            bindings
                .remove(parameter.name.as_str())
                .map(|value| (parameter.name.clone(), value))
        })
        .collect::<Option<Vec<_>>>()
}

use super::{
    OmenaCheckerCodeBundleNameV0, OmenaCheckerCodeBundleV0, OmenaCheckerFindingCategoryV0,
    OmenaCheckerRuleCodeV0, OmenaCheckerRuleDescriptorV0, OmenaCheckerRuleFixabilityV0,
    OmenaCheckerRulePresetV0, OmenaCheckerRuleTierV0, OmenaCheckerSeverityV0,
};

pub(crate) fn rule(
    code: OmenaCheckerRuleCodeV0,
    category: OmenaCheckerFindingCategoryV0,
    default_severity: OmenaCheckerSeverityV0,
    fixability: OmenaCheckerRuleFixabilityV0,
    presets: &[OmenaCheckerRulePresetV0],
    description: &'static str,
) -> OmenaCheckerRuleDescriptorV0 {
    let tier = rule_tier_for_code(code);
    OmenaCheckerRuleDescriptorV0 {
        code,
        code_name: code.as_str(),
        ordinal: rule_ordinal_for_code(code),
        category,
        category_name: category.as_str(),
        tier,
        tier_name: tier.as_str(),
        default_severity,
        default_severity_name: default_severity.as_str(),
        fixability,
        fixability_name: fixability.as_str(),
        presets: presets.to_vec(),
        preset_names: presets.iter().map(|preset| preset.as_str()).collect(),
        description,
    }
}

fn rule_ordinal_for_code(code: OmenaCheckerRuleCodeV0) -> u16 {
    use OmenaCheckerRuleCodeV0::{
        CascadeDeepConflict, CascadeSMTViolation, CascadeUnreachableRule,
        CategoricalCascadeEvidenceInconsistency, CircularVar, DeadCascadeLayer,
        DesignSystemMdlBudget, DesignerIntentInconsistency, IacvtProne, InvalidPropertyValue,
        MissingComposedModule,
        MissingComposedSelector, MissingCustomProperty, MissingImportedValue, MissingKeyframes,
        MissingModule, MissingResolvedClassDomain, MissingResolvedClassValues, MissingSassSymbol,
        MissingStaticClass, MissingTemplatePrefix, MissingValueModule, NoImpossibleSelector,
        NoImpreciseValue, NoUnknownDynamicClass, RegisteredPropertyTypeMismatch,
        ReplicaEnsembleInconsistency, RgFlowRelevantOperator, StreamingIfdsPrecisionParity,
        UnreachableDeclaration, UnspecifiedCascadeTie, UnusedSelector,
    };

    match code {
        NoUnknownDynamicClass => 1,
        NoImpreciseValue => 2,
        NoImpossibleSelector => 3,
        MissingModule => 4,
        MissingStaticClass => 5,
        MissingTemplatePrefix => 6,
        MissingResolvedClassValues => 7,
        MissingResolvedClassDomain => 8,
        UnusedSelector => 9,
        MissingComposedModule => 10,
        MissingComposedSelector => 11,
        MissingValueModule => 12,
        MissingImportedValue => 13,
        MissingKeyframes => 14,
        MissingCustomProperty => 15,
        MissingSassSymbol => 16,
        UnreachableDeclaration => 17,
        DeadCascadeLayer => 18,
        IacvtProne => 19,
        CircularVar => 20,
        UnspecifiedCascadeTie => 21,
        DesignerIntentInconsistency => 22,
        CascadeSMTViolation => 23,
        DesignSystemMdlBudget => 24,
        StreamingIfdsPrecisionParity => 25,
        CascadeDeepConflict => 26,
        CascadeUnreachableRule => 27,
        RgFlowRelevantOperator => 28,
        ReplicaEnsembleInconsistency => 29,
        CategoricalCascadeEvidenceInconsistency => 30,
        RegisteredPropertyTypeMismatch => 31,
        InvalidPropertyValue => 32,
    }
}

pub(crate) fn rule_tier_for_code(code: OmenaCheckerRuleCodeV0) -> OmenaCheckerRuleTierV0 {
    use OmenaCheckerRuleCodeV0::{
        CascadeDeepConflict, CascadeSMTViolation, CascadeUnreachableRule,
        CategoricalCascadeEvidenceInconsistency, CircularVar, DeadCascadeLayer,
        DesignSystemMdlBudget, DesignerIntentInconsistency, IacvtProne, InvalidPropertyValue,
        MissingComposedModule,
        MissingComposedSelector, MissingCustomProperty, MissingImportedValue, MissingKeyframes,
        MissingModule, MissingResolvedClassDomain, MissingResolvedClassValues, MissingSassSymbol,
        MissingStaticClass, MissingTemplatePrefix, MissingValueModule, NoImpossibleSelector,
        NoImpreciseValue, NoUnknownDynamicClass, RegisteredPropertyTypeMismatch,
        ReplicaEnsembleInconsistency, RgFlowRelevantOperator, StreamingIfdsPrecisionParity,
        UnreachableDeclaration, UnspecifiedCascadeTie, UnusedSelector,
    };

    match code {
        NoUnknownDynamicClass | NoImpreciseValue | NoImpossibleSelector => {
            OmenaCheckerRuleTierV0::M
        }
        MissingModule
        | MissingStaticClass
        | MissingTemplatePrefix
        | MissingResolvedClassValues
        | MissingResolvedClassDomain
        | CascadeDeepConflict
        | CascadeSMTViolation => OmenaCheckerRuleTierV0::S,
        UnusedSelector
        | MissingComposedModule
        | MissingComposedSelector
        | MissingValueModule
        | MissingImportedValue
        | MissingKeyframes
        | MissingCustomProperty
        | MissingSassSymbol
        | UnreachableDeclaration
        | DeadCascadeLayer
        | IacvtProne
        | CircularVar
        | RegisteredPropertyTypeMismatch
        | InvalidPropertyValue
        | UnspecifiedCascadeTie => OmenaCheckerRuleTierV0::T,
        DesignSystemMdlBudget
        | CascadeUnreachableRule
        | DesignerIntentInconsistency
        | StreamingIfdsPrecisionParity
        | RgFlowRelevantOperator
        | ReplicaEnsembleInconsistency
        | CategoricalCascadeEvidenceInconsistency => OmenaCheckerRuleTierV0::I,
    }
}

pub(crate) fn count_rules_in_tier(
    descriptors: &[OmenaCheckerRuleDescriptorV0],
    tier: OmenaCheckerRuleTierV0,
) -> usize {
    descriptors
        .iter()
        .filter(|descriptor| descriptor.tier == tier)
        .count()
}

pub(crate) fn bundle(
    bundle: OmenaCheckerCodeBundleNameV0,
    codes: &[OmenaCheckerRuleCodeV0],
) -> OmenaCheckerCodeBundleV0 {
    OmenaCheckerCodeBundleV0 {
        bundle,
        bundle_name: bundle.as_str(),
        codes: codes.to_vec(),
        code_names: codes.iter().map(|code| code.as_str()).collect(),
    }
}

import type { DomainClassReferenceHIR, SourceDocumentHIR } from "../hir/source-types";

export interface DomainClassReferenceGroup {
  readonly pluginId: string;
  readonly domain: string;
  readonly references: readonly DomainClassReferenceHIR[];
  readonly literalCount: number;
  readonly templatePrefixCount: number;
}

export interface DomainClassReferenceSummary {
  readonly groups: readonly DomainClassReferenceGroup[];
  readonly totalReferences: number;
  readonly hasUtilityDomainReferences: boolean;
}

export function readDomainClassReferenceSummary(
  sourceDocument: SourceDocumentHIR,
): DomainClassReferenceSummary {
  const groupsByKey = new Map<string, DomainClassReferenceHIR[]>();

  for (const reference of sourceDocument.domainClassReferences) {
    const key = groupKey(reference.pluginId, reference.domain);
    const group = groupsByKey.get(key);
    if (group) {
      group.push(reference);
    } else {
      groupsByKey.set(key, [reference]);
    }
  }

  const groups = Array.from(groupsByKey.entries())
    .map(([key, references]) => {
      const [pluginId, domain] = key.split("\0", 2);
      return {
        pluginId: pluginId!,
        domain: domain!,
        references,
        literalCount: references.filter((reference) => reference.matchKind === "literal").length,
        templatePrefixCount: references.filter(
          (reference) => reference.matchKind === "templatePrefix",
        ).length,
      };
    })
    .toSorted((a, b) => `${a.pluginId}:${a.domain}`.localeCompare(`${b.pluginId}:${b.domain}`));

  return {
    groups,
    totalReferences: sourceDocument.domainClassReferences.length,
    hasUtilityDomainReferences: sourceDocument.domainClassReferences.length > 0,
  };
}

function groupKey(pluginId: string, domain: string): string {
  return `${pluginId}\0${domain}`;
}

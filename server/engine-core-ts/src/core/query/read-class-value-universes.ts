import type { ClassValueUniverseEntryV0 } from "../binder/class-value-universe-provider";
import { classNamesForUniverse } from "../abstract-value/class-value-universe";

export interface ClassValueUniverseGroup {
  readonly pluginId: string;
  readonly domain: string;
  readonly entries: readonly ClassValueUniverseEntryV0[];
  readonly finiteCount: number;
  readonly reducedProductCount: number;
  readonly noneCount: number;
  readonly classNames: readonly string[];
}

export interface ClassValueUniverseSummary {
  readonly groups: readonly ClassValueUniverseGroup[];
  readonly totalUniverses: number;
  readonly hasReducedProductUniverse: boolean;
  readonly classNames: readonly string[];
}

export function readClassValueUniverseSummary(
  entries: readonly ClassValueUniverseEntryV0[],
): ClassValueUniverseSummary {
  const groupsByKey = new Map<string, ClassValueUniverseEntryV0[]>();
  for (const entry of entries) {
    const key = groupKey(entry.pluginId, entry.domain);
    const group = groupsByKey.get(key);
    if (group) {
      group.push(entry);
    } else {
      groupsByKey.set(key, [entry]);
    }
  }

  const groups = Array.from(groupsByKey.entries())
    .map(([key, groupEntries]) => {
      const [pluginId, domain] = key.split("\0", 2);
      return {
        pluginId: pluginId!,
        domain: domain!,
        entries: groupEntries,
        finiteCount: groupEntries.filter((entry) => entry.universe.kind === "finite").length,
        reducedProductCount: groupEntries.filter(
          (entry) => entry.universe.kind === "reduced-product",
        ).length,
        noneCount: groupEntries.filter((entry) => entry.universe.kind === "none").length,
        classNames: uniqueSortedStrings(
          groupEntries.flatMap((entry) => classNamesForUniverse(entry.universe)),
        ),
      };
    })
    .toSorted((a, b) => `${a.pluginId}:${a.domain}`.localeCompare(`${b.pluginId}:${b.domain}`));

  return {
    groups,
    totalUniverses: entries.length,
    hasReducedProductUniverse: entries.some((entry) => entry.universe.kind === "reduced-product"),
    classNames: uniqueSortedStrings(
      entries.flatMap((entry) => classNamesForUniverse(entry.universe)),
    ),
  };
}

function groupKey(pluginId: string, domain: string): string {
  return `${pluginId}\0${domain}`;
}

function uniqueSortedStrings(values: readonly string[]): readonly string[] {
  return Array.from(new Set(values)).toSorted();
}

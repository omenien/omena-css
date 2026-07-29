"use client";

import {
  SearchDialog,
  SearchDialogClose,
  SearchDialogContent,
  SearchDialogHeader,
  SearchDialogIcon,
  SearchDialogInput,
  SearchDialogList,
  SearchDialogListItem,
  SearchDialogOverlay,
  type SearchItemType,
  type SharedProps,
  useSearchList,
} from "fumadocs-ui/components/dialog/search";
import { useDocsSearch } from "fumadocs-core/search/client";
import { oramaStaticClient } from "fumadocs-core/search/client/orama-static";
import { useEffect, useRef, useState } from "react";
import { withBasePath } from "@/lib/site";

export function OmenaSearchDialog(props: SharedProps) {
  const returnFocusRef = useRef<HTMLElement | null>(null);
  const [activeDescendant, setActiveDescendant] = useState<string>();
  const { search, setSearch, query } = useDocsSearch({
    client: oramaStaticClient({
      from: withBasePath("/api/search"),
    }),
  });

  useEffect(() => {
    if (!props.open) setActiveDescendant(undefined);
  }, [props.open]);

  return (
    <SearchDialog search={search} onSearchChange={setSearch} isLoading={query.isLoading} {...props}>
      <SearchDialogOverlay />
      <SearchDialogContent
        onOpenAutoFocus={() => {
          returnFocusRef.current =
            globalThis.document.activeElement instanceof globalThis.HTMLElement
              ? globalThis.document.activeElement
              : null;
        }}
        onCloseAutoFocus={(event) => {
          if (!returnFocusRef.current) return;
          event.preventDefault();
          returnFocusRef.current.focus();
        }}
      >
        <SearchDialogHeader>
          <SearchDialogIcon />
          <SearchDialogInput
            aria-activedescendant={activeDescendant}
            aria-autocomplete="list"
            aria-controls="documentation-search-results"
            aria-expanded={props.open}
            aria-label="Search documentation"
            role="combobox"
          />
          <SearchDialogClose />
        </SearchDialogHeader>
        <SearchDialogList
          id="documentation-search-results"
          aria-label="Documentation search results"
          role="listbox"
          items={query.data === "empty" ? null : query.data}
          Item={({ item, onClick }) => (
            <DocumentationSearchResult
              item={item}
              onClick={onClick}
              onActiveDescendantChange={setActiveDescendant}
            />
          )}
        />
      </SearchDialogContent>
    </SearchDialog>
  );
}

function DocumentationSearchResult({
  item,
  onClick,
  onActiveDescendantChange,
}: {
  item: SearchItemType;
  onClick: () => void;
  onActiveDescendantChange: (id: string) => void;
}) {
  const { active } = useSearchList();
  const id = `documentation-search-${encodeURIComponent(item.id).replaceAll("%", "-")}`;

  useEffect(() => {
    if (active === item.id) onActiveDescendantChange(id);
  }, [active, id, item.id, onActiveDescendantChange]);

  return <SearchDialogListItem id={id} item={item} onClick={onClick} role="option" />;
}

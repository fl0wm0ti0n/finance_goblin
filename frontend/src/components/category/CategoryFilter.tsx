import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  fetchCategories,
  UNCATEGORIZED_CATEGORY_ID,
} from "../../lib/api";

export interface CategoryFilterProps {
  value: string;
  onChange: (categoryId: string) => void;
  allowAll?: boolean;
  includeUncategorized?: boolean;
  label?: string;
}

export function CategoryFilter({
  value,
  onChange,
  allowAll = true,
  includeUncategorized = true,
  label = "Category",
}: CategoryFilterProps) {
  const [search, setSearch] = useState("");
  const catalogQuery = useQuery({
    queryKey: ["categories-catalog", search],
    queryFn: () => fetchCategories(search.length >= 2 ? search : undefined),
  });

  const categories = catalogQuery.data?.categories ?? [];
  const useCombobox = categories.length > 20;

  const options = useMemo(() => {
    const items: { value: string; label: string }[] = [];
    if (allowAll) {
      items.push({ value: "", label: "All categories" });
    }
    if (includeUncategorized) {
      items.push({ value: UNCATEGORIZED_CATEGORY_ID, label: "Uncategorized" });
    }
    for (const cat of categories) {
      items.push({ value: cat.id, label: cat.name || cat.id });
    }
    return items;
  }, [allowAll, includeUncategorized, categories]);

  if (useCombobox) {
    return (
      <label style={{ display: "block", marginBottom: "0.75rem" }}>
        {label}
        <input
          type="search"
          className="input"
          placeholder="Search categories…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          style={{ display: "block", width: "100%", marginTop: "0.25rem", marginBottom: "0.25rem" }}
        />
        <select
          className="input"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          style={{ display: "block", width: "100%" }}
        >
          {options.map((opt) => (
            <option key={opt.value || "__all__"} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
        {catalogQuery.data?.truncated && (
          <span style={{ fontSize: "0.85rem", color: "#64748b" }}>
            Showing first 200 matches — refine search.
          </span>
        )}
      </label>
    );
  }

  return (
    <label style={{ display: "block", marginBottom: "0.75rem" }}>
      {label}
      <select
        className="input"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        style={{ display: "block", width: "100%", marginTop: "0.25rem" }}
      >
        {options.map((opt) => (
          <option key={opt.value || "__all__"} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </label>
  );
}

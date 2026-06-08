import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, expect, it } from "vitest";
import { CategoryTrendChart } from "./CategoryTrendChart";

describe("CategoryTrendChart", () => {
  it("shows disabled prompt when no category selected", () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    render(
      <QueryClientProvider client={queryClient}>
        <CategoryTrendChart categoryId="" />
      </QueryClientProvider>,
    );
    expect(screen.getByText(/Select a category to view monthly actual spending/i)).toBeTruthy();
  });
});

import { describe, expect, it } from "vitest";
import { formatPlanningError } from "./planningFeedback";

describe("formatPlanningError", () => {
  it("extracts message from JSON error body", () => {
    expect(
      formatPlanningError(
        new Error(
          JSON.stringify({
            error: "active_plan_delete_forbidden",
            message: "Cannot delete the active plan.",
          }),
        ),
        "Could not delete plan",
      ),
    ).toBe("Cannot delete the active plan.");
  });

  it("returns fallback with Request failed when message empty", () => {
    expect(formatPlanningError(new Error(""), "Could not create plan")).toBe(
      "Could not create plan: Request failed",
    );
  });

  it("returns apiFetch message when present", () => {
    expect(formatPlanningError(new Error("Invalid amount"), "Could not add adjustment")).toBe(
      "Invalid amount",
    );
  });

  it("truncates long messages at 240 chars", () => {
    const long = "x".repeat(300);
    const result = formatPlanningError(new Error(long), "Could not create plan");
    expect(result.length).toBe(241);
    expect(result.endsWith("…")).toBe(true);
  });
});

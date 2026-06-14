import { describe, expect, it } from "vitest";
import type { PlanListItem } from "../lib/api";
import {
  isDeleteDisabled,
  resolveDisplayedPlanId,
  shouldShowSolePlanDeleteHint,
  SOLE_PLAN_DELETE_HINT,
} from "./planSelector";

function plan(id: string, isActive: boolean): PlanListItem {
  return {
    id,
    name: `Plan ${id}`,
    template: "default",
    is_active: isActive,
    latest_version_id: null,
    latest_version_number: null,
    plan_stale: false,
  };
}

describe("resolveDisplayedPlanId", () => {
  it("prefers selected non-active plan when global active exists", () => {
    const plans = [plan("active", true), plan("scenario", false)];
    expect(resolveDisplayedPlanId(plans, "scenario")).toBe("scenario");
  });

  it("falls back to global active when selected is null", () => {
    const plans = [plan("active", true), plan("scenario", false)];
    expect(resolveDisplayedPlanId(plans, null)).toBe("active");
  });

  it("falls back to first plan when no global active and selected is null", () => {
    const plans = [plan("first", false), plan("second", false)];
    expect(resolveDisplayedPlanId(plans, null)).toBe("first");
  });

  it("returns null for empty plans", () => {
    expect(resolveDisplayedPlanId([], null)).toBeNull();
    expect(resolveDisplayedPlanId(undefined, null)).toBeNull();
  });
});

describe("isDeleteDisabled", () => {
  it("enables delete when displayed plan is non-active", () => {
    const plans = [plan("active", true), plan("scenario", false)];
    expect(isDeleteDisabled(plans, "scenario")).toBe(false);
  });

  it("disables delete when displayed plan is active", () => {
    const plans = [plan("active", true), plan("scenario", false)];
    expect(isDeleteDisabled(plans, "active")).toBe(true);
  });

  it("disables delete when displayed plan id is null", () => {
    const plans = [plan("active", true)];
    expect(isDeleteDisabled(plans, null)).toBe(true);
  });

  it("disables delete for empty plans", () => {
    expect(isDeleteDisabled([], null)).toBe(true);
    expect(isDeleteDisabled(undefined, null)).toBe(true);
  });
});

describe("shouldShowSolePlanDeleteHint", () => {
  it.each([
    {
      name: "sole plan active with active selected",
      plans: [plan("sole", true)],
      activePlanIsSelected: true,
      expected: true,
    },
    {
      name: "sole plan active with active not selected",
      plans: [plan("sole", true)],
      activePlanIsSelected: false,
      expected: false,
    },
    {
      name: "two plans with active selected",
      plans: [plan("active", true), plan("scenario", false)],
      activePlanIsSelected: true,
      expected: false,
    },
    {
      name: "two plans with non-active selected",
      plans: [plan("active", true), plan("scenario", false)],
      activePlanIsSelected: false,
      expected: false,
    },
    {
      name: "empty plans",
      plans: [] as PlanListItem[],
      activePlanIsSelected: true,
      expected: false,
    },
    {
      name: "undefined plans",
      plans: undefined,
      activePlanIsSelected: true,
      expected: false,
    },
  ])("$name", ({ plans, activePlanIsSelected, expected }) => {
    expect(shouldShowSolePlanDeleteHint(plans, activePlanIsSelected)).toBe(expected);
  });

  it("exposes create-another-scenario guidance copy", () => {
    expect(SOLE_PLAN_DELETE_HINT).toContain("create another scenario");
  });
});

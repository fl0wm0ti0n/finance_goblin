import type { PlanListItem } from "../lib/api";

export function resolveDisplayedPlanId(
  plans: PlanListItem[] | undefined,
  selectedPlanId: string | null,
): string | null {
  if (!plans || plans.length === 0) {
    return null;
  }
  const globalActiveId = plans.find((p) => p.is_active)?.id ?? null;
  return selectedPlanId ?? globalActiveId ?? plans[0]?.id ?? null;
}

export function isDeleteDisabled(
  plans: PlanListItem[] | undefined,
  displayedPlanId: string | null,
): boolean {
  if (!displayedPlanId || !plans) {
    return true;
  }
  const plan = plans.find((p) => p.id === displayedPlanId);
  return plan?.is_active === true;
}

export const SOLE_PLAN_DELETE_HINT =
  "To delete this plan, create another scenario, set it active, then delete this one.";

export function shouldShowSolePlanDeleteHint(
  plans: PlanListItem[] | undefined,
  activePlanIsSelected: boolean,
): boolean {
  if (!plans || plans.length !== 1) {
    return false;
  }
  return plans[0].is_active === true && activePlanIsSelected === true;
}

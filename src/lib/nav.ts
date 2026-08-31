export type AppRoute = "tasks" | "digests" | "settings";

export const APP_NAVIGATE_EVENT = "app-navigate";

export interface AppNavigatePayload {
  route: AppRoute;
}

export function isAppRoute(value: unknown): value is AppRoute {
  return value === "tasks" || value === "digests" || value === "settings";
}

import { createRouter } from "@tanstack/react-router";
import { deploymentBasePath } from "@/lib/site";
import { routeTree } from "./routeTree.gen";

export function getRouter() {
  return createRouter({
    routeTree,
    basepath: deploymentBasePath || "/",
    defaultPreload: "intent",
    scrollRestoration: true,
  });
}

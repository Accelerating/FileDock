import { type RouteConfig, index, route, layout } from "@react-router/dev/routes";

export default [
  index("routes/home.tsx"),
  layout("routes/layout.tsx", [
    route("browse/*", "routes/browse.tsx"),
  ]),
] satisfies RouteConfig;

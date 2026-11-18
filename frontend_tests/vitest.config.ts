import webConfig from "../frontend/vite.config";

export default {
  ...webConfig,
  resolve: {
    dedupe: ["@tanstack/react-query", "react", "react-dom", "vitest"],
  },
  test: {
    include: ["../frontend_tests/**/*.{test,spec}.{ts,tsx}"],
  },
};

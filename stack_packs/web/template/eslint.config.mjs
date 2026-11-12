// The linter's rules: typescript-eslint's strict, type-checked set, React's hooks and accessibility rules, TanStack
// Router's, and what the architecture adds to them.
import eslint from "@eslint/js";
import pluginRouter from "@tanstack/eslint-plugin-router";
import jsxA11y from "eslint-plugin-jsx-a11y";
import reactHooks from "eslint-plugin-react-hooks";
import { defineConfig } from "eslint/config";
import tseslint from "typescript-eslint";

// What React renders: hooks are React's only there (Nitro's useNitroHooks, in the server entry, is no React hook).
const REACT_FILES = ["src/routes/**/*.tsx", "src/components/**/*.tsx", "src/router.tsx"];

export default defineConfig(
  { ignores: ["node_modules/", ".output/", ".nitro/", ".tanstack/", "src/routeTree.gen.ts", "eslint.config.mjs"] },
  eslint.configs.recommended,
  tseslint.configs.strictTypeChecked,
  pluginRouter.configs["flat/recommended"],
  { files: REACT_FILES, ...reactHooks.configs.flat.recommended },
  { files: REACT_FILES, ...jsxA11y.flatConfigs.strict },
  {
    languageOptions: {
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
    rules: {
      // Every function says what it returns, and every member who may use it.
      "@typescript-eslint/explicit-function-return-type": "error",
      "@typescript-eslint/explicit-member-accessibility": "error",
      // A switch over a union names every member (e.g. every exception family), so a new one fails until it is decided.
      "@typescript-eslint/switch-exhaustiveness-check": ["error", { considerDefaultExhaustiveForUnions: false }],
      "@typescript-eslint/consistent-type-imports": "error",
      // A group of static methods (a validator, a mapper) is a class of static methods.
      "@typescript-eslint/no-extraneous-class": ["error", { allowStaticOnly: true }],
      "@typescript-eslint/restrict-template-expressions": ["error", { allowNumber: true }],
      // TanStack Router's notFound() and redirect() are thrown, as the router expects, and are no Errors.
      "@typescript-eslint/only-throw-error": [
        "error",
        { allow: [{ from: "package", package: "@tanstack/router-core", name: ["NotFoundError", "Redirect"] }] },
      ],
    },
  },
  {
    // The router's type is the one createRouter infers from the route tree; written out, it would lose it.
    files: ["src/router.tsx"],
    rules: { "@typescript-eslint/explicit-function-return-type": "off" },
  },
);

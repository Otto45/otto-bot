// The linter's rules: typescript-eslint's strict, type-checked set, and what the architecture adds to it.
import eslint from "@eslint/js";
import { defineConfig } from "eslint/config";
import tseslint from "typescript-eslint";

export default defineConfig(
  { ignores: ["node_modules/", "eslint.config.mjs"] },
  eslint.configs.recommended,
  tseslint.configs.strictTypeChecked,
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
    },
  },
);

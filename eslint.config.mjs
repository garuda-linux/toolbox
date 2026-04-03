import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import eslintConfig from '@electron-toolkit/eslint-config';
import oxlint from 'eslint-plugin-oxlint';
import angular from 'angular-eslint';
import {dirname} from 'node:path';
import {fileURLToPath} from 'node:url';

export default tseslint.config(
  {
    ignores: ["**/build/**", "**/dist/**", "node_modules"],
  },
  {
    files: ["**/*.ts"],
    extends: [
      eslint.configs.recommended,
      eslintConfig,
      ...tseslint.configs.strict,
      ...tseslint.configs.stylistic,
      ...angular.configs.tsRecommended,
    ],
    processor: angular.processInlineTemplates,
    rules: {
      "@typescript-eslint/no-explicit-any": "warn",
      "@typescript-eslint/no-unused-vars": ["warn", { argsIgnorePattern: "^_" }],
    },
    languageOptions: {
      parserOptions: {
        tsconfigRootDir: dirname(fileURLToPath(import.meta.url)),
        projectService: true,
      },
    },
  },
  {
    files: ["**/*.html"],
    extends: [...angular.configs.templateRecommended, ...angular.configs.templateAccessibility],
    rules: {},
  },
  {
    files: ["types/**/*.d.ts", "assets/**"],
    languageOptions: {
      parserOptions: { projectService: false },
    },
  },
  ...oxlint.buildFromOxlintConfigFile(".oxlintrc.json"),
);

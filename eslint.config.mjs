import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import eslintConfig from '@electron-toolkit/eslint-config';
import oxlint from 'eslint-plugin-oxlint';
import angular from 'angular-eslint';

export default tseslint.config(
  {
    // config with just ignores is the replacement for `.eslintignore`
    ignores: ['**/build/**', '**/dist/**', 'node_modules'],
  },
  {
    files: ['**/*.ts'],
    extends: [
      eslint.configs.recommended,
      eslintConfig,
      ...tseslint.configs.strict,
      ...tseslint.configs.stylistic,
      ...angular.configs.tsRecommended,
    ],
    processor: angular.processInlineTemplates,
    rules: {
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
    },
    languageOptions: {
      tsconfigRootDir: '.',
    },
  },
  {
    files: ['**/*.html'],
    extends: [...angular.configs.templateRecommended, ...angular.configs.templateAccessibility],
    rules: {},
  },
  ...oxlint.buildFromOxlintConfigFile('.oxlintrc.json'),
);

/** Lightweight ESLint for React + TS + Tailwind + shadcn */
module.exports = {
  root: true,
  parser: "@typescript-eslint/parser",
  parserOptions: { ecmaVersion: "latest", sourceType: "module" },
  plugins: ["@typescript-eslint", "react", "react-hooks", "import"],
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:react/recommended",
    "plugin:react-hooks/recommended",
    "plugin:import/typescript",
    "prettier", // turns off stylistic rules that Prettier handles
  ],
  settings: {
    react: { version: "detect" },
    // lets ESLint resolve "@/..." using your tsconfig paths
    "import/resolver": { typescript: true },
  },
  rules: {
    // keep imports tidy
    "import/order": [
      "warn",
      {
        groups: [
          ["builtin", "external"],
          "internal",
          ["parent", "sibling", "index"],
        ],
        "newlines-between": "always",
        alphabetize: { order: "asc", caseInsensitive: true },
      },
    ],
    // TS ergonomics
    "@typescript-eslint/no-unused-vars": [
      "warn",
      { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
    ],
    // React in modern JSX (no need to import React)
    "react/react-in-jsx-scope": "off",
    "react/jsx-uses-react": "off",
  },
  ignorePatterns: [
    "dist",
    "build",
    "coverage",
    "src-tauri/target",
    "src-tauri/**/bundle",
  ],
};

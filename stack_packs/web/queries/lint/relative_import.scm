; A relative import (`./x.ts`, `../x.ts`): imports go through the #src/ alias, so the lint's import rules see every one.
(import_statement source: (string (string_fragment) @violation (#match? @violation "^\\.")))
(export_statement source: (string (string_fragment) @violation (#match? @violation "^\\.")))

; A const, let or var with no type written out. Not flagged: one whose value is a function (its type is the function's
; own, written on it) or an `as const` literal, and a zod schema (`...Schema`), whose type only inference can name (the
; type of what it reads is `z.infer`'s, written beside it); destructuring, and the bindings of `for ... of` and
; `catch`, aren't declarators of a plain name.
(variable_declarator name: (identifier) !type) @violation
(variable_declarator name: (identifier) !type value: (arrow_function)) @allowed
(variable_declarator name: (identifier) !type value: (function_expression)) @allowed
(variable_declarator name: (identifier) !type value: (as_expression . (_) .)) @allowed
((variable_declarator name: (identifier) @name !type) @allowed (#match? @name "Schema$"))

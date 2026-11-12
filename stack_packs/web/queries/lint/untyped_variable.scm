; A const, let or var with no type written out. Not flagged: one whose value is a function (its type is the function's
; own, written on it) or an `as const` literal; a zod schema (`...Schema`), whose type only inference can name (the
; type of what it reads is `z.infer`'s, written beside it); what TanStack Start builds (a route, a server function, a
; middleware, the start instance), and a hook's result, whose types only inference can name too. Destructuring, and
; the bindings of `for ... of` and `catch`, aren't declarators of a plain name.
(variable_declarator name: (identifier) !type) @violation
(variable_declarator name: (identifier) !type value: (arrow_function)) @allowed
(variable_declarator name: (identifier) !type value: (function_expression)) @allowed
(variable_declarator name: (identifier) !type value: (as_expression . (_) .)) @allowed
((variable_declarator name: (identifier) @name !type) @allowed (#match? @name "Schema$"))
((variable_declarator name: (identifier) !type value: (call_expression) @value) @allowed
  (#match? @value "^create(FileRoute|RootRoute|ServerFn|Middleware|Start|Router)\\("))
((variable_declarator name: (identifier) !type value: (call_expression function: (identifier) @hook)) @allowed
  (#match? @hook "^use[A-Z]"))
((variable_declarator name: (identifier) !type
  value: (call_expression function: (member_expression property: (property_identifier) @hook))) @allowed
  (#match? @hook "^use[A-Z]"))

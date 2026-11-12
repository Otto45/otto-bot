; The exported items of a TypeScript or TSX file, for the project map. @item is an item to show; a @body or an @omit
; inside it is left out, so a function, method or component shows as its signature, a class as its declaration line
; (what it extends and implements), and its public and protected members as items of their own. Interfaces (a
; component's props among them), type aliases and exported constants (a zod schema, LogLevel, a route, a server
; function) show whole, but for the bodies of the functions they are built with.

(export_statement declaration: (class_declaration body: (class_body) @body)) @item
(export_statement declaration: (abstract_class_declaration body: (class_body) @body)) @item

(class_body (method_definition (accessibility_modifier) @access body: (statement_block) @body) @item
  (#not-eq? @access "private"))
(class_body (abstract_method_signature (accessibility_modifier) @access) @item (#not-eq? @access "private"))
(class_body (public_field_definition (accessibility_modifier) @access) @item (#not-eq? @access "private"))

(export_statement declaration: (interface_declaration)) @item
(export_statement declaration: (type_alias_declaration)) @item
(export_statement declaration: (lexical_declaration)) @item
(export_statement declaration: (function_declaration body: (statement_block) @body)) @item

; What a server function's handler, a middleware's server half and a route's loader do (a block of statements) is
; left out: the map shows its method, its request model and what it returns, or the server function a loader calls.
(export_statement declaration: (lexical_declaration (variable_declarator value: (call_expression
  arguments: (arguments (arrow_function body: (statement_block) @body))))))
(export_statement declaration: (lexical_declaration (variable_declarator value: (call_expression
  arguments: (arguments (object (pair value: (arrow_function body: (statement_block) @body))))))))

(comment) @omit

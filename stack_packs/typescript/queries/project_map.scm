; The exported items of a TypeScript file, for the project map. @item is an item to show; a @body or an @omit inside it
; is left out, so a function or method shows as its signature, a class as its declaration line (what it extends and
; implements), and its public and protected members as items of their own. Interfaces, type aliases and exported
; constants (a zod schema, LogLevel) show whole.

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

(comment) @omit

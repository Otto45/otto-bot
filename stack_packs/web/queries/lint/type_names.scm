; The names a file declares: its classes, interfaces and type aliases, its functions (a component), and its exported
; constants (a server function, a middleware).
(class_declaration name: (type_identifier) @name)
(abstract_class_declaration name: (type_identifier) @name)
(interface_declaration name: (type_identifier) @name)
(type_alias_declaration name: (type_identifier) @name)
(program (function_declaration name: (identifier) @name))
(program (export_statement declaration: (function_declaration name: (identifier) @name)))
(program (export_statement declaration: (lexical_declaration (variable_declarator name: (identifier) @name))))

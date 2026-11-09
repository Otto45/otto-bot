; A function outside a class (only an entry point's module may have them): declared, or an arrow function held in a
; module-level constant.
(program (function_declaration) @violation)
(program (export_statement declaration: (function_declaration) @violation))
(program (lexical_declaration (variable_declarator value: (arrow_function))) @violation)
(program (lexical_declaration (variable_declarator value: (function_expression))) @violation)
(program (export_statement declaration: (lexical_declaration (variable_declarator value: (arrow_function))) @violation))
(program (export_statement declaration: (lexical_declaration (variable_declarator value: (function_expression))) @violation))

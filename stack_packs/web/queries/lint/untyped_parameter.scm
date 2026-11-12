; A parameter with no type written out (one with a default value, which the compiler would otherwise infer). Not
; flagged: a callback's, passed as an argument or as an option, which the function it is passed to types.
(required_parameter pattern: (identifier) !type) @violation
(optional_parameter pattern: (identifier) !type) @violation
(arrow_function parameter: (identifier) @violation)

(arguments (arrow_function parameters: (formal_parameters [(required_parameter) (optional_parameter)] @allowed)))
(arguments (arrow_function parameter: (identifier) @allowed))
(pair value: (arrow_function parameters: (formal_parameters [(required_parameter) (optional_parameter)] @allowed)))
(pair value: (arrow_function parameter: (identifier) @allowed))

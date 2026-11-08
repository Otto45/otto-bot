; In a Protocol: a method whose body isn't `...`, or a field (a Protocol declares method signatures only).
(class_definition
  superclasses: (argument_list (identifier) @base)
  body: (block (function_definition body: (block) @violation))
  (#eq? @base "Protocol"))
(class_definition
  superclasses: (argument_list (identifier) @base)
  body: (block (expression_statement (assignment) @violation))
  (#eq? @base "Protocol"))
(function_definition body: (block . (expression_statement (ellipsis)) .) @allowed)

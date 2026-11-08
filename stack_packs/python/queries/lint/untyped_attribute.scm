; An instance attribute first set in __init__ with no type annotation (`self._pool = pool`).
(function_definition
  name: (identifier) @function_name
  body: (block (expression_statement
    (assignment left: (attribute object: (identifier) @object) !type) @violation))
  (#eq? @function_name "__init__")
  (#eq? @object "self"))

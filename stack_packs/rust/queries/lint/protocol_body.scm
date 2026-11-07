; In a trait named ...Protocol: a method with a default body, or an async method.
(trait_item
  name: (type_identifier) @trait_name
  (#match? @trait_name "Protocol$")
  body: (declaration_list (function_item) @violation))
(trait_item
  name: (type_identifier) @trait_name
  (#match? @trait_name "Protocol$")
  body: (declaration_list (function_signature_item (function_modifiers) @violation)))

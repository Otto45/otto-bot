; In an interface named ...Protocol: a field (a Protocol declares method signatures only).
(interface_declaration
  name: (type_identifier) @name
  body: (interface_body (property_signature) @violation)
  (#match? @name "Protocol$"))

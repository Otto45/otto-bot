; A class field with no type written out, unless it is an `as const` literal.
(public_field_definition name: (_) !type) @violation
(public_field_definition name: (_) !type value: (as_expression . (_) .)) @allowed

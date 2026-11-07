; A let binding with no type written out, unless its value's type only inference can name (a closure, an async
; block), or it destructures with `else` (refutable, like `if let`).
(let_declaration pattern: [(identifier) (tuple_pattern)] !type) @violation
(let_declaration pattern: [(identifier) (tuple_pattern)] !type value: [(closure_expression) (async_block)]) @allowed
(let_declaration pattern: [(identifier) (tuple_pattern)] !type alternative: (_)) @allowed

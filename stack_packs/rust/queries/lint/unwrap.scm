; A call to .unwrap(): it panics on what is usually an expected failure.
(call_expression
  function: (field_expression field: (field_identifier) @method)
  (#eq? @method "unwrap")) @violation

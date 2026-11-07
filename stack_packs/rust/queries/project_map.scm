; The public items of a Rust file, for the project map. @item is an item to show; a @body or an @omit inside it is
; left out, so a function shows as its signature, a trait as its name (its methods' signatures match on their own),
; and nothing shows a comment.

(function_item (visibility_modifier) body: (block) @body) @item
(function_signature_item) @item
(struct_item (visibility_modifier)) @item
(enum_item (visibility_modifier)) @item
(trait_item (visibility_modifier) body: (declaration_list) @body) @item
(type_item (visibility_modifier)) @item

(line_comment) @omit
(block_comment) @omit

; The public items of a Python file, for the project map. @item is an item to show; a @body or an @omit inside it is
; left out, so a function shows as its signature (with its decorators), a class as its declaration line, and its
; fields and methods as items of their own. A name starting with one underscore is private, so it isn't shown;
; dunder methods (__init__, __aenter__) are.

; Module-level classes, with their decorators (@final, @dataclass(...)).
(module (class_definition name: (identifier) @name body: (block) @body) @item (#not-match? @name "^_[^_]"))
(module (decorated_definition
  definition: (class_definition name: (identifier) @name body: (block) @body)) @item (#not-match? @name "^_[^_]"))

; A class's fields (a dataclass's, a pydantic model's, a Protocol's).
(class_definition body: (block (expression_statement
  (assignment left: (identifier) @name type: (_)) @item)) (#not-match? @name "^_"))

; An enum's members (which have no annotation).
(class_definition
  superclasses: (argument_list (identifier) @base)
  body: (block (expression_statement (assignment left: (identifier) !type) @item))
  (#match? @base "Enum$"))

; A class's methods, with their decorators (@staticmethod, @classmethod).
(class_definition body: (block
  (function_definition name: (identifier) @name body: (block) @body) @item) (#not-match? @name "^_[^_]"))
(class_definition body: (block (decorated_definition
  definition: (function_definition name: (identifier) @name body: (block) @body)) @item) (#not-match? @name "^_[^_]"))

; Module-level functions (an entry point's main) and type aliases.
(module (function_definition name: (identifier) @name body: (block) @body) @item (#not-match? @name "^_"))
(module (decorated_definition
  definition: (function_definition name: (identifier) @name body: (block) @body)) @item (#not-match? @name "^_"))
(module (type_alias_statement) @item)

(comment) @omit

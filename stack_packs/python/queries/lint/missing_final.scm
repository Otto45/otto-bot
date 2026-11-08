; A module-level class not marked @final. Not flagged: a model (a dataclass, a pydantic BaseModel), a Protocol, an
; ABC, an enum, and an exception (a family's base class is inherited from; its exceptions are @final, which is
; checked by reading).
(module (class_definition) @violation)
(module (decorated_definition definition: (class_definition) @violation))

(module (decorated_definition
  (decorator [(identifier) @decorator (call function: (identifier) @decorator)])
  definition: (class_definition) @allowed)
  (#any-of? @decorator "final" "dataclass"))
(module (class_definition superclasses: (argument_list (identifier) @base)) @allowed
  (#any-of? @base "Protocol" "BaseModel" "ABC" "Enum" "IntEnum" "StrEnum"))
(module (decorated_definition
  definition: (class_definition superclasses: (argument_list (identifier) @base)) @allowed)
  (#any-of? @base "Protocol" "BaseModel" "ABC" "Enum" "IntEnum" "StrEnum"))
(module (class_definition name: (identifier) @name) @allowed (#match? @name "Exception$"))

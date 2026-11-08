; An assignment to a plain name with no type annotation, rebinding included (pyright accepts the same annotation
; again). Not flagged: dunder names (__slots__), an enum's members (which may not be annotated), and the targets
; Python can't annotate (tuple unpacking, for, with, except, comprehensions), which aren't plain-name assignments.
((assignment left: (identifier) @name !type) @violation (#not-match? @name "^__.*__$"))
(class_definition
  superclasses: (argument_list (identifier) @base)
  body: (block (expression_statement (assignment left: (identifier) !type) @allowed))
  (#match? @base "Enum$"))

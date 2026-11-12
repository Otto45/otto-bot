; A colour of Tailwind's own palette (`bg-blue-600`, `text-white`) or an arbitrary one (`bg-[#3b82f6]`) in a string
; (a className, or a map of them): the theme's tokens are the only colours there are.
((string_fragment) @violation
  (#match? @violation "(^|[\\s:'\"!])-?(bg|text|border(-[trblxyse])?|ring(-offset)?|outline|fill|stroke|from|via|to|decoration|divide|placeholder|caret|accent|shadow|inset-shadow|inset-ring)-((slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-[0-9]{2,3}|black|white|\\[(#|rgb|hsl|oklch|oklab|lab|lch|color))"))

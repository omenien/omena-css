# omena-value-lattice

Region-local CSS value lens and canonical value equality substrate for Omena CSS.

The crate intentionally accepts declaration value slices, not rules or stylesheets.
That keeps value reasoning local to the source region that produced the value and
prevents this layer from becoming a whole-document property AST.

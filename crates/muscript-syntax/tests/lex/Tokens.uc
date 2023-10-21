// This is a comment. It should be ignored when parsing, but still lexed properly.
/* This is a block comment.
   These can span multiple lines /* and can also nest /* infinitely */. */
   They should not screw up the line counter (because there isn't one :^) ) */

lowerIdent
UpperIdent
snake_ident

// Literals
none None true True TRUE false False FALSE
1234 0xAABBCCDD
1.0 1.0e-1
"Hello, world!"
'Jeff'

// Operators
+ - * ** / dot cross
<< >> >>> & |
$ @
: ?
! == != ~= < > <= >= && ||
=

// Also no compound assignments since those are any of the above followed by the token =.

// Sigils and delimiters
() [] {}
, ; `

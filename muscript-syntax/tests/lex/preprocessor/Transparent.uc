// Test that the preprocessor is transparent to its TokenStream's user.

`define EXAMPLE class Example extends Object pecl;

`EXAMPLE

// The tokens produced above should form a class declaration.

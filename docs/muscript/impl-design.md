# The design of the MuScript compiler

This document outlines some design decisions behind MuScript.

## Concrete syntax tree

MuScript parses the source code into a concrete syntax tree (CST) instead of the more traditional
abstract syntax tree (AST). This is done for convenience reasons - it makes writing rules inside the
parser as simple as declaring a struct with `#[derive(Parse)]`. For example, here's the parser for
`if` statements:

```rust
#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct StmtIf {
    pub kif: KIf,
    pub cond: ParenExpr,
    pub true_branch: Box<Stmt>,
    pub false_branch: Option<Else>,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct ParenExpr {
    pub open: LeftParen,
    pub cond: Expr,
    pub close: RightParen,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct Else {
    pub kelse: KElse,
    pub then: Box<Stmt>,
}
```

Here's a rundown of what each `derive`d trait does:

- `Parse` does the actual parsing work; it turns tokens from the input stream into the concrete
  syntax tree.
- `PredictiveParse` allows us to peek at an input token to determine whether it starts the rule;
  for example, for `StmtIf` it returns `true` if it sees the keyword `if`. If `PredictiveParse` is
  implemented for `T`, then `Option<T>` implements `Parse`, and `T` can be used as a field in a
  `#[derive(Parse)]` enum's variant.
- `Spanned` allows us to extract a span covering the entire syntactic construction, instead of just
  singular tokens.

### Partitions

Since the CST is quite hard to process in its raw form, before any attempts at interpreting its
contents are made each class is divided up into a set of _partitions_, where each partition
corresponds to a single file (a class can have multiple files if it's `partial`.)

The job of a partition is to collate the items declared in a class into data structures that are
easier to work with. The raw list of items as they appear in the CST is divided into multiple
scopes - variables, types, and functions. This is also the point at which naming conflicts are
resolved.

Once computed, a class's partitions are the single source of truth about items declared in the
class. The names and CSTs of items can be looked up from any of the aforementioned scopes.

## Compile as little as possible

The best way to go fast is to go lazy. The less unnecessary work you have to do, the less time doing
the entire task will take, thus MuScript only compiles code that is actually used.

Note that when exporting a package, all classes within that package are considered used - heck, they
have to be, otherwise we wouldn't generate any bytecode for them - so **your mod** is analyzed
either way.

This can lead to surprising behavior when you try to use a class which contains syntax errors.
There are few such classes in A Hat in Time, but they do exist. Most of them are fairly deep in the
inheritance hierarchy though, so it's unlikely you'll ever stumble upon them, but in case you do
you will need to either copy the class or edit its source code to fix the error.

Sorry, I am not GfB, I have no power over engine code!

## Demand-based rather than pass-based

Traditional compilers are implemented in a way where separate passes are executed on the source.
If it were implemented this way, MuScript would first parse the file into a concrete syntax tree
(CST), then lower it into a partition, then analyze every single item declared of the class, then
emit code for every function, and then package everything up. This approach however performs an
unnecessarily large amount of work and it's very hard to implement incremental compilation with it.

That's why MuScript uses an architecture inspired by rustc, where instead we tell the compiler
"hey, compile me these classes." Then the compiler requests function code for every class to
package it up, but of course functions are not analyzed yet, so we do that, but to analyze functions
we need to find out what items are in the class, and to do that we need the class's CST.

We make every piece of the pipeline into a pure function - which is a function whose output
depends solely on its arguments - and then memoize (cache) the results. That's the foundation of
rustc's query system, and conversely, MuScript.

Unlike rustc however, MuScript does not have a sophisticated query system in place. In the name of
simplicity and behavioral transparency, we perform caching manually instead of using complicated
macro logic. This makes the source code a lot easier to understand, at the expense of just having
more of it to deal with.

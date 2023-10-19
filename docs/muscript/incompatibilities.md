# Incompatibilities between MuScript and UnrealScript

MuScript aims to improve on UnrealScript's design by removing features that can be easily misused,
or augmenting existing features with usability improvements.

The following is a listing of intentional incompatibilities between MuScript and vanilla
UnrealScript.

## Laziness

The most surprising aspect of MuScript might be that it does not compile anything until you
explicitly ask it to. This is the reason MuScript is so fast; it does not look at code you do not
care about.

It is also the reason why MuScript is capable of compiling a great deal of code despite not being
fully compatible with vanilla UnrealScript. Even `Object.uc` causes a bunch of compilation errors
if you compile the `Core` package itself (and those are intentional - it _will_ continue to not
compile, probably forever.) All these errors occur within function bodies, which MuScript does not
look at unless you're actually compiling the package.

However, this can cause some fairly surprising behavior. For example, MuScript currently does not
look at items declared within `.uci` files. They _are parsed_, but are not _analyzed_. This results
in `EPixelFormat` not being visible to the compiler; this is a problem if you ever try to read
`Texture2D`'s `Format` property, because the compiler cannot find its type (remember that `.uci`
files are not analyzed, and that results in `EPixelFormat` being undeclared.)

Of course there are a bunch more `EPixelFormat` variables you cannot read from, but you get
the idea.

The main point is that `Texture2D`'s `Format` property will not produce an error until you try to
use it, because the compiler does not process code you do not care about. And the same happens with
any other item the compiler sees.

## Syntax

### Preprocessor

The MuScript preprocessor differs from the UnrealScript preprocessor quite significantly, as it
operates at the lexical level rather than performing a primitive string search and replace before
the code is passed onto the lexer.

This generally leads to better error messages, though you shouldn't rely on the preprocessor too
much anyways, since MuScript generally has better ways of handling the common cases where the
preprocessor is used.

Because the preprocessor operates quite differently, several incompatibilities can be observed:

- `if` expands if the token stream in the parentheses contains at least one token.
- `isdefined`, expands nothing when the macro is not defined, or _literally_ the tokens
  <code>\`isdefined(MACRO_NAME)</code> when the macro is defined, and <code>\`</code> is not
  a valid token in the MuScript syntax. Therefore `isdefined` is only usable inside the `if` macro.
  - Naturally, the same thing happens with `notdefined`.
- `include` is ignored. All .uci files are included by default.
- The preprocessor does not run inside strings. Therefore, macros such as `ShowVar` do not work.
- Not tested, but the MuScript preprocessor is probably more strict than the UnrealScript
  preprocessor around some places.
  - It implements all features such that it can process the entire engine and game source code
    without errors, but it may not replicate quirks such as allowing mismatched parentheses
    (though none of these quirks were actually tested for! for what it's worth, UPP might disallow
    mismatched parentheses. I simply don't know.)

### Default properties

MuScript's `defaultproperties` syntax is a lot more strict than UnrealScript's, since MuScript
actually integrates `defaultproperties` into its parser. Therefore not all classes from the engine
or AHiT may parse correctly.

In particular, vanilla UnrealScript allows garbage between properties, but MuScript doesn't.
Therefore something like this is not allowed:

```unrealscript
defaultproperties
{
    Example = SomeClass'SomeObject',    // Error.
    Example2 = SomeClass'OtherObject')  // Error, too.
    Garbage                             // Also an error.
}
```

The above three cases occur in engine and game code and can be fixed by removing the
offending token.

On the other hand, MuScript enhances default properties with more consistent syntax. Consider for
example that UnrealScript will allow using hexadecimal integer literals for values of `Int`
variables, but fails to parse hexadecimal literals in structs, so `(X = 0xABC)` does not work.
Since MuScript tokenizes default properties the same as the rest of all your code, it handles that
case correctly.

Since default properties are tokenized like the rest of your code, that also means newlines are not
significant. Therefore it's not necessary to wrap multiline structs or arrays in braces `{}`; the
following parses and works fine:

```unrealscript
defaultproperties
{
    Example = (
        X = 1,
        Y = 2,
        Z = 3,
    )
}
```

Wrapping struct and array literals in braces is allowed for backwards compatibility, but not doing
so should be preferred in modern code.

#### Include files

The behavior around `.uci` files is quite different from vanilla UnrealScript.

During compilation, `.uci` files are processed before `.uc` files. Macros defined in said files
are collected into a single namespace, which is then used as the base macro namespace for all `.uc`
files (on top of macros defined via the command line.) Note that this namespace is duplicated for
each `.uc` file, so macros defined in one `.uc` file will not be visible in any other `.uc` file.

Some `.uci` files also define items such as `enum`s and `const`s. These are read by the parser,
but not analyzed; therefore these items are not visible in any namespace. This means it is
impossible to use eg. variables whose type is `EPixelFormat`.

### Unsigned arithmetic right shift `>>>` operator

This operator is parsed correctly inside function definitions, but is not supported in expressions
due to how limited our current parsing of multi-character operators is. This limitation might be
lifted in the future (and probably with it, the possibility of overloading your own operators would
be introduced. Maybe one day.)

## Type system

The following sections describe differences between the UnrealScript and MuScript type systems.

### Type coercions

**Type coercions** are what happens when one type turns into another, compatible type automatically
(without requiring an explicit cast.) The following coercions are always applied, both in
UnrealScript and MuScript:

- More specific object to less specific object, eg. `Actor` coerces to `Object`, but not the other
  way around, since not every `Object` may be an `Actor`.
- More specific class to less specific class, eg. `Class<Actor>` coerces to `Class<Object>`, but
  not the other way around, since not every `Class<Object>` may inherit from `Actor`.
- `Array<T>` may coerce to `Array<U>` if `T` and `U` are object or class types and the above rules
  hold true

The following coercions are done by vanilla UnrealScript, but **not done** by MuScript:

- `Byte`, `Int`, and `Float` may convert between each other freely. These automatic conversions are
  not performed since every one (except `Byte` to `Int`) incurs a precision loss. The `Byte` to
  `Int` or `Float` conversion is not performed implicitly for consistency sake, but this rule may
  be relaxed in the future.
- Implicit conversions on parameters marked `coerce`. MuScript ignores `coerce` and always requires
  an explicit conversion, since that's less prone to errors and makes performance more predictable
  (since `String()` conversions are not exactly cheap.)

MuScript also carries an _expected type_ with each expression, such that it may perform more
contextual type inference where necessary, thus avoiding the need for some casts around literals.
For example, this means that despite MuScript's more strict rules around numeric types, the
following code compiles just fine:

```unrealscript
function Float Example()
{
    return 1;  // An integer literal! These convert to floats automatically.
}
```

On the other hand, the following code does not compile, since there is no operator overload defined
for `Int / Float`:

```unrealscript
function Float Reciprocal(Float x)
{
    return 1 / x;  // Error.
}
```

To fix this, you can turn the integer literal into a float literal by adding a decimal point:

```unrealscript
function Float Reciprocal(Float x)
{
    return 1.0 / x;  // All good!
}
```

which will choose the overload `Float / Float`, which is defined and works as expected.

One other quirk around literals you may come across is that `Int` is always preferred over `Byte`,
so for example the following does not compile:

```unrealscript
local Byte b;
b = 1;   // All good, since given that the variable `b` is of type `Byte`,
         // the type system will expect `Byte` on the right.
b += 1;  // This is an error: no overload of operator `+=` exists for `Byte`, `Int`
```

This is because `=` is implemented using compiler magic, and the compiler is smart enough to use
the left-hand side's type as a hint on what should be expected on the right-hand side - and in cases
where there's an integer literal that we know we'd like to be `Byte`, it'll be converted
automatically. `+=` is not compiler magic but a regular operator just like `+` - therefore
this type hinting does not apply and the compiler cannot infer that the right-hand side is expected
to be a `Byte`.

This is something we would like to be able to compile in the future, but is not supported as of now.
To work around it, specify an explicit type for the literal by using a type cast:

```unrealscript
local Byte b;
b = 1;
b += Byte(1);  // All good now.
```

### Conditions

Conditions in `if`, `while`, and `for` statements are not converted to `Bool` automatically.
Therefore explicit comparisons are always required.

```unrealscript
function Bad(Object o)
{
    if (o)
    {
        // Doesn't compile...
    }
}

function Good(Object o)
{
    if (o != none)
    {
        // All good!
    }
}
```

## Local variables

MuScript allows defining local variables anywhere in a block, not just at the top of the function:

```unrealscript
function Example()
{
    local Int i;
    i = 1;

    local Float f;
    f = 2;

    {
        local Float g;
    }
    // g = 3; // Referring to g here is disallowed since we're outside
              // the block it was declared in.
}
```

## Optimizations

Currently the MuScript compiler is pretty dumb; its internal representation of code is more
primitive and friendlier to optimizations than UnrealScript's, but no advanced optimizations are
performed on it currently. Therefore at this point you can expect MuScript code to run about on par
with the equivalent UnrealScript code, but the situation will get better with each release, as the
compiler is taught about common patterns which can be written more optimally.

You can rely on the fact that MuScript will never sacrifice correctness for performance; it's more
important that the code does what you expect than that it runs fast.

TL;DR: MuScript may generate vastly different bytecode than UnrealScript but it should behave
the same in the end.

## Conventions

Vanilla UnrealScript is pretty inconsistent when naming things. MuScript aims to tame that a bit:

- We choose to use `PascalCase` names for _all_ types, including primitives
  (like `Float` - which may look a bit surprising in error messages.)
- Keywords are always spelled lowercase, including `none` (which is sometimes spelled as `None` in
  engine code.)
  - Perhaps the most surprising set of words to be spelled that way is `begin object` and
    `end object`, which is usually written as `Begin Object` and `End Object`.
- The compiler does its best to preserve the casing of your identifiers, so if you get warned about
  an unused variable that you named `i` (lowercase), the compiler will not report it to you as `I`
  (uppercase).

Since MuScript is case-insensitive these are purely cosmetic differences and you're still free to
choose whichever style you prefer.

# Incompatibilities between MuScript and UnrealScript

MuScript aims to improve on UnrealScript's design by removing features that can be easily misused,
or augmenting existing features with usability improvements.

The following is a listing of intentional incompatibilities between MuScript and vanilla
UnrealScript.

## Default properties

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

The above three cases occur in engine and game code and can be fixed simply by removing the
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

## Preprocessor

The MuScript preprocessor differs from the UnrealScript preprocessor quite significantly, as it
operates at the lexical level rather than performing a primitive string search and replace before
the code is passed onto the lexer.

This generally leads to better error messages, though you shouldn't rely on the preprocessor too
much anyways, since MuScript generally has better ways of handling the common cases where the
preprocessor is used.

Because the preprocessor operates quite differently, several incompatibilities can be observed:

- `if` expands if the token stream in the parentheses contains at least one token.
- `isdefined`, when the provided macro is defined, expands to an unspecified token that is not
  representable nor valid in human-written source code. Therefore `isdefined` is only usable inside
  the `if` macro.
  - Naturally, the same thing happens with `notdefined`.
- `include` is ignored. All .uci files are included by default.
- The preprocessor currently does not run inside strings. Therefore, macros such as `ShowVar` do not
  work.
- Not tested, but the MuScript preprocessor is probably more strict than the UnrealScript
  preprocessor around some places.
  - It implements all features such that it can process the entire engine and game source code
    without errors, but it may not replicate quirks such as allowing mismatched parentheses
    (though none of these quirks were actually tested for! for what it's worth, UPP might disallow
    mismatched parentheses. I simply don't know.)

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
  `Int` conversion is not performed implicitly for consistency sake, but this rule may be relaxed in
  the future.
- Implicit conversions on parameters marked `coerce`. MuScript ignores `coerce` and always requires
  an explicit conversion, since that's less prone to errors and makes performance more predictable
  (since `string()` conversions are not exactly cheap.)

MuScript also carries an _expected type_ with each expression, such that it may perform more
contextual type inference where necessary, thus avoiding the need for some casts around literals.
For example, this means that despite MuScript's more strict rules around numeric types, the
following code compiles just fine:

```unrealscript
function float Example()
{
    return 1;  // An integer literal! These convert to floats automatically.
}
```

On the other hand, the following code does not compile, since there is no operator overload defined
for `Int / Float`:

```unrealscript
function float Reciprocal(float x)
{
    return 1 / x;  // Error.
}
```

To fix this, you can turn the integer literal into a float literal by adding a decimal point:

```unrealscript
function float Reciprocal(float x)
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

This is something we would like to be able to compile in the future, but is not supported as of now.
To work around it, specify an explicit type for the literal by using a type cast:

```unrealscript
local Byte b;
b = 1;
b += Byte(1);  // All good now.
```

### Conditions

Conditions in `if`, `while`, and `for` statements are not converted to `bool` automatically.
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

## Optimizations

Currently the MuScript compiler is pretty dumb; its internal representation of code is more
primitive and friendlier to optimizations than UnrealScript's, but no advanced optimizations are
performed on it currently. Therefore at this point you can expect MuScript code to run about on par
with the equivalent UnrealScript code, but the situation will get better with each release, as the
compiler is taught about common patterns which can be written more optimally.

MuScript will never sacrifice correctness for performance; it's more important that the code does
what you expect than that it runs fast.

## Conventions

Vanilla UnrealScript is pretty inconsistent when naming things. MuScript aims to tame that a bit:

- We choose to use `PascalCase` names for _all_ types, including primitives
  (like `Float` - which may look a bit surprising in error messages.)
- Keywords are always spelled lowercase, including `none` (which is sometimes spelled as `None` in
  engine code.)
  - Perhaps the most surprising set of words to be spelled that way is `begin object` and
    `end object`, which is usually written as `Begin Object` and `End Object`.

Since MuScript is case-insensitive these are purely cosmetic differences and you're still free to
choose whichever style you prefer.

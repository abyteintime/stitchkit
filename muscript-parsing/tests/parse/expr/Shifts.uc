class Example extends Object;

var array<class<object>> ThisIsValid;

function Exprs()
{
    local int a, b;
    local bool bb;

    bb = a < b;
    bb = a > b;
    bb = a << b;
    bb = a >> b;
    bb = a >>> b;
}

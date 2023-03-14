class Example extends Object;

`define debugEffectIsRelevant(msg, cond) if (`cond) { (`msg); }

function Test()
{
    `debugEffectIsRelevant("boo", true);
}

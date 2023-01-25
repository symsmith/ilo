# ilo

A simple interpreted scripting language.

[![Coverage badge](https://ilo-coverage.vercel.app/badges/flat.svg)](https://ilo-coverage.vercel.app/)


## Current state

- [x] Comments
- [x] Simple expressions (math, simple strings (no interpolation))
- [x] Expression statements
- [x] `out()` statement
- [x] Global variables
- [x] Empty variables
- [x] Block statements
- [x] `if` / `else`
- [ ] Everything else

## Installation

1. Clone the project
```bash
git clone git@github.com:symsmith/ilo.git
```
2. `cd` into the project
```bash
cd ilo
```
3. Check the usage of the CLI
```bash
cargo run -- -h # use -- to pass arguments
```

## Syntax

### Comments

```jsx
// single-line comment
/* multiline
comment */
```

### Variables

```jsx
// declaration
a = 2
// assignment
a = 3
// fixed type
a = "string" // type error
// usage
out(a * 2)
```

Variables are scoped to statements: basically, if you define a variable inside braces you can’t use it outside of them.

Since the syntax for declaration and reassigment is the same, the outer scopes are searched in "inside to outside" order when assigning a value. If no variable with this name is found, it is created in this scope.

```jsx
if true {
  b = 4 // declaration
  a = 3 // assignment of outer scope is fine
}
out(b) // runtime error: b not found
```

The rule is: you can use any variable in your scope or those above, and the variable you create can be used in your scope and those below yours.

#### Empty

```jsx
// `empty` means "no value"
a = empty(number) // empty primary value needs to be typed (boolean or number)
a = 3             // type gets set
a = "a"           // type error
a = empty         // type is still number, no explicit type needed
a = "a"           // type error

// an empty string is defined only with ""
b = ""
c = empty(string) // syntax error: empty string variables cannot be initialized this way
```

#### Numbers

```jsx
// numbers are 64-bit floats (doubles)
a = 2
a = 2.0 * 3
a = 3 % 2  // 1 (remainder of euclidean division)
a = -3 % 2 // 1
a = 2^3    // 8 (exponentiation)
a /= 4     // 2; or +=, -=, *=, %=, ^=
a++        // 3; or a--
```

#### Booleans

```jsx
a = 2
a == 2                // true
a != 2                // false
1 == 2 or 1 == 1      // true
1 == 1 and 2 == 2     // true
!(1 == 2) == (1 != 2) // true
1 == true             // false
"a" != 4              // true
/*
or: evaluates left-hand-side and skips evaluation
    of right-hand side if it is true
and: evaluates left-hand-side and skips evaluation
    of right-hand side if it is false
*/
```

#### Strings

```jsx
m = "strings can be
multiline"
"a" + "b" // "ab"
"a" + 3   // type error
"a" * 3   // "aaa"
"a" * 0   // ""
"a" * -1  // runtime error

a = "world"
b = "hello {a + "!"}" // "hello world!"
b = "hello {{a + !}}" // "hello {a + !}" ({{ escapes {)

b = r"hello {a}" // "hello {a}" (raw string)

out("hello" + " world") // "hello world"
size("hello")           // 5

a[0]           // "w"
a[-2]          // "l"
a[1] = "d"     // a == "wdrld"
a[2] = "hello" // runtime error
a[3] = 3       // type error
```

#### Lists

```jsx
// creation
// explicit type for empty list to enforce single type
a = number[]    // or a = string[], a = boolean[],
                // a = string[][]...
b = []          // type error, explicit type needed
b = ["a", 1]    // type error
a = [3]
s = ["a"] * 3   // ["a", "a", "a"], all different references
t = [1, 2] * 3  // [1, 2, 1, 2, 1, 2]
t = a * 3       // [3, 3, 3]
t = [1...3]     // [1, 2, 3]
t = [1...1]     // [1]
t = [1...-1]    // [1, 0, -1]
t = [1.2...3.4] // type error, int needed both sides

// read, write
s[1]        // "a"
s[1] = "b"  // s == ["a", "b", "a"]
s[2] = 3    // type error
s[-1] = "b" // s == ["a", "b", "b"]

// append
a = [3]
b = a + 2             // [3, 2]
b = 2 + a             // [2, 3]
b = [1, 2] + "a"      // type error
b = [1, 2] + [3, 4]   // [1, 2, 3, 4]
b = [[1, 2]] + [3, 4] // [[1, 2], [3, 4]]

// filter
b = [2, 1, 2] - 2      // [1] (remove occurences)
b = [2, 1, 3] - [2, 3] // [1] (remove occurences)
b = [1, 2] - "a"       // [1, 2] (no occurence)

// pop
b = [1, 2] / 1          // [1] ("slash"/cut n last)
b = [1, 2] / "a"        // type error
b = 1 / [1, 2]          // [2] ("slash"/cut n first)
b = 1 / [1, 2] / 1      // [] = (1 / [1, 2]) / 1
b = [1, 2] / 1 / [1, 2] // = ([1, 2] / 1) / [1, 2] 
                        // = [1] / [1, 2]
                        // type error

// belonging
a = [1...4]
2 in a   // true
"a" in a // false
-1 in a  // false

// list length
size([1...3]) // 3
size(4)       // type error
```

#### Objects

```jsx
o = {
  key: "value",
  "Some other key": 3,
  a: [3, 5],
  b: { key: "other value" },
  c: f () {
    return "hello"
  }, // optional `,` at end
}

// read, write
o.key     // "value"
o["key"]  // "value"
o.c()     // "hello"
o.key = "something else"
o.key = 4 // error

// create key
o.new = "hello"
o.new // "hello"

// delete key
delete(o.new)
o.new // runtime error

// keys list
keys(o) // ["key", "Some other key", ...]
```

### Blocks

```jsx
a = 3
{
  a = 4
  b = 2
  out(a) // 4
  out(b) // 2
}
out(a) // 4
out(b) // runtime error
```

### Loops

```jsx
a = 0
while a < 10 {
  out("something")
  a++
  break    // stops closest loop
  continue // goes to next iteration of closest loop
}

// for is for lists and strings only
b = [1...10]
for item in b {
  out(b) // 1 2 3...
}

c = "word"
for char in c {
  out(char) // w o r d
}

for char, i in c {
  out(char + " {i}") // w 0 o 1 r 2 d 3
}
```

### Conditionals

```jsx
a = 3
if a < 10 {
  out("something")
} else if a > 10 {
  out("something else")
} else {
  out("a = 10")
}

t = a == 10 ? "a = 10" : "a != 10"
u = a ? 1 : 2         // type error (a not boolean)
u = a == 10 ? "1" : 2 // type error (type mismatch)

match a {
  1 -> out("1")
  2 or 3 -> {
    out("something")
  }
  default -> out("else") // optional
}
```

### Functions

```jsx
f someFunc(a, b) {
  c = a + b
  return c
}

f inlineFunc(a, b) -> a + b

// anonymous functions
a = f () -> out("something")
a() // "something"

f () {
  doSomething()
  return "something"
}() // "something"

f func2() {
  return // syntax error
}

f func3(a) {
  if (a == 3) {
    return "a"
  } else {
    return 1 // type error (branches mismatch)
  }
}
```

### Builtin commands

```jsx
a = ask("test")   // string
out(a)
size([1, 2])
cmd("echo hello") // shell command
delete(o.key)
keys(o)
```
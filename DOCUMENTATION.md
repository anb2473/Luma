# Luma Developement Documentation

## 4 / 20 / 2025

### Progress Log

Successfully implemented `Value` enum and surrounding functionalities:

* Enum implementation with types including `Int` (`i32`), `Char` (`char`), `Str` (`String`), `Float` (`f64`), and `Undefined` (`()`). **REASONING:**

    a. `i32` range: -2,147,483,648 to 2,147,483,647, this provides a generous range for initial developement, with room for shorthand and longhand types.
    b. `char` range: all ASCII characters (`i8` only allows a limited selection), using the `char` type also allows for easy integration with the `Str` (`String`) type as the `String` type is built on the `char` type.
    c. `String`: built on `char` allowing easy integration, and alternatives such as `str` are simply built on the `String` type.
    d. `f64` range: 1.7976931348623157E+308 and -1.7976931348623157E+308, this allows almost any numerical value possible (**NOTE:** `f64` allows for a very large range, allowing for almost any functionality, however shorthand and longhand will be added in the future).
    e. `()`: `Undefined` types should not clog up memory, and as such the `()` allows for the least load in a high capacity environment.

* Casting functions:

    a. The casting functions are defined as traits of the `Value` enum (named `cast_to`, each function takes a reference to a `Value` and requires a type notation).
    b. Each cast function takes a type notation of the root type of the `Value` (i.e., `()`, `String`, etc.).

* Evaluate function (`evaluate`) implementation as a static method of the `Value` enum, the function takes in a `String` value, and generates a response of type `Value` from the provided `String`.

* Get type function (`get_type`) implementation as a static method of the `Value` enum, the function takes in a `Value` value, and generates a response of type `Value::Str` from the provided `Value` which corresponds to the name of the `Value`'s type (**REASONING:** A `Value::Str` allows for easy referencing directly from Luma functions).

### Program Documentation

enum `Value`

members: `Value::Int(i32)`, `Value::Char(char)`, `Value::Str(String)`, `Value::Float(f64)`, `Value::Undefined(())`

description: An enum storing types for all major types (provides the base of the language's type management).

__________________________________________________________________________________________________________________

trait `CastTo`

method: `cast_to<t>(&self) -> Option<Value>`

description: Takes in a reference to a `Value` with a type notation representing the direct type of the desired casting `Value` and returns a `Value` of that desired type.

__________________________________________________________________________________________________________________

function `evaluate(String) -> Value`

description: Takes in a `String` representation of a `Value` under the syntax guidlines for a `Value`, and returns the associated `Value`.

__________________________________________________________________________________________________________________

function `get_type(Value) -> String`

description: Takes in a `Value` object and returns a `Value::Str` representation of the `Value`'s type.

### Test Coverage

* Tested the `evaluate` function of the `Value` enum with all major types.

## 5 / 20 / 2025

### Test Coverage

* Tested the `cast_to` function of the `Value` enum with all major types.

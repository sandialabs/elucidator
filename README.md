# Elucidator

This document is an explanation of the Elucidation Metatadata Standard and our own implementation of the standard as a set of libraries and tools.
This Elucidation Metadata Standard provides a way to create implementation-independent metadata standards using text, so that you can create arbitrary byte-based metadata and store it using technologies like databases, without having to resort to creating your own libraries or locking you in to a particular library or knowing schemas in advance.
Elucidator itself is a set of libraries and tools intended to provide real-world use of this standard.
Other implementors are free to use this standard to implement other tools and libraries as needed.

## The Elucidation Metadata Standard

This standard specifies how to describe byte-based metadata using text, and interpret it on reading.
From here on

### Nomenclature

From here on, "The Standard" is an abbreviation of "The Elucidation Metadata Standard".
- `Metadata` is a collection of bytes that can be interpreted based on some `Specification` (singular `Metadatum`).
- `Group` is a set of `Metadata Specification`s, often particular to a project or domain.
- `Specification` is the association of some `Identifier` with some set of rules for interpretation.
- `Identifier` is the string which is associated with some set of rules about how something should be interpreted.
  Identifiers must be ASCII-encoded alphanumerical or underscore characters, beginning with an alphabetical character.
- `Interpreter` is a routine which can convert an individual `Metadatum` into the correct associated types.
- `Metadata Specification` is the `Specification` of a Metadata `Designation` and its associated, ordered `Member`s.
- `Designation` is the `Identifier` associated with a particular class of `Metadata`.
- `Member` is a component of an individual `Metadatum`, which has an associated `Identifier`, `Data Type`, and `Value`.
   For a member, the identifier must be unique to an individual `Metadata Specification`, but need not be unique to across all `Metadata Specification`s.
- `Data Type` indicates how a particular `Value` should be extracted from a collection of bytes.
   The Standard specifies a discrete set of possible types.
   This is often abbreviated as `Dtype`.
- `Value` is the contents of some `Member` interpreted through its `Data Type`.
- `Member Specification` defines the association of some `Identifier` with a `Data Type` for a particular `Member`.
- `Array` is an ordered set of values with homogeneous `Data Type`.

### The Standard

#### Metadata Specification

Metadata Specification consists of a `Designation` and an ordered set of `Members`.
The `Designation` should be unique for a given set of related `Metadata`.
Implementors are allowed to make any link between a `Designation and the ordered set of `Member`s that they please; for example, using columns in a SQL database, one for `Designation` and one for `Metadata Specification` which contains a textual representation of the `Member`s.
In the absence of an implementation-defined linkage, the following grammar should be used to indicate the mapping of designation to ordered member sets:

```
specification: Designation(member, member, member, ...)(context);
```
with `Designation` the designation for this specification, `context` an optional string with additional descriptive information, and `member`s specified by the grammar
```
member: Identifier: Dtype
```
.
Compliant implementations may NOT use a `context` to perform any processing; this field is intended for human readability and information only, much like comments in source code.
Whitespace is ignored except for the `context` string, as Identifiers and Dtypes are not allowed to contain them.

#### Data Types
The following table indicates all allowable data types.
Compliant implementations must implement all data types.


| Name                          | String Representation |
|-------------------------------|-----------------------|
| Byte                          | u8                    |
| Unsigned 16-bit integer       | u16                   |
| Unsigned 32-bit integer       | u32                   |
| Unsigned 64-bit integer       | u64                   |
| Signed 8-bit integer          | i8                    |
| Signed 16-bit integer         | i16                   |
| Signed 32-bit integer         | i32                   |
| Signed 64-bit integer         | i64                   |
| IEEE 32-bit floating point    | f32                   |
| IEEE 64-bit floating point    | f64                   |
| String                        | string                |

#### Arrays

All Data Types which are not `String` may be constructed as an `Array`.
An `Array` may be of fixed size in the `Member Specification`, or of dynamic size.

NOTE: signed integers used for dynamic sizing are NOT compliant with The Standard.

Arrays are specified using the following grammar for fixed size:
```
Dtype[literal]
```
and the following grammar for dynamic size:
```
Dtype[]
```

#### Byte Representation

For all types, little endian byte ordering is required.
The `String` type consists of one unsigned 64-bit integer, followed by that number of bytes to represent the string.
NOTE: The `String` type is NOT nul-terminated.
For fixed arrays, the underlying data type is repeated for the size of the array with no padding.
For dynamic arrays, like `String`s, the array begins with one unsigned 64-bit integer, followed by that number of elements of the designated type in byte representation.

## Elucidator

Elucidator contains the following components:
- [ ] A rust-based library which implements manipulations of metadata based on The Standard
- [ ] A rust-based library which adds database storage of metadata with spatiotemporal bounding boxes associated with each metadatum
- [ ] Python bindings for the rust libraries
- [ ] C bindings for the rust libraries
- [ ] A small set of utility tools

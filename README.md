## How this works?

This database uses file memory mapping to manage data.
One file per one data structure is used.
List of existing data structures:
- Array: contiguously stored fixed sized items.
This is the simplest structure here. Some operations included: `add new item`, `get item by index`, `remove last`, `swap remove`.
- Dynamic array: stores items with dynamic size (Such as String, Vec).
- Indexed dynamic array: same as dynamic array but also stores a layer of IDs to items. This means location of items can be moved without changing their IDs.
- Binary tree: represents a binary search tree map. Keys and values are of a fixed size.
This is, for example, used for indexing fields in a database for efficient (exact) search.

## Macros
The library has two major proc macros: `bindb::fixed!` for entry types of constant (fixed) size.

And `bindb::dynamic!` for entry types of dynamic size, such as String or Vec.

## Margination / Capacity
Whenever a file representing a data structure reaches its capacity (file size) it'll be extended to a new capacity and re-memory mapped. This is a potentially slow operation.

## Contributing
The library is not yet well documented so it'd be hard to understand it (and difficult to make contributions). (I'm working on documenting it)
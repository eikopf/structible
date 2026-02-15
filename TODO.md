# TODOs
1. Extend `BackingMap` with a `with_capacity` method and use it to optimise generated constructors.
1. Make the generated `*Fields` type return the unknown fields by collecting them in an instance of the backing map.
1. Decide what happens to the visibility of fields in the generated `*Fields` type.
1. Determine whether it's possible to avoid using the `full` feature of `syn`, since it's quite a heavy dependency.
1. Add more tests for macro generation edge cases.
1. Update the documentation to be more terse, and to focus on giving examples.
1. Prepare the crates in the workspace for publication on crates.io.

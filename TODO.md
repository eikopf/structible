# TODOs
1. Fix the failing doctests in the README.
1. Add extra tests for more pathological cases. Test for complex generics involving lifetimes, const parameters, and where bounds.
1. Stop hard-coding the possible backing map types, and instead rely on the BackingMap trait.
1. Determine whether it's possible to avoid using the `full` feature of `syn`, since it's quite a heavy dependency.
1. Add more tests for macro generation edge cases.
1. Update the documentation to be more terse, and to focus on giving examples.
1. Prepare the crates in the workspace for publication on crates.io.

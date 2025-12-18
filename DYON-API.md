# Dyon API

At the top level of the project, there is usually a "DYON-API.md" file.

This "DYON-API.md" file is used to explain the API for the project.

Here you will find links and resources to learn how a project works.

### About Dyon library API

Dyon API is usually put under the "src" folder in a project.

For example, in this project, there is a `src/lib.dyon` file:

- [Link to "lib.dyon"](https://github.com/PistonDevelopers/dyon/blob/master/src/lib.dyon)

These links should be to the upstream repository, so people can easily
find out the latest version and compare it with their local copy.

Doc comments use `///` and external functions have no function body `{ ... }`.

For example:

```dyon no_run
/// Prints out variable to standard output, adding newline character.
fn println(var: any) { ... }
```

These files are often used for an overview, so try keep them short.
However, remember to put a blank line between functions for readability.

By convention, it is a good practice to keep doc comments short.
If you need extensive documentation, then link to more information.

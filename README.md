# mdbook-transcheck
Checker for translated mdbook

# Install

```console
$ cargo install mdbook-transcheck
```

# Usage

```console
$ mdbook-transcheck src tgt
```

`src` is the source directory of original mdbook.
`tgt` is the source directory of translated mdbook.

# Example

```console
$ mdbook-transcheck ./testcase/original ./testcase/translated

Error: target path is not found
    source path: testcase/original/missing_file.md
    target path: testcase/translated/missing_file.md


Error: source line is missing
 source --> testcase/original/missing_lines.md:2
  |
2 | Orange
  |


Error: target line is modifies
 source --> testcase/original/missing_lines.md:5
  |
5 | This is an orange.
  |            ^^ ^^
  |

 target --> testcase/translated/missing_lines.md:9
  |
9 | This is an apple.
  |             ^^^
  |
```

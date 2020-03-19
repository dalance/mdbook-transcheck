# mdbook-transcheck
Checker for translated mdbook

[![Actions Status](https://github.com/dalance/mdbook-transcheck/workflows/Regression/badge.svg)](https://github.com/dalance/mdbook-transcheck/actions)
[![Crates.io](https://img.shields.io/crates/v/mdbook-transcheck.svg)](https://crates.io/crates/mdbook-transcheck)

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

# Configuration

The configuration file is `transcheck.toml`, which is put at the repository root.

```toml
enable_code_comment_tweak = true
code_comment_header = "# "
```

| Key                       | Value       | Default | Description                                                                                                                     |
| ------------------------- | ----------- | ------- | ------------------------------------------------------------------------------------------------------------------------------- |
| enable_code_comment_tweak | true, false | false   | Match code comment without `code_comment_header`                                                                                |
| code_comment_header       | [String]    | `"# "`  |                                                                                                                                 |
| similar_threshold         | [Float]     | 0.5     | If the ratio which the original and translated lines are matched exceeds `similar_threshold`, the line is judged as *modified*. |

# Example

```console
$ mdbook-transcheck ./testcase/original ./testcase/translated

Error: target path is not found
    source path: testcase/original/missing_file.md
    target path: testcase/translated/missing_file.md


Error: source line has been modified
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


Error: lines has been inserted to the source file
 source --> testcase/original/missing_lines.md:2
  |
2 | Orange
  |
  = hint: The lines should be inserted at testcase/translated/missing_lines.md:2
```

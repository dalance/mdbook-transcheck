# mdbook-transcheck
Checker for translated mdbook

[![Actions Status](https://github.com/dalance/mdbook-transcheck/workflows/Regression/badge.svg)](https://github.com/dalance/mdbook-transcheck/actions)
[![Crates.io](https://img.shields.io/crates/v/mdbook-transcheck.svg)](https://crates.io/crates/mdbook-transcheck)

# Install

```console
$ cargo install mdbook-transcheck
```

# Usage

## Check

The following command checks whether `src` and `tgt` are synchronized.

```console
$ mdbook-transcheck src tgt
```

`src` is the source directory of original mdbook.
`tgt` is the source directory of translated mdbook.

## Fix

The following command applies the differences between `src` and `tgt` to `tgt`.

```console
$ mdbook-transcheck --fix src tgt
```

## Lint

The following command checks whether translated texts satisfy lint rules.

```console
$ mdbook-transcheck --lint src tgt
```

# Configuration

The configuration file is `transcheck.toml`, which is put at the repository root.

```toml
[matcher]
enable_code_comment_tweak = true
code_comment_header = "# "
[linter]
enable_emphasis_check = true
enable_half_paren_check = true
enable_full_paren_check = true
```

## `[matcher]` section

| Key                       | Value       | Default | Description                                                                                                                     |
| ------------------------- | ----------- | ------- | ------------------------------------------------------------------------------------------------------------------------------- |
| enable_code_comment_tweak | true, false | false   | Match code comment without `code_comment_header`                                                                                |
| code_comment_header       | String      | `"# "`  |                                                                                                                                 |
| similar_threshold         | Float       | 0.5     | If the ratio which the original and translated lines are matched exceeds `similar_threshold`, the line is judged as *modified*. |

## `[linter]` section

| Key                     | Value       | Default | Description                                                             |
| ----------------------- | ----------- | ------- | ----------------------------------------------------------------------- |
| enable_emphasis_check   | true, false | false   | Check wether emphasis (`*..*`/`**..**`) has spaces before and after it. |
| enable_half_paren_check | true, false | false   | Check wether half-width paren (`()`) has ascii charactors only.         |
| enable_full_paren_check | true, false | false   | Check wether full-width paren (`（）`) has non-ascii charactors.        |

# Example

```console
$ mdbook-transcheck ./testcase/original ./testcase/translated

Error: target path is not found
    source path: ./testcase/original/missing_file.md
    target path: ./testcase/translated/missing_file.md


Error: source line has been modified
 source --> ./testcase/original/mismatch_lines.md:5
  |
5 | This is an orange.
  |            ^^ ^^
  |

 target --> ./testcase/translated/mismatch_lines.md:11
   |
11 | This is an apple.
   |             ^^^
   |


Error: lines has been inserted to the source file
 source --> ./testcase/original/mismatch_lines.md:2
  |
2 | Orange
  |
  = hint: The lines should be inserted at ./testcase/translated/mismatch_lines.md:2


Error: lines has been removed from the source file
 target --> ./testcase/translated/mismatch_lines.md:4
  |
4 | Lemon
  |
```

# Markdown rule

The translated markdown should follow some rules.

* Keep original lines
* Comment out original lines by `<!--` and `-->`

## Simple example

* original

```markdown
Apple
Orange
Peach
```

* translated

```markdown
<!--
Apple
Orange
Peach
-->
りんご
オレンジ
桃
```

The following is NG because `<!-- Apple` and `Peach -->` are not matched with original lines.

```markdown
<!-- Apple
Orange
Peach -->
りんご
オレンジ
桃
```

## Code block

* original

````markdown
```rust
// comment
let a = b; // comment
```
````

* translated

````markdown
```rust
// comment
// コメント
let a = b; // comment
           // コメント
```
````

You can use `# ` to hide the original comment.
`enable_code_comment_tweak` should be `true`, and `code_comment_header` should be `# `.

````markdown
```rust
# // comment
// コメント
let a = b; // comment
           // コメント
```
````

You can use `# // ` to hide the original code and comment.
`enable_code_comment_tweak` should be `true`, and `code_comment_header` should be `# // `.

````markdown
```rust
# // // comment
// コメント
# // let a = b; // comment
let a = b; // コメント
```
````

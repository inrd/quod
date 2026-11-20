# Grammar draft — phase-1 core

> **Status: provisional, non-normative.** Nothing here is accepted. It becomes `spec/grammar.md` only via an ADR at the end of phase 1.

## Goals

- **LL(1)** after lexing: every alternative is chosen by one token of lookahead. This keeps the parser trivial and makes the grammar directly usable for **grammar-constrained decoding**.
- **Small**: phase 1 covers integers, `bool`, structs, fixed arrays, slices, functions, bounded loops.
- **No ambiguity by construction**: no overloading, no implicit conversions, no textual macros, no operator precedence surprises (see §6).
- **One canonical format**: whitespace and comments are not significant to the AST; the formatter defines layout.

Function contracts, `while` loops and spec expressions (`result`, `old`, quantifiers, `==>`) are added for the thin slice ([contracts.md](contracts.md), [ADR-0006](../docs/adr/0006-thin-end-to-end-slice.md) S2). Other phase-2+ constructs are included under **Reserved** so the parser can reject them with a clear diagnostic instead of a syntax error.

## Notation

ISO-style EBNF: `{ x }` zero or more, `[ x ]` optional, `( a | b )` grouping, `"x"` terminal. `UPPER` names are lexical tokens.

## 1. Lexical

```ebnf
IDENT      = letter { letter | digit | "_" } ;          (* not a keyword *)
letter     = "a".."z" | "A".."Z" | "_" ;
INT_LIT    = dec_lit | hex_lit | bin_lit ;
dec_lit    = digit { digit | "_" } [ int_suffix ] ;
hex_lit    = "0x" hexdigit { hexdigit | "_" } [ int_suffix ] ;
bin_lit    = "0b" ( "0" | "1" ) { "0" | "1" | "_" } [ int_suffix ] ;
int_suffix = "u8" | "u16" | "u32" | "u64" | "usize"
           | "i8" | "i16" | "i32" | "i64" | "isize" ;
HOLE       = "?" IDENT ;                                  (* typed hole, e.g. ?h1 *)
COMMENT    = "//" { any_char_except_newline } ;           (* line comments only *)
```

**Keywords (phase 1):** `fn type const let mut if else for in while return true false as effects pub mod use break continue`.

**Keywords (slice, [contracts.md](contracts.md)):** `requires ensures invariant decreases old result forall exists`.

**Reserved (phase 2+):** `assert where spec impl refines trust assume ghost proof lemma cost stack noalloc mem provenance`.

`==>` is one token (longest match, so `==` followed by `>` never lexes as two tokens).

No block comments (they nest badly and confuse constrained decoders). No string literals in phase 1.

## 2. Items

```ebnf
file        = { item } EOF ;
item        = [ "pub" ] ( fn_item | type_item | mod_item | use_item ) ;

mod_item    = "mod" IDENT ";" ;
use_item    = "use" path ";" ;
path        = IDENT { "::" IDENT } ;

type_item   = "type" IDENT [ generic_params ] "{" field { "," field } [ "," ] "}" ;
field       = IDENT ":" type [ where_clause ] ;           (* where_clause: phase 2 *)

generic_params = "<" generic_param { "," generic_param } ">" ;
generic_param  = IDENT | "const" IDENT ":" type ;

fn_item     = "fn" IDENT [ generic_params ]
              "(" [ param { "," param } ] ")"
              "->" type
              effects_clause
              { "requires" expr }                          (* contracts.md §3 *)
              { "ensures" expr }
              block ;
param       = IDENT ":" type ;
effects_clause = "effects" "{" [ IDENT { "," IDENT } ] "}" ;
```

Notes:
- The return type is **mandatory** (`-> ()` for unit). Nothing implicit.
- `effects { }` is **mandatory** on every function even in phase 1, where only the empty set is accepted. This keeps phase-1 code forward-compatible and makes the "pure" claim explicit.

## 3. Types

```ebnf
type        = path [ generic_args ]                       (* named, incl. u32, bool *)
            | "[" type ";" const_expr "]"                 (* fixed array *)
            | "&" [ "mut" ] slice_or_type
            | "(" ")" ;                                   (* unit *)
slice_or_type = "[" type "]" | type ;
generic_args = "<" type_or_const { "," type_or_const } ">" ;
type_or_const = type | INT_LIT ;                          (* refine in phase 2 *)
const_expr  = INT_LIT | IDENT ;
```

Built-in scalar types: `u8 u16 u32 u64 usize i8 i16 i32 i64 isize bool`. Spec-only: `int` (quantifier binders only, [contracts.md §5](contracts.md#5-spec-contexts)), `nat` (phase 2). No floats in phase 1 (their semantics need a decision; the README's `f32` example is phase-2+).

`&[T; N]` vs `&[T]`: after `&` `[`, the parser reads `type` then branches on `;` (array) or `]` (slice). This is a one-token decision after a shared prefix, factored as:

```ebnf
bracket_type = "[" type ( ";" const_expr "]" | "]" ) ;
```

## 4. Statements

```ebnf
block       = "{" { stmt } "}" ;
stmt        = let_stmt | if_stmt | for_stmt | while_stmt | return_stmt
            | break_stmt | continue_stmt | block | expr_stmt ;

let_stmt    = "let" [ "mut" ] IDENT ":" type "=" expr ";" ;   (* type annotation mandatory *)
if_stmt     = "if" expr block [ "else" ( if_stmt | block ) ] ;
for_stmt    = "for" IDENT ":" type "in" expr ".." expr block ;
while_stmt  = "while" expr { "invariant" expr }            (* contracts.md §4 *)
              "decreases" expr block ;
return_stmt = "return" expr ";" ;
break_stmt  = "break" ";" ;
continue_stmt = "continue" ";" ;
expr_stmt   = expr ( "=" expr | compound_op expr | ) ";" ;
compound_op = "+=" | "-=" | "*=" ;
```

Notes:
- **Type annotations on `let` are mandatory** (provisional; redundancy as a consistency check). Open: allow omission when the initializer is a literal of the same type?
- **Bounded loops:** `for i: u32 in a..b` iterates a half-open range evaluated once. The bound is fixed at loop entry; the body cannot mutate `i`. `for` takes no clauses yet; a loop that needs an invariant is written as `while`.
- **`while`** has zero or more `invariant` clauses and exactly one `decreases`, in that order ([contracts.md §4](contracts.md#4-while-loops)). The `decreases` clause is mandatory, so a `while` header always ends in a clause before its block.
- Assignment is a **statement**, not an expression. `expr_stmt` parses an expression, then one token decides between `=`, a compound operator, or `;`. The checker restricts the left side to places (variables, fields, indexed elements).
- No `++`/`--`. No `/=` or `%=` in phase 1 (they carry preconditions; keep them visible).
- `if` conditions must be `bool`; no truthiness.

## 5. Expressions

Precedence is **shallow and explicit**: mixing operators from different groups without parentheses is a syntax error (see §6).

```ebnf
expr        = or_expr [ "==>" or_expr ] ;                 (* ==> : spec contexts only; not associative *)
or_expr     = and_expr { "||" and_expr } ;
and_expr    = cmp_expr { "&&" cmp_expr } ;
cmp_expr    = arith_expr [ cmp_op arith_expr ] ;          (* non-associative *)
cmp_op      = "==" | "!=" | "<" | "<=" | ">" | ">=" ;
arith_expr  = unary { arith_op unary } ;                  (* single operator kind per chain, see §6 *)
arith_op    = "+" | "-" | "*" | "/" | "%" | "&" | "|" | "^" | "<<" | ">>" ;
unary       = ( "-" | "!" ) unary
            | "&" [ "mut" ] unary                         (* reference argument, typing.md §8 *)
            | cast_expr ;
cast_expr   = postfix [ "as" IDENT ] ;                    (* target: scalar type name *)
postfix     = primary { "." IDENT | index | "(" [ args ] ")" } ;
index       = "[" expr ( "]" | ".." expr "]" ) ;            (* e[i] or e[lo..hi] *)
args        = expr { "," expr } ;
primary     = INT_LIT | "true" | "false" | HOLE
            | "result"                                    (* spec: contracts.md §5.3 *)
            | "old" "(" expr ")"
            | quant
            | expr_path [ struct_lit ]
            | "(" ( ")" | expr ")" )                       (* unit value or parenthesized *)
            | "[" [ args ] "]" ;                          (* array literal *)
expr_path   = IDENT path_tail ;
path_tail   = "::" ( IDENT path_tail | generic_args ) | ;   (* id::<u32> ends the path *)
struct_lit  = DOT_LBRACE field_init { "," field_init } [ "," ] "}" ;
field_init  = IDENT ":" expr ;
quant       = ( "forall" | "exists" ) IDENT ":" type      (* spec: contracts.md §5.4 *)
              "in" expr ".." expr "{" expr "}" ;
```

`DOT_LBRACE` is the single token `.{` (no space allowed between `.` and `{`). This is the preferred fix from §8 and is what the phase-1 parser implements.

Notes:
- **Cast targets are a scalar type name** (`x as u32`), not a full `type`. With `as type`, `x as T < y` would need more than one token to decide between generic arguments and a comparison. Casts only apply to integer types, so nothing is lost.
- **References and slices** ([ADR-0004](../docs/adr/0004-phase-1-typing-choices.md), [typing.md §8](typing.md#8-references-and-slices-phase-1-subset)): `&p`, `&mut p` and `&p[lo..hi]`. The grammar accepts them anywhere; the type checker restricts them to call arguments. `&&x` is a syntax error (the lexer produces `&&`); write `&(&x)` if ever needed.
- **Unit value** `()` is a primary, so `-> ()` functions can `return ();`.
- **`>>` and `>=` inside generic arguments** are split by the parser when it expects `>` (`Ring<Box<u8>>`). This is a lexer/parser interface detail, not an LL(1) conflict.
- **Struct literals** use `Name.{ f: e }` (provisional). The leading `.` avoids the Rust/C ambiguity between `if x { … }` and `if Point { … }` while staying LL(1).
- **Generic calls** need explicit arguments in phase 1: `id::<u32>(x)` (provisional). `::<` keeps `<` unambiguous with comparison.
- `as` casts carry a **range precondition** (no silent truncation). Truncation, wrapping and saturation are named functions, not operators.
- Integer literals without suffix take their type from the expected type (bidirectional checking); if none, a suffix is required in code. In spec contexts, unsuffixed literals have type `int` (§5a), except as bitwise operands ([contracts.md §5.1](contracts.md#51-types-and-literals)).
- **`==>`** binds weakest: `a && b ==> c` is `(a && b) ==> c`. It is not a chain operator; `a ==> b ==> c` is a syntax error. The grammar accepts it anywhere; the type checker rejects it in code (`E0326`).
- **Quantifiers** are primaries delimited by braces, so they need no precedence rule: `forall k: int in 0..n { a[k] == 0 }`. The range uses the same `lo..hi` form as `for`.

## 5a. Arithmetic semantics (accepted: [ADR-0002](../docs/adr/0002-arithmetic-semantics.md))

The syntax above is provisional; the semantics in this section are accepted.

| Operator | Code context | Spec context |
|---|---|---|
| `+ - *`, unary `-` | precondition: exact result fits in `T` | operands lifted to `int`; exact; no precondition |
| `/ %` | precondition: `b != 0`, and for signed `!(a == MIN_T && b == -1)`; truncate toward zero | on `int`; truncate toward zero; `b != 0` is a spec well-formedness obligation |
| `<< n` | precondition: `n < bits(T)` and no bits lost | bit-vector meaning, same machine type both sides |
| `>> n` | precondition: `n < bits(T)`; arithmetic for signed | bit-vector meaning |
| `& \| ^`, unary `!` on integers | total | bit-vector meaning; `int(e)` converts explicitly |
| `e as U` | precondition: value fits in `U` (widening is total) | `x as int` allowed; `int → machine` carries a range obligation |

- **Spec contexts** are expressions under `requires`, `ensures`, `invariant`, `decreases`, `where`, `assert`, `assume`, `spec fn` bodies, `lemma`s and ghost code. Determined by the enclosing keyword only. Typing details for the slice: [contracts.md §5](contracts.md#5-spec-contexts).
- **Spec-only types:** `int`, `nat`. Using them in executable code is a type error.
- **Named operations** (code): `add_wrap`, `add_sat`, `add_checked` (→ `Option<T>`), same for `sub`, `mul`, `shl`; `trunc::<U>(e)`.
- **Phase 1:** preconditions that are not statically trivial compile to runtime traps and are listed in the "unverified checks" report. From phase 2 they are proof obligations.

## 6. Operator mixing rule

To remove precedence as a source of agent (and human) error:

- `a + b * c` → **syntax error**; write `a + (b * c)`.
- `a + b + c` → allowed (same operator, left-associative).
- `a + b - c` → **error** (different operators) — provisional; may relax for `+`/`-` only.
- `a < b < c` → error (comparison is non-associative).
- `&&` and `||` cannot be mixed without parentheses.

The grammar above accepts the chain syntactically; a parser-level check (still single-pass, LL(1)) rejects mixed chains. Alternative: encode each operator as its own production (larger grammar, same language).

## 7. Reserved phase-2 clauses (illustrative)

```ebnf
where_clause    = "where" expr ;
fn_decreases    = "decreases" expr ;                      (* recursion; after ensures *)
```

## 8. LL(1) audit checklist

To verify mechanically in phase 1 (e.g. with a grammar tool):

- [ ] No left recursion (all repetitions use `{ }`).
- [ ] FIRST sets of `stmt` alternatives disjoint (`let`, `if`, `for`, `while`, `return`, `break`, `continue`, `{`, else expression starters). Note: `block` and array literal both start with `{`/`[` — `{` only starts a block in statement position; array literals start with `[`. OK.
- [ ] `path [ struct_lit ]`: decided by `.` followed by `{` — **needs 2 tokens** (`.` alone is field access in `postfix`). Fix options: lex `.{` as a single token (preferred), or use a keyword prefix. The phase-1 parser lexes `.{` as `DOT_LBRACE` (§5); still provisional.
- [ ] `cast_expr`: `as type` followed by `<` is ambiguous (generic args vs comparison). Restricted to `as IDENT` (§5).
- [ ] `expr_stmt` tail decided by one token.
- [ ] `bracket_type` factored.
- [ ] `index`: after `[ expr`, one token decides `]` or `..`.
- [ ] `fn_item` clauses: after `effects { … }`, one token decides `requires`, `ensures` or `{`; after an `ensures`, `requires` is not in FIRST, so the order is fixed by the grammar.
- [ ] `while_stmt`: the condition ends at `invariant` or `decreases` (neither can continue an expression); the block follows the `decreases` expression.
- [ ] `quant`: the range ends at `{`, which cannot continue an expression (struct literals use `DOT_LBRACE`).
- [ ] `expr`: after `or_expr`, one token decides `==>`.

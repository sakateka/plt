Theory of Programming Languages and Translation Methods
=======================================================
- Labs
- Course works

Build for Windows
=================
Instruction from https://github.com/japaric/rust-cross

```
Step 0: Our target triple is `x86_64-pc-windows-gnu`

Step 1: Install the C cross toolchain
$ sudo apt install gcc-mingw-w64-x86-64

Step 2: Install the cross compiled standard crates
$ rustup target add x86_64-pc-windows-gnu

Step 3: Configure cargo for cross compilation
$ mkdir -p ~/.cargo
$ cat >> ~/.cargo/config <<EOF
> [target.x86_64-pc-windows-gnu]
> linker = "x86_64-w64-mingw32-gcc"
> EOF

Step 4: Build
$ cargo build --release --target "x86_64-pc-windows-gnu"

Step 5: Check result
$ file target/x86_64-pc-windows-gnu/release/plt.exe
target/x86_64-pc-windows-gnu/release/plt.exe: PE32+ executable (console) x86-64, for MS Windows
```


### Generation Example

```
cargo run -- gen sample/cfg/gen.cfg --len_max 4
...
bbbc
bbba
bbb
babb
cbbb
bbbb
abbb
bcbb
```

### DFA example

```
cargo run dfa sample/dfa/dfa.txt -p
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/plt dfa sample/dfa/dfa.txt -p`
a
->N->B
a - OK
aba
->N->B->C->B
aba - OK
ababa
->N->B->C->B->C->B
ababa - OK
abba
->N->B->C->N->B
abba - OK
abbaab
->N->B->C->N->B->B->C
abbaab - EOL but DFA state 'C' at row:1 is not accepting
```


### Grammar simplification

```rust
pub fn simplify(&self) -> CFG {
    self.remove_epsilon_rules()
        .remove_unit_rules()
        .remove_useless_rules()
        .remove_unreachable_rules()
}
```

```
cat test.cfg
<e> -> <X><X>
<X> -> <X><X>
<X> -> <X>(<X>) | <X>(<X>,<X>) | <X>(<X>,<X>,<X>) |

cargo run -- simplify test.cfg
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/plt simplify test.cfg`
<e> ->  | () | (,) | (,,) | (,,X) | (,X) | (,X,) | (,X,X) | (X) | (X,) | (X,,) | (X,,X) | (X,X) | (X,X,) | (X,X,X) | X() | X(,) | X(,,) | X(,,X) | X(,X) | X(,X,) | X(,X,X) | X(X) | X(X,) | X(X,,) | X(X,,X) | X(X,X) | X(X,X,) | X(X,X,X) | XX
X -> () | (,) | (,,) | (,,X) | (,X) | (,X,) | (,X,X) | (X) | (X,) | (X,,) | (X,,X) | (X,X) | (X,X,) | (X,X,X) | X() | X(,) | X(,,) | X(,,X) | X(,X) | X(,X,) | X(,X,X) | X(X) | X(X,) | X(X,,) | X(X,,X) | X(X,X) | X(X,X,) | X(X,X,X) | XX

```

```rust
let cfg = self
    .remove_start_from_rhs()
    .remove_epsilon_rules()
    .remove_unit_rules()
    .remove_useless_rules()
    .remove_unreachable_rules();
// and then
// Eliminate all rules having more than two symbols on the right-hand side.
...
// and then
// Eliminate all rules of the form A →  u₁u₂,
// where u₁ and u₂ are not both variables.
...
```

```
cargo run -- simplify --chomsky test.cfg
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/plt simplify --chomsky test.cfg`
<e> ->  | <(><)> | <(><,)> | <(><,,)> | <(><,,X)> | <(><,X)> | <(><,X,)> | <(><,X,X)> | <(><X)> | <(><X,)> | <(><X,,)> | <(><X,,X)> | <(><X,X)> | <(><X,X,)> | <(><X,X,X)> | X<()> | X<(,)> | X<(,,)> | X<(,,X)> | X<(,X)> | X<(,X,)> | X<(,X,X)> | X<(X)> | X<(X,)> | X<(X,,)> | X<(X,,X)> | X<(X,X)> | X<(X,X,)> | X<(X,X,X)> | XX
<(> -> (
<()> -> <(><)>
<(,)> -> <(><,)>
<(,,)> -> <(><,,)>
<(,,X)> -> <(><,,X)>
<(,X)> -> <(><,X)>
<(,X,)> -> <(><,X,)>
<(,X,X)> -> <(><,X,X)>
<(X)> -> <(><X)>
<(X,)> -> <(><X,)>
<(X,,)> -> <(><X,,)>
<(X,,X)> -> <(><X,,X)>
<(X,X)> -> <(><X,X)>
<(X,X,)> -> <(><X,X,)>
<(X,X,X)> -> <(><X,X,X)>
<)> -> )
<,> -> ,
<,)> -> <,><)>
<,,)> -> <,><,)>
<,,X)> -> <,><,X)>
<,X)> -> <,><X)>
<,X,)> -> <,><X,)>
<,X,X)> -> <,><X,X)>
X -> <(><)> | <(><,)> | <(><,,)> | <(><,,X)> | <(><,X)> | <(><,X,)> | <(><,X,X)> | <(><X)> | <(><X,)> | <(><X,,)> | <(><X,,X)> | <(><X,X)> | <(><X,X,)> | <(><X,X,X)> | X<()> | X<(,)> | X<(,,)> | X<(,,X)> | X<(,X)> | X<(,X,)> | X<(,X,X)> | X<(X)> | X<(X,)> | X<(X,,)> | X<(X,,X)> | X<(X,X)> | X<(X,X,)> | X<(X,X,X)> | XX
<X)> -> X<)>
<X,)> -> X<,)>
<X,,)> -> X<,,)>
<X,,X)> -> X<,,X)>
<X,X)> -> X<,X)>
<X,X,)> -> X<,X,)>
<X,X,X)> -> X<,X,X)>
```



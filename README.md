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

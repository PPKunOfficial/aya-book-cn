# 跨平台编译基于Aya的程序

以下说明展示了如何在Mac上跨平台编译Aya eBPF程序。
在其他系统上进行跨平台编译也是可行的，我们将很快添加相应的说明（欢迎提交PR！）。

# 在Mac上跨平台编译基于Aya的程序

跨平台编译在Intel和Apple Silicon Mac上都应该可以工作。

1. 根据<https://rustup.rs/>上的说明安装`rustup`。
2. 安装稳定版和夜间版的Rust工具链：
```bash
rustup install stable
rustup toolchain install nightly --component rust-src
```
3. 为您的Linux目标平台安装[rustup target](https://doc.rust-lang.org/nightly/rustc/platform-support.html#tier-1-with-host-tools)：
```bash
ARCH=x86_64
rustup target add ${ARCH}-unknown-linux-musl
```
4. 使用brew安装LLVM：
```bash
brew install llvm
```
5. 安装musl交叉编译器：  
   仅为`x86_64`目标进行跨编译（musl-cross中的默认设置）：
```bash
brew install FiloSottile/musl-cross/musl-cross
```
   仅为`aarch64`目标进行跨编译：
```bash
brew install FiloSottile/musl-cross/musl-cross --without-x86_64 --with-aarch64
```
   为`x86_64`和`aarch64`目标进行跨编译：
```bash
brew install FiloSottile/musl-cross/musl-cross --with-aarch64
```
   有关其他平台特定选项，请参见[homebrew-musl-cross](https://github.com/FiloSottile/homebrew-musl-cross)。

6. 安装bpf-linker。将`LLVM_SYS_<version>_PREFIX`中的版本号更改为对应于[llvm-sys](https://crates.io/crates/llvm-sys) crate的主版本号：

```bash
LLVM_SYS_180_PREFIX=$(brew --prefix llvm) cargo install bpf-linker --no-default-features
```
7. 构建BPF对象文件：
```bash
cargo xtask build-ebpf --release
```
8. 构建用户空间代码：
```bash
RUSTFLAGS="-Clinker=${ARCH}-linux-musl-ld" cargo build --release --target=${ARCH}-unknown-linux-musl
```
跨编译的程序  
`target/${ARCH}-unknown-linux-musl/release/<program_name>`  
可以复制到Linux服务器或虚拟机上运行。
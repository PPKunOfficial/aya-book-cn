# 开发环境

## 先决条件

在开始之前，您需要在系统上安装Rust的稳定版和夜间版工具链。这可以通过[`rustup`](https://rustup.rs)轻松实现：

```console
rustup install stable
rustup toolchain install nightly --component rust-src
```

安装完Rust工具链后，还需要安装`bpf-linker`。该链接器依赖于LLVM，如果您在**Linux x86_64系统**上运行，可以使用以下命令构建：

```console
cargo install bpf-linker
```

如果您使用的是**macOS或其他架构的Linux**，则需要先安装最新的稳定版LLVM（例如，通过`brew install llvm`），然后使用以下命令安装链接器：

```console
LLVM_SYS_180_PREFIX=$(brew --prefix llvm) cargo install --no-default-features bpf-linker
```

要为您的项目生成脚手架，您需要安装`cargo-generate`，可以通过以下命令安装：

```console
cargo install cargo-generate
```

最后，为生成内核数据结构的绑定，您必须安装`bpftool`，可以从您的发行版获取，或者从[源代码](https://github.com/libbpf/bpftool)构建。

!!! bug "在Ubuntu 20.04 LTS (Focal)上运行？"

    如果您在Ubuntu 20.04上运行，bpftool和发行版默认安装的内核存在一个bug。为了避免遇到这个问题，您可以安装不包含该bug的更新版本bpftool：

    ```console
    sudo apt install linux-tools-5.8.0-63-generic
    export PATH=/usr/lib/linux-tools/5.8.0-63-generic:$PATH
    ```

## 开始一个新项目

要开始一个新项目，可以使用`cargo-generate`：

```console
cargo generate https://github.com/aya-rs/aya-template
```

这将提示您输入项目名称——在本示例中，我们将使用`myapp`。它还会提示您选择一个程序类型，以及可能根据选择的类型提供其他选项（例如，网络分类器的附加方向）。

如果您愿意，也可以直接在命令行中设置模板选项，例如：

```console
cargo generate --name myapp -d program_type=xdp https://github.com/aya-rs/aya-template
```

有关可用选项的完整列表，请参见[aya-template存储库中的cargo-generate.toml文件](https://github.com/aya-rs/aya-template/blob/main/cargo-generate.toml)。
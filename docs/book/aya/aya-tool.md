# 使用 aya-tool

!!! example "源代码"

    本章示例的完整代码可在[此处](https://github.com/aya-rs/book/tree/main/examples/aya-tool)找到。

在许多情况下，您需要使用正在运行的Linux内核在其源代码中使用的类型定义。例如，您可能需要定义[task_struct](https://elixir.bootlin.com/linux/v5.15.3/source/include/linux/sched.h#L723)，因为您即将编写一个接收新调度进程/任务信息的BPF程序。Aya并没有提供这种结构的定义。应该怎么做才能获得这种定义呢？而且我们需要Rust中的定义，而不是C语言中的。

这就是aya-tool的设计初衷。它是一个允许为特定内核结构生成Rust绑定的工具。

可以通过以下命令进行安装：

```console
$ cargo install bindgen-cli
$ cargo install --git https://github.com/aya-rs/aya -- aya-tool
```

确保您的系统中安装了`bpftool`和`bindgen`，否则`aya-tool`将无法工作。

命令的语法是：

```console
$ aya-tool
aya-tool 

USAGE:
    aya-tool <SUBCOMMAND>

OPTIONS:
    -h, --help    打印帮助信息

SUBCOMMANDS:
    generate    使用bpftool生成内核类型的Rust绑定
    help        打印此消息或给定子命令的帮助信息
```

假设我们想要生成[task_struct](https://elixir.bootlin.com/linux/v5.15.3/source/include/linux/sched.h#L723)的Rust定义。假设您的项目叫做`myapp`。您的用户空间部分在`myapp`子目录中，您的eBPF部分在`myapp-ebpf`中。我们需要为eBPF部分生成绑定，可以通过以下命令完成：

```console
$ aya-tool generate task_struct > myapp-ebpf/src/vmlinux.rs
```

!!! tip "为多个类型生成"

    您也可以指定多个类型进行生成，例如：
    ```console
    $ aya-tool generate task_struct dentry > vmlinux.rs
    ```
    但在接下来的示例中，我们将只关注`task_struct`。

然后我们可以在我们的eBPF程序中使用`mod vmlinux`将`vmlinux`作为一个模块，就像这样：

```rust linenums="1" title="myapp-ebpf/src/main.rs"
--8<-- "examples/aya-tool/myapp-ebpf/src/main.rs"
```

## 可移植性和不同的内核版本

通过名为[BPF CO-RE](https://facebookmicrosites.github.io/bpf/blog/2020/02/19/bpf-portability-and-co-re.html)的机制，aya-tool生成的结构在不同的Linux内核版本之间是可移植的。这些结构不是简单地从内核头文件中生成的。然而，目标内核（无论版本如何）都应该启用`CONFIG_DEBUG_INFO_BTF`选项。
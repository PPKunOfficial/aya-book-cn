# 探针

!!! example "源代码"

    本章示例的完整代码可在[此处](https://github.com/aya-rs/book/tree/main/examples/kprobetcp)找到。

## eBPF中的探针是什么？

探针BPF程序可以附加到内核（kprobes）或用户端（uprobes）函数，并能够访问这些函数的参数。您可以在[内核文档](https://docs.kernel.org/trace/kprobes.html)中找到有关探针的更多信息，包括kprobes和kretprobes之间的区别。

## 示例项目

为了使用Aya说明kprobes，让我们编写一个程序，将eBPF处理程序附加到[`tcp_connect`](https://elixir.bootlin.com/linux/latest/source/net/ipv4/tcp_output.c#L3837)函数，并允许打印来自套接字参数的源和目标IP地址。

## 设计

对于这个演示程序，我们将依赖aya-log从BPF程序中打印IP地址，并且不会有任何自定义的BPF映射（除了那些由aya-log创建的）。

## eBPF代码
- 从`tcp_connect`的签名中可以看出，`struct sock *sk`是唯一的函数参数。我们将从`ProbeContext`的ctx句柄中访问它。
- 我们调用`bpf_probe_read_kernel`辅助函数来复制套接字结构的`struct sock_common __sk_common`部分。（对于uprobes程序，我们需要调用`bpf_probe_read_user`。）
- 我们匹配`skc_family`字段，并为`AF_INET`（IPv4）和`AF_INET6`（IPv6）值提取和打印源和目标地址，使用aya-log的`info!`宏。

以下是eBPF代码的样子：

```rust linenums="1" title="kprobetcp-ebpf/src/main.rs"
--8<-- "examples/kprobetcp/kprobetcp-ebpf/src/main.rs"
```

## 用户空间代码

用户空间代码的目的是加载eBPF程序并将其附加到`tcp_connect`函数。

以下是代码的样子：

```rust linenums="1" title="kprobetcp/src/main.rs"
--8<-- "examples/kprobetcp/kprobetcp/src/main.rs"
```

## 运行程序

```console
$ RUST_LOG=info cargo xtask run --release
[2022-12-28T20:50:00Z INFO  kprobetcp] Waiting for Ctrl-C...
[2022-12-28T20:50:05Z INFO  kprobetcp] AF_INET6 src addr: 2001:4998:efeb:282::249, dest addr: 2606:2800:220:1:248:1893:25c8:1946
[2022-12-28T20:50:11Z INFO  kprobetcp] AF_INET src address: 10.53.149.148, dest address: 10.87.116.72
[2022-12-28T20:50:30Z INFO  kprobetcp] AF_INET src address: 10.53.149.148, dest address: 98.138.219.201
```
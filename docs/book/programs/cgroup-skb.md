# Cgroup SKB

!!! example "源代码"

    本章示例的完整代码可在[此处](https://github.com/aya-rs/book/tree/main/examples/cgroup-skb-egress)找到。

## 什么是Cgroup SKB？

Cgroup SKB程序附加到v2 cgroup，并通过与给定cgroup内的进程相关的网络流量（出站或入站）触发。它们允许拦截和过滤与特定cgroup（因此也包括容器）相关的流量。

## Cgroup SKB和分类器有什么区别？

Cgroup SKB和分类器都接收相同类型的上下文——`SkBuffContext`。

区别在于分类器附加到网络接口。

## 示例项目

本示例将类似于[分类器](classifiers.md)示例——一个允许丢弃特定cgroup出站流量的程序。

## 设计

我们将：

- 创建一个`HashMap`，用作阻止列表。
- 从数据包中检查目的IP地址，并根据`HashMap`做出策略决策（通过或丢弃）。
- 从用户空间向阻止列表中添加条目。

## 生成vmlinux.h的绑定

在本例中，我们将使用一个名为`iphdr`的内核结构，它代表IP协议头。我们需要生成它的Rust绑定。

首先，我们必须确保`bindgen`已安装。
```sh
cargo install bindgen-cli
```

我们使用`xtask`来自动化绑定生成过程，以便将来可以通过添加以下代码轻松重现：

=== "xtask/src/codegen.rs"

    ```rust linenums="1"
    --8<-- "examples/cgroup-skb-egress/xtask/src/codegen.rs"
    ```

=== "xtask/Cargo.toml"

    ```toml linenums="1"
    --8<-- "examples/cgroup-skb-egress/xtask/Cargo.toml"
    ```

=== "xtask/src/main.rs"

    ```rust linenums="1"
    --8<-- "examples/cgroup-skb-egress/xtask/src/main.rs"
    ```

一旦我们从项目根目录使用`cargo xtask codegen`生成了文件，我们可以通过在eBPF代码中包含`mod bindings`来访问它。

## eBPF代码

程序将从定义`BLOCKLIST`映射开始。为了强制执行策略，程序将在该映射中查找目的IP地址。如果该地址的映射条目存在，我们将通过返回`0`来丢弃数据包。否则，我们将通过返回`1`来接受它。

以下是eBPF代码的样子：

```rust linenums="1" title="cgroup-skb-egress-ebpf/src/main.rs"
--8<-- "examples/cgroup-skb-egress/cgroup-skb-egress-ebpf/src/main.rs"
```

1. 创建我们的映射。
2. 检查是否应该允许或拒绝数据包。
3. 返回正确的操作。

## 用户空间代码

用户空间代码的目的是加载eBPF程序，将其附加到cgroup，然后用要阻止的地址填充映射。

在此示例中，我们将阻止所有出站到`1.1.1.1`的流量。

以下是代码的样子：

```rust linenums="1" title="cgroup-skb-egress/src/main.rs"
--8<-- "examples/cgroup-skb-egress/cgroup-skb-egress/src/main.rs"
```

1. 加载eBPF程序。
2. 将其附加到给定的cgroup。
3. 用我们希望阻止出站流量的远程IP地址填充映射。

第三步是通过获取`BLOCKLIST`映射的引用并调用`blocklist.insert`完成的。在Rust中使用`IPv4Addr`类型将允许我们读取IP地址的易读表示并将其转换为`u32`，这是在eBPF映射中使用的适当类型。

## 测试程序

首先，检查cgroup v2的挂载位置：

```console
$ mount | grep cgroup2
cgroup2 on /sys/fs/cgroup type cgroup2 (rw,nosuid,nodev,noexec,relatime,nsdelegate,memory_recursiveprot)
```

最常见的位置是`/sys/fs/cgroup`或`/sys/fs/cgroup/unified`。

在该位置内，我们需要创建一个新的cgroup（以root身份）：

```console
# mkdir /sys/fs/cgroup/foo
```

然后运行程序：

```console
RUST_LOG=info cargo xtask run
```

然后，在一个单独的终端中，以root身份，尝试访问`1.1.1.1`：

```console
# bash -c "echo \$ >> /sys/fs/cgroup/foo/cgroup.procs && curl 1.1.1.1"
```

该命令应挂起，我们程序的日志应如下所示：

```console
LOG: DST 1.1.1.1, ACTION 0
LOG: DST 1.1.1.1, ACTION 0
```

另一方面，访问任何其他地址应成功，例如：

```console
# bash -c "echo \$ >> /sys/fs/cgroup/foo/cgroup.procs && curl google.com"
<HTML><HEAD><meta http-equiv="content-type" content="text/html;charset=utf-8">
<TITLE>301 Moved</TITLE></HEAD><BODY>
<H1>301 Moved</H1>
The document has moved
<A HREF="http://www.google.com/">here</A>.
</BODY></HTML>
```

并应产生以下日志：

```console
LOG: DST 192.168.88.10, ACTION 1
LOG: DST 192.168.88.10, ACTION 1
LOG: DST 172.217.19.78, ACTION 1
LOG: DST 172.217.19.78, ACTION 1
LOG: DST 172.217.19.78, ACTION 1
LOG: DST 172.217.19.78, ACTION 1
LOG: DST 172.217.19.78, ACTION 1
LOG: DST 172.217.19.78, ACTION 1
```
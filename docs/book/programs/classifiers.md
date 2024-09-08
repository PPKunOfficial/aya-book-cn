# 分类器

!!! example "源代码"

    本章示例的完整代码可在[此处](https://github.com/aya-rs/book/tree/main/examples/tc-egress)找到。

## eBPF中的分类器是什么？

分类器是一种eBPF程序，它附加到Linux内核网络中的**排队学科**（通常称为**qdisc**），因此可以对与qdisc关联的网络接口上接收到的数据包做出决策。

对于每个网络接口，入站和出站流量都有单独的qdisc。当将分类器程序附加到接口时，

## 分类器和XDP有什么区别？

* 分类器比XDP更早出现，自内核4.1版本起可用，而XDP自4.8版本起可用。
* 分类器可以检查入站和出站流量。XDP仅限于入站。
* XDP提供更好的性能，因为它执行得更早——在数据包到达任何内核网络栈层并解析为`sk_buff`结构之前，它接收到的是来自NIC驱动的原始数据包。

## 示例项目

为了与XDP示例有所不同，我们尝试编写一个允许丢弃出站流量的程序。

## 设计

我们将：

- 创建一个`HashMap`，用作阻止列表。
- 从数据包中检查目的IP地址，并根据`HashMap`做出策略决策（通过或丢弃）。
- 从用户空间向阻止列表中添加条目。

## eBPF代码

程序代码将从定义`BLOCKLIST`映射开始。为了强制执行策略，程序将在该映射中查找目的IP地址。如果该地址的映射条目存在，我们将丢弃数据包。否则，我们将使用`TC_ACT_PIPE`动作**传递**它——这意味着在我们这一侧允许它，但也让其他分类器程序和qdisc过滤器检查数据包。

!!! note "TC_ACT_OK"

    还有一种可能性是允许数据包绕过其他程序和过滤器——`TC_ACT_OK`。我们建议只有在绝对确定您希望您的程序优先于其他程序或过滤器时才使用该选项。

以下是eBPF代码的样子：

```rust linenums="1" title="tc-egress-ebpf/src/main.rs"
--8<-- "examples/tc-egress/tc-egress-ebpf/src/main.rs"
```

1. 创建我们的映射。
2. 检查是否应该允许或拒绝数据包。
3. 返回正确的操作。

## 用户空间代码

用户空间代码的目的是加载eBPF程序，将其附加到给定的网络接口，然后用要阻止的地址填充映射。

在此示例中，我们将阻止所有出站到`1.1.1.1`的流量。

以下是代码的样子：

```rust linenums="1" title="tc-egress/src/main.rs"
--8<-- "examples/tc-egress/tc-egress/src/main.rs"
```

1. 获取映射的引用。
2. 创建一个IPv4Addr。
3. 用我们希望阻止出站流量的远程IP地址填充映射。

第三步是通过获取`BLOCKLIST`映射的引用并调用`blocklist.insert`完成的。在Rust中使用`IPv4Addr`类型将允许我们读取IP地址的易读表示并将其转换为`u32`，这是在eBPF映射中使用的适当类型。

## 运行程序

```console
$ RUST_LOG=info cargo xtask run
LOG: SRC 1.1.1.1, ACTION 2
LOG: SRC 35.186.224.47, ACTION 3
LOG: SRC 35.186.224.47, ACTION 3
LOG: SRC 1.1.1.1, ACTION 2
LOG: SRC 168.100.68.32, ACTION 3
LOG: SRC 168.100.68.239, ACTION 3
LOG: SRC 168.100.68.32, ACTION 3
LOG: SRC 168.100.68.239, ACTION 3
LOG: SRC 1.1.1.1, ACTION 2
LOG: SRC 13.248.212.111, ACTION 3
```
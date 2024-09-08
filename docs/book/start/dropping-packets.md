# 丢弃数据包

在上一章中，我们的XDP程序只是记录了流量。在本章中，我们将扩展它以允许丢弃流量。

!!! example "源代码"

    本章示例的完整代码可在[此处](https://github.com/aya-rs/book/tree/main/examples/xdp-drop)找到。

## 设计

为了使我们的程序能够丢弃数据包，我们需要一个IP地址列表来进行丢弃。由于我们希望能够高效地查找这些地址，我们将使用[`HashMap`](https://docs.rs/aya/latest/aya/maps/struct.HashMap.html)来存储它们。

我们将：

- 在我们的eBPF程序中创建一个`HashMap`作为阻止列表
- 根据`HashMap`检查数据包的IP地址以做出策略决定（通过或丢弃）
- 从用户空间添加条目到阻止列表

## 在eBPF中丢弃数据包

我们将在eBPF代码中创建一个名为`BLOCKLIST`的新映射。为了做出策略决定，我们需要在`HashMap`中查找源IP地址。如果存在，我们就丢弃数据包；如果不存在，我们允许它。我们将在一个名为`block_ip`的函数中保留这个逻辑。

以下是代码的样子：

```rust linenums="1" title="xdp-drop-ebpf/src/main.rs"
--8<-- "examples/xdp-drop/xdp-drop-ebpf/src/main.rs"
```

1. 创建我们的映射
2. 检查我们是否应该允许或拒绝我们的数据包
3. 返回正确的操作

## 从用户空间填充我们的映射

为了添加要阻止的地址，我们首先需要获取对`BLOCKLIST`映射的引用。一旦我们得到它，只需调用`blocklist.insert()`即可。我们将使用`IPv4Addr`类型来表示我们的IP地址，因为它是人类可读的，并且可以轻松转换为`u32`。在这个示例中，我们将阻止所有来自`1.1.1.1`的流量。

!!! note "字节序"

    在数据包中，IP地址总是以网络字节序（大端）编码。在我们的eBPF程序中，在检查阻止列表之前，我们使用`u32::from_be`将它们转换为主机字节序。因此，从用户空间以主机字节序格式编写我们的IP地址是正确的。

    另一种方法也可以工作：我们可以在从用户空间插入时将IP转换为网络字节序，这样在eBPF程序中进行索引时就不需要转换。

以下是用户空间代码的样子：

```rust linenums="1" title="xdp-drop/src/main.rs"
--8<-- "examples/xdp-drop/xdp-drop/src/main.rs"
```

1. 获取对映射的引用
2. 创建一个IPv4Addr
3. 将此写入我们的映射

## 运行程序

```console
$ RUST_LOG=info cargo xtask run
[2022-10-04T12:46:05Z INFO  xdp_drop] SRC: 1.1.1.1, ACTION: 1
[2022-10-04T12:46:05Z INFO  xdp_drop] SRC: 192.168.1.21, ACTION: 2
[2022-10-04T12:46:05Z INFO  xdp_drop] SRC: 192.168.1.21, ACTION: 2
[2022-10-04T12:46:05Z INFO  xdp_drop] SRC: 18.168.253.132, ACTION: 2
[2022-10-04T12:46:05Z INFO  xdp_drop] SRC: 1.1.1.1, ACTION: 1
[2022-10-04T12:46:05Z INFO  xdp_drop] SRC: 18.168.253.132, ACTION: 2
[2022-10-04T12:46:05Z INFO  xdp_drop] SRC: 18.168.253.132, ACTION: 2
[2022-10-04T12:46:05Z INFO  xdp_drop] SRC: 1.1.1.1, ACTION: 1
[2022-10-04T12:46:05Z INFO  xdp_drop] SRC: 140.82.121.6, ACTION: 2
```

在此输出中，`ACTION: 1`表示数据包被丢弃，而`ACTION: 2`表示数据包被允许通过。
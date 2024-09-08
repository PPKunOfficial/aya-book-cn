# 解析数据包

在上一章中，我们的XDP应用程序运行直到按下Ctrl-C，并允许所有流量。每次接收到数据包时，eBPF程序会记录字符串`"received a packet"`。在本章中，我们将展示如何解析数据包。

虽然我们可以深入解析到L7，但我们将把示例限制在L3，并且为了简化，只处理IPv4。

!!! example "源代码"

    本章示例的完整代码可在[此处](https://github.com/aya-rs/book/tree/main/examples/xdp-log)找到。

## 使用网络类型

我们将记录传入数据包的源IP地址。因此，我们需要：

* 读取以太网头以确定是否处理IPv4数据包，否则终止解析。
* 从IPv4头读取源IP地址。

我们可以查阅这些协议的规范并手动解析，但我们将使用[network-types](https://crates.io/crates/network-types) crate，它提供了许多常见互联网协议的便捷类型定义。

让我们通过在`xdp-log-ebpf/Cargo.toml`中添加对`network-types`的依赖，将其添加到我们的eBPF crate中：

=== "xdp-log-ebpf/Cargo.toml"

    ```toml linenums="1"
    --8<-- "examples/xdp-log/xdp-log-ebpf/Cargo.toml"
    ```

## 从上下文获取数据包数据

`XdpContext`包含我们将使用的两个字段：`data`和`data_end`，它们分别是指向数据包开始和结束的指针。

为了访问数据包中的数据并确保以使eBPF验证器满意的方式进行，我们将引入一个名为`ptr_at`的辅助函数。该函数确保在访问任何数据包数据之前，我们插入验证器所需的边界检查。

最后，为了访问以太网和IPv4头的各个字段，我们将使用memoffset crate，让我们在`xdp-log-ebpf/Cargo.toml`中为其添加依赖。

!!! tip "使用`offset_of!`读取字段"

    由于堆栈空间有限，使用`offset_of!`宏读取结构体中的单个字段比读取整个结构体并通过名称访问字段更节省内存。

生成的代码如下所示：

```rust linenums="1" title="xdp-log-ebpf/src/main.rs"
--8<-- "examples/xdp-log/xdp-log-ebpf/src/main.rs"
```

1. 在这里我们定义`ptr_at`以确保数据包访问总是进行边界检查。
2. 使用`ptr_at`读取我们的以太网头。
3. 在这里我们记录IP和端口。

不要忘记重新构建您的eBPF程序！

## 用户空间组件

我们的用户空间代码与上一章没有太大区别，但为了参考，以下是代码：

```rust linenums="1" title="xdp-log/src/main.rs"
--8<-- "examples/xdp-log/xdp-log/src/main.rs"
```

## 运行程序

与之前一样，可以通过提供接口名称作为参数来覆盖接口，例如，`RUST_LOG=info cargo xtask run -- --iface wlp2s0`。

```console
$ RUST_LOG=info cargo xtask run
[2022-12-22T11:32:21Z INFO  xdp_log] SRC IP: 172.52.22.104, SRC PORT: 443
[2022-12-22T11:32:21Z INFO  xdp_log] SRC IP: 172.52.22.104, SRC PORT: 443
[2022-12-22T11:32:21Z INFO  xdp_log] SRC IP: 172.52.22.104, SRC PORT: 443
[2022-12-22T11:32:21Z INFO  xdp_log] SRC IP: 172.52.22.104, SRC PORT: 443
[2022-12-22T11:32:21Z INFO  xdp_log] SRC IP: 234.130.159.162, SRC PORT: 443
```

每次接收到数据包时，程序会记录其源IP地址和端口。
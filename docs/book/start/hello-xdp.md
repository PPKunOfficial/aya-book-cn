# Hello XDP!

!!! example "源代码"

    本章示例的完整代码可在[此处](https://github.com/aya-rs/book/tree/main/examples/xdp-hello)找到。

## 示例项目

虽然有许多跟踪点可以附加和编写程序类型，但我们应该从简单的地方开始。

XDP（eXpress Data Path）程序允许我们的eBPF程序对接收到的数据包做出决策，以决定是否允许数据包通过接口。为了保持简单，我们将构建一个非常简单的防火墙来允许或拒绝流量。

## eBPF组件

### 允许所有流量

我们必须首先编写我们程序的eBPF组件。
这是一个允许所有流量的最小化生成的XDP程序。
该程序的逻辑位于`xdp-hello-ebpf/src/main.rs`中，目前看起来是这样的：

```rust linenums="1" title="xdp-hello-ebpf/src/main.rs"
--8<-- "examples/xdp-hello/xdp-hello-ebpf/src/main.rs"
```

1. 使用`#![no_std]` 是因为我们不能使用标准库。
2. 使用`#![no_main]` 是因为我们没有主函数。
3. `#[panic_handler]` 是为了让编译器满意，尽管它从未使用过，因为我们不能panic。
4. 这表明这个函数是一个XDP程序。
5. 我们的主入口点委托给另一个函数并执行错误处理，返回`XDP_ABORTED`，这将丢弃数据包。
6. 每次接收到数据包时写入日志条目。
7. 这个函数返回一个允许所有流量的`Result`。

现在我们可以使用`cargo xtask build-ebpf`来编译它。

### 验证程序

让我们看看编译后的eBPF程序：

```console
$ llvm-objdump -S target/bpfel-unknown-none/debug/xdp-hello

target/bpfel-unknown-none/debug/xdp-hello:	file format elf64-bpf

Disassembly of section .text:

0000000000000000 <memset>:
       0:	15 03 06 00 00 00 00 00	if r3 == 0 goto +6 <LBB1_3>
       1:	b7 04 00 00 00 00 00 00	r4 = 0

0000000000000010 <LBB1_2>:
       2:	bf 15 00 00 00 00 00 00	r5 = r1
       3:	0f 45 00 00 00 00 00 00	r5 += r4
       4:	73 25 00 00 00 00 00 00	*(u8 *)(r5 + 0) = r2
       5:	07 04 00 00 01 00 00 00	r4 += 1
       6:	2d 43 fb ff 00 00 00 00	if r3 > r4 goto -5 <LBB1_2>

0000000000000038 <LBB1_3>:
       7:	95 00 00 00 00 00 00 00	exit

0000000000000040 <memcpy>:
       8:	15 03 09 00 00 00 00 00	if r3 == 0 goto +9 <LBB2_3>
       9:	b7 04 00 00 00 00 00 00	r4 = 0

0000000000000050 <LBB2_2>:
      10:	bf 15 00 00 00 00 00 00	r5 = r1
      11:	0f 45 00 00 00 00 00 00	r5 += r4
      12:	bf 20 00 00 00 00 00 00	r0 = r2
      13:	0f 40 00 00 00 00 00 00	r0 += r4
      14:	71 00 00 00 00 00 00 00	r0 = *(u8 *)(r0 + 0)
      15:	73 05 00 00 00 00 00 00	*(u8 *)(r5 + 0) = r0
      16:	07 04 00 00 01 00 00 00	r4 += 1
      17:	2d 43 f8 ff 00 00 00 00	if r3 > r4 goto -8 <LBB2_2>

0000000000000090 <LBB2_3>:
      18:	95 00 00 00 00 00 00 00	exit

Disassembly of section xdp/xdp_hello:

0000000000000000 <xdp_hello>:
       0:	bf 16 00 00 00 00 00 00	r6 = r1
       1:	b7 07 00 00 00 00 00 00	r7 = 0
       2:	63 7a fc ff 00 00 00 00	*(u32 *)(r10 - 4) = r7
       3:	bf a2 00 00 00 00 00 00	r2 = r10
:
     245:	18 03 00 00 ff ff ff ff 00 00 00 00 00 00 00 00	r3 = 4294967295 ll
     247:	bf 04 00 00 00 00 00 00	r4 = r0
     248:	b7 05 00 00 aa 00 00 00	r5 = 170
     249:	85 00 00 00 19 00 00 00	call 25

00000000000007d0 <LBB0_2>:
     250:	b7 00 00 00 02 00 00 00	r0 = 2
     251:	95 00 00 00 00 00 00 00	exit
```

输出经过了简化以保持简洁。
我们可以在这里看到一个`xdp/xdp_hello`段。
在`<LBB0_2>`中，`r0 = 2`将寄存器`0`设置为`2`，这是`XDP_PASS`动作的值。
`exit`结束程序。

简单吧！

## 用户空间组件

现在我们的eBPF程序已经完成并编译，我们需要一个用户空间程序来加载它并将其附加到一个跟踪点。
幸运的是，我们在`xdp-hello/src/main.rs`中有一个生成的程序可以为我们完成这项工作。

### 开始

让我们看看生成的用户空间应用程序的细节：

```rust linenums="1" title="xdp-hello/src/main.rs"
--8<-- "examples/xdp-hello/xdp-hello/src/main.rs"
```

1. `tokio`是我们使用的异步库，它提供了一个[Ctrl-C处理程序](https://docs.rs/tokio/latest/tokio/signal/fn.ctrl_c.html)。在我们扩展初始程序的功能时，它将派上用场。
2. 在这里我们声明了CLI标志。现在只有`--iface`用于传递接口名称。
3. 这是我们的主入口点。
4. `include_bytes_aligned!()`在编译时复制BPF ELF对象文件的内容。
5. `Bpf::load()`从上一个命令的输出中读取BPF ELF对象文件的内容，创建任何映射，执行BTF重定位。
6. 我们提取XDP程序。
7. 然后将其加载到内核中。
8. 最后，我们可以将其附加到接口。

让我们试试看！

```console
$ cargo xtask run -- -h
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/xtask run -- -h`
:
    Finished dev [optimized] target(s) in 0.90s
    Finished dev [unoptimized + debuginfo] target(s) in 0.60s
xdp-hello

USAGE:
    xdp-hello [OPTIONS]

OPTIONS:
    -h, --help             打印帮助信息
    -i, --iface <IFACE>    [默认: eth0]
```

!!! note "接口名称"

    此命令假定接口默认是`eth0`。如果您希望附加到其他名称的接口，请使用`RUST_LOG=info cargo xtask run -- --iface wlp2s0`，其中`wlp2s0`是您的接口。

```console
$ RUST_LOG=info cargo xtask run
[2022-12-21T18:03:09Z INFO  xdp_hello] Waiting for Ctrl-C...
[2022-12-21T18:03:11Z INFO  xdp_hello] received a packet
[2022-12-21T18:03:11Z INFO  xdp_hello] received a packet
[2022-12-21T18:03:11Z INFO  xdp_hello] received a packet
[2022-12-21T18:03:11Z INFO  xdp_hello] received a packet
^C[2022-12-21T18:03:11Z INFO  xdp_hello] Exiting...
```

因此，每次在接口上接收到数据包时，都会打印一个日志！

!!! bug "加载程序时出错？"

    如果加载程序时出错，请尝试将`XdpFlags::default()`更改为`XdpFlags::SKB_MODE`。

### eBPF程序的生命周期

程序运行直到按下CTRL+C然后退出。
退出时，Aya会帮助我们卸载程序。

如果在`xdp_hello`运行时发出`sudo bpftool prog list`命令，您可以验证它是否已加载：

```console
958: xdp  name xdp_hello  tag 0137ce4fce70b467  gpl
	loaded_at 2022-06-23T13:55:28-0400  uid 0
	xlated 2016B  jited 1138B  memlock 4096B  map_ids 275,274,273
	pids xdp-hello(131677)
```

再次运行命令，一旦`xdp_hello`退出，将显示程序不再运行。
# XDP

!!! example "源代码"

    本章示例的完整代码可在[此处](https://github.com/aya-rs/book/tree/main/examples/xdp-drop)找到。

## 什么是eBPF中的XDP？

XDP（eXpress Data Path）是一种eBPF程序，附加到网络接口。
它使得能够在网络数据包从网络驱动接收时立即进行过滤、操作及重定向，
甚至在它们进入Linux内核网络栈之前，从而实现低延迟和高吞吐量。

XDP的思想是在内核的`RX`路径中添加一个早期钩子，
并让用户提供的eBPF程序决定数据包的命运。
该钩子放置在NIC驱动中，紧接中断处理之后，
并且在任何网络栈自身所需的内存分配之前。

XDP程序允许编辑数据包数据，
在XDP程序返回后，动作代码决定如何处理数据包：

* `XDP_PASS`: 让数据包继续通过网络栈
* `XDP_DROP`: 静默丢弃数据包
* `XDP_ABORTED`: 丢弃数据包并记录异常
* `XDP_TX`: 将数据包返回到其到达的同一NIC
* `XDP_REDIRECT`: 通过[`AF_XDP`](https://www.kernel.org/doc/html/latest/networking/af_xdp.html)地址族将数据包重定向到另一个NIC或用户空间套接字

## AF_XDP

随着XDP的出现，Linux内核在4.18版本中引入了一个新的地址族。
`AF_XDP`，以前称为`AF_PACKETv4`（从未包含在主线内核中），
是一种为高性能数据包处理而优化的原始套接字，
允许在内核和应用程序之间进行零拷贝。
由于套接字可以用于接收和发送，
它支持在用户空间中纯粹运行的高性能网络应用程序。

如果您想要关于`AF_XDP`的更详细的解释，
可以在[内核文档](https://www.kernel.org/doc/html/latest/networking/af_xdp.html)中找到。

## XDP操作模式

您可以使用以下模式将XDP程序连接到接口：

### 通用XDP

* XDP程序作为普通网络路径的一部分加载到内核中
* 不需要网络卡驱动的支持即可运行
* 不提供完整的性能优势
* 测试XDP程序的简单方法

### 原生XDP

* XDP程序由网络卡驱动作为其初始接收路径的一部分加载
* 需要网络卡驱动的支持才能运行
* 默认操作模式

### 卸载XDP

* XDP程序直接加载到NIC上，并在不使用CPU的情况下执行
* 需要NIC的支持

## 驱动支持原生XDP

支持原生XDP的驱动的列表可以在下表中找到：

| 厂商              | 驱动       | XDP支持版本 |
| ----------------- | ---------- | ----------- |
| Amazon            | ena        | >=5.6       |
| Broadcom          | bnxt_en    | >=4.11      |
| Cavium            | thunderx   | >=4.12      |
| Freescale         | dpaa2      | >=5.0       |
| Intel             | ixgbe      | >=4.12      |
| Intel             | ixgbevf    | >=4.17      |
| Intel             | i40e       | >=4.13      |
| Intel             | ice        | >=5.5       |
| Marvell           | mvneta     | >=5.5       |
| Mellanox          | mlx4       | >=4.8       |
| Mellanox          | mlx5       | >=4.9       |
| Microsoft         | hv_netvsc  | >=5.6       |
| Netronome         | nfp        | >=4.10      |
| Others            | virtio_net | >=4.10      |
| Others            | tun/tap    | >=4.14      |
| Others            | bond       | >=5.15      |
| Qlogic            | qede       | >=4.10      |
| Socionext         | netsec     | >=5.3       |
| Solarflare        | sfc        | >=5.5       |
| Texas Instruments | cpsw       | >=5.3       |

您可以使用以下命令检查接口的网络驱动程序名称：
`ethtool -i <interface>`。

## 驱动支持卸载XDP

目前，仅Netronome NFP驱动支持卸载XDP。

## 示例项目

现在您对XDP有了一些了解，让我们继续一个实际的例子。
我们将编写一个简单的XDP程序，用于丢弃来自某些IP的数据包。

### 设置开发环境

确保您已经具备[前提条件](https://aya-rs.dev/book/start/development/)。

由于我们正在编写一个XDP程序，我们将使用XDP模板（通过`cargo generate`创建）：

```
cargo generate --name simple-xdp-program -d program_type=xdp https://github.com/aya-rs/aya-template
```

### 创建eBPF组件

首先，我们必须为我们的程序创建eBPF组件，
在这个组件中，我们将决定如何处理传入的数据包。

由于我们想要丢弃来自某些IP的传入数据包，
我们将在IP在我们的黑名单中时使用`XDP_DROP`动作代码，
对于其他所有情况将使用`XDP_PASS`动作代码。

```rust
#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::HashMap,
    programs::XdpContext,
};

use aya_log_ebpf::info;

use core::mem;
use network_types::{
    eth::{EthHdr, EtherType},
    ip::Ipv4Hdr,
};
```

我们导入了必要的依赖项：

* `aya_ebpf`: 对于XDP动作（`bindings::xdp_action`）、XDP上下文结构`XdpContext`（`programs:XdpContext`），
映射定义（对于我们的HashMap）和XDP程序宏（`macros::{map, xdp}`）
* `aya_log_ebpf`: 用于在eBPF程序中进行日志记录
* `core::mem`: 用于内存操作
* `network_types`: 用于以太网和IP头的定义

!!! note "重要"
    确保在您的`Cargo.toml`中添加`network_types`依赖项。

以下是代码的样子：

```rust
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
```

提供了一个eBPF兼容的panic处理程序，因为
eBPF程序不能使用默认的panic行为。

```rust
#[map]
static BLOCKLIST: HashMap<u32, u32> = HashMap::<u32, u32>::with_max_entries(1024, 0);
```

在这里，我们定义了一个带有`HashMap`的黑名单，
它存储整数（u32），最多可存储1024个条目。

```rust
#[xdp]
pub fn xdp_firewall(ctx: XdpContext) -> u32 {
    match try_xdp_firewall(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}
```

`xdp_firewall`函数（在用户空间中获取）接受`XdpContext`并返回一个`u32`。
它将主要的数据包处理逻辑委托给`try_xdp_firewall`函数。
如果发生错误，该函数返回`xdp_action::XDP_ABORTED`（等同于u32 `0`）。

```rust
#[inline(always)]
unsafe fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return Err(());
    }

    let ptr = (start + offset) as *const T;
    Ok(&*ptr)
}
```

我们的`ptr_at`函数旨在提供安全访问`XdpContext`中指定偏移量的泛型类型`T`。
它通过将所需的内存范围（`start + offset + len`）与数据的结束（`end`）进行比较来执行边界检查。
如果访问在边界内，它返回指向指定类型的指针；否则，
返回错误。我们将使用此函数从`XdpContext`中检索数据。

```rust

fn block_ip(address: u32) -> bool {
    unsafe { BLOCKLIST.get(&address).is_some() }
}

fn try_xdp_firewall(ctx: XdpContext) -> Result<u32, ()> {
    let ethhdr: *const EthHdr = unsafe { ptr_at(&ctx, 0)? };
    match unsafe { (*ethhdr).ether_type } {
        EtherType::Ipv4 => {}
        _ => return Ok(xdp_action::XDP_PASS),
    }

    let ipv4hdr: *const Ipv4Hdr = unsafe { ptr_at(&ctx, EthHdr::LEN)? };
    let source = u32::from_be(unsafe { (*ipv4hdr).src_addr });

    let action = if block_ip(source) {
        xdp_action::XDP_DROP
    } else {
        xdp_action::XDP_PASS
    };
    info!(&ctx, "SRC: {:i}, ACTION: {}", source, action);

    Ok(action)
}
```

`block_ip`函数检查给定的IP地址是否存在于黑名单中。

如前所述，`try_xdp_firewall`包含我们的防火墙的主要逻辑。
我们首先使用`ptr_at`函数从`XdpContext`中检索以太网头，
该头位于`XdpContext`的开头，因此我们使用`0`作为偏移量。

如果数据包不是IPv4（`ether_type`检查），该函数返回`xdp_action::XDP_PASS`并
允许数据包通过网络栈。

`ipv4hdr`用于检索IPv4头，`source`用于存储IPv4头中的源IP地址。
然后，我们使用之前创建的`block_ip`函数将IP地址与黑名单中的IP进行比较。
如果`block_ip`匹配，意味着IP在黑名单中，我们使用`XDP_DROP`动作代码以便它不会
通过网络栈，否则我们使用`XDP_PASS`动作代码让它通过。

最后，我们记录活动，`SRC`是源IP地址，`ACTION`
是对其使用的动作代码。然后返回`Ok(action)`作为结果。

完整代码：

```rust
#![no_std]
#![no_main]
#![allow(nonstandard_style, dead_code)]

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::HashMap,
    programs::XdpContext,
};
use aya_log_ebpf::info;

use core::mem;
use network_types::{
    eth::{EthHdr, EtherType},
    ip::Ipv4Hdr,
};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

#[map]
static IP_BLOCKLIST: HashMap<u32, u32> = HashMap::<u32, u32>::with_max_entries(1024, 0);

#[xdp]
pub fn xdp_firewall(ctx: XdpContext) -> u32 {
    match try_xdp_firewall(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

#[inline(always)]
unsafe fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return Err(());
    }

    let ptr = (start + offset) as *const T;
    Ok(&*ptr)
}

fn block_ip(address: u32) -> bool {
    unsafe { IP_BLOCKLIST.get(&address).is_some() }
}

fn try_xdp_firewall(ctx: XdpContext) -> Result<u32, ()> {
    let ethhdr: *const EthHdr = unsafe { ptr_at(&ctx, 0)? };
    match unsafe { (*ethhdr).ether_type } {
        EtherType::Ipv4 => {}
        _ => return Ok(xdp_action::XDP_PASS),
    }

    let ipv4hdr: *const Ipv4Hdr = unsafe { ptr_at(&ctx, EthHdr::LEN)? };
    let source = u32::from_be(unsafe { (*ipv4hdr).src_addr });

    let action = if block_ip(source) {
        xdp_action::XDP_DROP
    } else {
        xdp_action::XDP_PASS
    };
    info!(&ctx, "SRC: {:i}, ACTION: {}", source, action);

    Ok(action)
}
```

### 从用户空间填充我们的映射

为了添加要阻止的地址，我们首先需要获取到`BLOCKLIST`映射的引用。

一旦我们拥有它，只需调用`ip_blocklist.insert()`即可
将IP插入到黑名单中。

我们将使用`IPv4Addr`类型来表示我们的IP地址，因为
它是易读的，可以轻松地转换为u32。

在这个例子中，我们将阻止所有来自`1.1.1.1`的流量。

!!! note "字节序"

    IP地址始终在数据包中以网络字节顺序（大端）编码。在我们的eBPF程序中，在检查黑名单之前，我们使用`u32::from_be`将它们转换为主机字节序。因此，从用户空间以主机字节序格式编写我们的IP地址是正确的。

    另一种方法也可以：我们可以在从用户空间插入时将IP转换为网络字节序，然后在从eBPF程序中索引时就不需要转换了。

让我们开始编写用户空间代码：

#### 导入依赖项

```rust
use anyhow::Context;
use aya::{
    include_bytes_aligned,
    maps::HashMap,
    programs::{Xdp, XdpFlags},
    Ebpf,
};
use aya_log::EbpfLogger;
use clap::Parser;
use log::{info, warn};
use std::net::Ipv4Addr;
use tokio::signal;
```

* `anyhow::Context`: 为错误处理提供附加的上下文
* `aya`: 提供用于加载eBPF程序的Bpf结构和相关函数，
以及XDP程序及其标志（`aya::programs::{Xdp, XdpFlags}`）
* `aya_log::EbpfLogger`: 用于在eBPF程序中进行日志记录
* `clap::Parser`: 提供参数解析
* `log::{info, warn}`: 我们用于信息和警告消息的[日志库](https://docs.rs/log/latest/log/index.html)
* `std::net::Ipv4Addr`: 用于处理IPv4地址的结构
* `tokio::signal`: 用于异步处理信号，更多信息请参见[此链接](https://docs.rs/tokio/latest/tokio/signal/)

!!! note
    `aya::Bpf`自版本`0.13.0`起被弃用，`aya_log:BpfLogger`自版本`0.2.1`起被弃用。
    如果您使用更高版本，请使用[`aya::Ebpf`](https://docs.aya-rs.dev/aya/struct.ebpf)和
    [`aya_log:EbpfLogger`](https://docs.aya-rs.dev/aya_log/struct.ebpflogger)代替。

#### 定义命令行参数

```rust
#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
}
```

使用[clap的派生功能](https://docs.rs/clap/latest/clap/_derive/index.html)定义了一个用于命令行解析的简单结构，
其中可选参数`iface`用于提供我们的网络接口名称。

#### 主函数

```rust
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    env_logger::init();

    #[cfg(debug_assertions)]
    let mut bpf = Ebpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/simple-xdp-program"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Ebpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/xdp-simple-xdp-program"
    ))?;
    if let Err(e) = EbpfLogger::init(&mut bpf) {
        warn!("failed to initialize eBPF logger: {}", e);
    }
    let program: &mut Xdp =
        bpf.program_mut("xdp_firewall").unwrap().try_into()?;
    program.load()?;
    program.attach(&opt.iface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    let mut blocklist: HashMap<_, u32, u32> =
        HashMap::try_from(bpf.map_mut("BLOCKLIST").unwrap())?;

    let block_addr: u32 = Ipv4Addr::new(1, 1, 1, 1).try_into()?;

    blocklist.insert(block_addr, 0, 0)?;

    info!("Waiting for Ctrl-C...");
    signal::ctrl_c().await?;
    info!("Exiting...");

    Ok(())
}
```

##### 解析命令行参数

在`main`函数中，我们首先使用[`Opt::parse()`](https://docs.rs/clap/latest/clap/trait.Parser.html#method.parse)和之前定义的结构解析命令行参数。

##### 初始化环境日志

使用[`env_logger::init()`](https://docs.rs/env_logger/latest/env_logger/fn.init.html)初始化日志，
稍后我们将在代码中使用环境日志。

##### 加载eBPF程序

使用`Ebpf::load()`加载eBPF程序，根据构建配置选择`debug`或
`release`版本（`debug_assertions`）。

##### 加载和附加我们的XDP

从我们之前定义的eBPF程序中检索名为`xdp_firewall`的XDP程序
 使用`bpf.program_mut()`。
 然后加载XDP程序并将其附加到我们的网络接口。

##### 设置IP黑名单

从eBPF程序加载IP黑名单（`BLOCKLIST`映射）并转换为`HashMap`。
将IP `1.1.1.1`添加到黑名单中。

##### 等待退出信号

程序使用`signal::ctrl_c().await`异步等待`CTRL+C`信号，
一旦收到信号，它会记录退出消息并返回`Ok(())`。

#### 完整的用户空间代码

```rust
use anyhow::Context;
use aya::{
    include_bytes_aligned,
    maps::HashMap,
    programs::{Xdp, XdpFlags},
    Ebpf,
};
use aya_log::EbpfLogger;
use clap::Parser;
use log::{info, warn};
use std::net::Ipv4Addr;
use tokio::signal;

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    env_logger::init();

    #[cfg(debug_assertions)]
    let mut bpf = Ebpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/simple-xdp-program"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Ebpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/xdp-simple-xdp-program"
    ))?;
    if let Err(e) = EbpfLogger::init(&mut bpf) {
        warn!("failed to initialize eBPF logger: {}", e);
    }
    let program: &mut Xdp =
        bpf.program_mut("xdp_firewall").unwrap().try_into()?;
    program.load()?;
    program.attach(&opt.iface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    let mut blocklist: HashMap<_, u32, u32> =
        HashMap::try_from(bpf.map_mut("BLOCKLIST").unwrap())?;

    let block_addr: u32 = Ipv4Addr::new(1, 1, 1, 1).try_into()?;

    blocklist.insert(block_addr, 0, 0)?;

    info!("Waiting for Ctrl-C...");
    signal::ctrl_c().await?;
    info!("Exiting...");

    Ok(())
}
```

### 运行我们的程序！

现在我们已经拥有了eBPF程序的所有组件，可以使用以下命令运行它：`RUST_LOG=info cargo xtask run`
或`RUST_LOG=info cargo xtask run -- --iface <interface>`如果您想提供另一个网络接口名称，
请注意您也可以不带其余部分使用`cargo xtask run`，但不会有任何日志记录。
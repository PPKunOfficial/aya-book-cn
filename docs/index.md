# 首页

eBPF是一种技术，允许运行用户提供的程序在Linux内核中。更多信息请参见["什么是eBPF?" 文档][what-is-ebpf]。

Aya是一个专注于可操作性和开发者体验的eBPF库。它不依赖于[libbpf]或[bcc]，而是完全用Rust从头构建，仅使用[libc] crate来执行系统调用。通过BTF支持和与musl链接，它提供了一种真正的[编译一次，到处运行的解决方案][co-re]，一个单一的自包含二进制文件可以部署在许多Linux发行版和内核版本上。

其提供的一些主要功能包括：

* 支持**BPF类型格式**（BTF），当目标内核支持时自动启用。这允许针对一个内核版本编译的eBPF程序在不同内核版本上运行，而无需重新编译。
* 支持函数调用重定位和全局数据映射，这使得eBPF程序可以进行**函数调用**并使用**全局变量和初始化器**。
* 与[tokio]和[async-std]的**异步支持**。
* 易于部署且构建速度快：Aya不需要内核构建或编译的头文件，甚至不需要C工具链；发布构建在几秒钟内完成。

[what-is-ebpf]:https://ebpf.io/what-is-ebpf
[libbpf]: https://github.com/libbpf/libbpf
[bcc]: https://github.com/iovisor/bcc
[libc]: https://docs.rs/libc
[co-re]: https://facebookmicrosites.github.io/bpf/blog/2020/02/19/bpf-portability-and-co-re.html
[tokio]: https://docs.rs/tokio
[async-std]: https://docs.rs/async-std

## 谁在使用Aya

### [![Deepfence](https://uploads-ssl.webflow.com/63eaa07bbe370228bab003ea/640a069335cf3921e24def21_Deepfence%20Line.svg){ width="150"}](https://deepfence.io/)
Deepfence使用Aya与XDP/TC作为他们的数据包过滤栈。更多信息请参见[这里](https://deepfence.io/aya-your-trusty-ebpf-companion/)。

### [![Exein](https://blog.exein.io/content/images/2023/03/logoexein.png){ width="150"}](https://exein.io)
Exein在[Pulsar](https://pulsar.sh/)中使用Aya，这是一种用于物联网的运行时安全可观察性工具。更多信息请参见[这里](https://github.com/Exein-io/pulsar)。

### [![Kubernetes SIGs](https://github.com/aya-rs/book/assets/5332524/abde6552-10ed-4c52-9717-732d1ec7ea6c){ width="150" }](https://github.com/kubernetes-sigs)
[Kubernetes特别兴趣小组（SIGs）](https://github.com/kubernetes-sigs)使用Aya开发[Blixt](https://github.com/kubernetes-sigs/blixt)，这是一个负载均衡器，支持[Gateway API项目](https://github.com/kubernetes-sigs/gateway-api)的开发和维护。

### [![Red Hat](https://www.redhat.com/cms/managed-files/Asset-Red_Hat-Logo_page-Logo-RGB.svg?itok=yWDK-rRz){ width="150"}](https://redhat.com)
Red Hat使用Aya开发[bpfman](https://github.com/bpfman/bpfman)，这是一个eBPF程序加载守护进程。
# LSM

!!! example "源代码"

    本章示例的完整代码可在[此处](https://github.com/aya-rs/book/tree/main/examples/lsm-nice)找到。

## 什么是LSM

LSM代表[Linux安全模块](https://en.wikipedia.org/wiki/Linux_Security_Modules)，这是一种框架，允许开发人员在Linux内核之上编写安全系统。在[Linux内核文档](https://www.kernel.org/doc/html/latest/security/lsm.html)中也有简要描述。

LSM由内核模块使用，或者（自内核5.7起）由eBPF程序使用。最受欢迎的使用LSM的模块包括AppArmor、SELinux、Smack和TOMOYO。eBPF LSM程序允许开发人员使用eBPF API实现上述模块所实现的相同功能。

LSM背后的核心概念是**LSM钩子**。LSM钩子在内核的关键位置暴露，eBPF程序可以附加到这些钩子上以实现自定义的安全策略。可以通过钩子进行策略控制的操作示例包括：

* 文件系统操作
  * 打开、创建、移动和删除文件
  * 挂载和卸载文件系统
* 任务/进程操作
  * 分配和释放任务、更改任务的用户和组标识
* 套接字操作
  * 创建和绑定套接字
  * 接收和发送消息

上述每个操作都有相应的LSM钩子。每个钩子接受多个参数，这些参数提供有关程序及其操作的上下文，以便实施策略决策。带有其参数的钩子列表可以在[lsm_hook_defs.h](https://github.com/torvalds/linux/blob/master/include/linux/lsm_hook_defs.h)头文件中找到。

例如，考虑`task_setnice`钩子，其定义如下：

```c
LSM_HOOK(int, 0, task_setnice, struct task_struct *p, int nice)
```

该钩子在为系统中的任何进程设置nice值时触发。如果您不熟悉进程优先级的概念，请查看[此文章](https://en.wikipedia.org/wiki/Nice_(Unix))。从定义中可以看出，该钩子接受以下参数：

* `p`是`task_struct`的实例，表示设置nice值的进程
* `nice`是nice值

通过附加到该钩子，eBPF程序可以决定是否接受或拒绝给定的nice值。

除了钩子定义中发现的参数外，eBPF程序还可以访问一个额外的参数——`ret`，这是可能的先前eBPF LSM程序的返回值。

## 确保启用了BPF LSM

在继续编写BPF LSM程序之前，请确保：

* 您的内核版本至少为5.7。
* 启用了BPF LSM。

可以通过以下方式检查第二点：

```console
$ cat /sys/kernel/security/lsm
capability,lockdown,landlock,yama,apparmor,bpf
```

正确的输出应包含`bpf`。如果没有，则必须通过将其添加到内核配置参数中手动启用BPF LSM。可以通过编辑`/etc/default/grub`中的GRUB配置并将以下内容添加到内核参数来实现：

```console
GRUB_CMDLINE_LINUX="lsm=[YOUR CURRENTLY ENABLED LSMs],bpf"
```

然后使用以下命令之一重建GRUB配置（每个命令可能在不同的Linux发行版中可用或不可用）：

```console
# update-grub2
```
```console
# grub2-mkconfig -o /boot/grub2/grub.cfg
```
```console
# grub-mkconfig -o /boot/grub/grub.cfg
```

最后，重新启动系统。

## 编写LSM BPF程序

让我们尝试创建一个由`task_setnice`钩子触发的LSM eBPF程序。该程序的目的是拒绝为特定进程设置低于0的nice值（意味着更高的优先级）。

可以使用`renice`工具更改niceness值：

```console
$ renice [value] -p [pid]
```

使用我们的eBPF程序，我们希望让给定`pid`的`renice`调用使用负`[value]`变得不可能。

eBPF项目由两部分组成：eBPF程序和用户空间程序。为了使我们的示例简单，我们可以尝试拒绝更改加载eBPF程序的用户空间进程的nice值。

第一步是创建一个新项目：

```console
$ cargo generate --name lsm-nice -d program_type=lsm -d lsm_hook=task_setnice https://github.com/aya-rs/aya-template
```

该命令应创建一个新的Aya项目，其中包含一个附加到`task_setnice`钩子的空程序。让我们进入其目录：

```console
$ cd lsm-nice
```

传递给`task_setnice`钩子参数之一是指向[task_struct类型](https://elixir.bootlin.com/linux/v5.15.3/source/include/linux/sched.h#L723)的指针。因此，我们需要使用aya-tool生成`task_struct`的绑定。

> 如果您不熟悉aya-tool，请参考[此部分](../aya/aya-tool.md)。

```console
$ aya-tool generate task_struct > lsm-nice-ebpf/src/vmlinux.rs
```

现在是时候修改`lsm-nice-ebpf`项目并在那里编写实际程序了。完整的程序代码应如下所示：

```rust linenums="1" title="lsm-nice-ebpf/src/main.rs"
--8<-- "examples/lsm-nice/lsm-nice-ebpf/src/main.rs"
```

1. 我们包含自动生成的`task_struct`绑定：
2. 然后我们定义一个全局变量`PID`。我们将值初始化为0，但在运行时，用户空间部分将用我们感兴趣的实际pid修补该值。
3. 最后，我们有程序和关于nice值的处理逻辑。

之后，我们还需要修改用户空间部分。我们不需要像eBPF部分那样多的工作，但我们需要：

1. 获取PID。
2. 记录它。
3. 将其写入eBPF对象中的全局变量。

最终结果应如下所示：

```rust linenums="1" title="lsm-nice/src/main.rs"
--8<-- "examples/lsm-nice/lsm-nice/src/main.rs"
```

1. 我们从获取和记录PID开始：
2. 然后我们设置全局变量：

之后，我们可以使用以下命令构建并运行我们的项目：

```console
$ RUST_LOG=info cargo xtask run
```

输出应包含显示用户空间进程PID的日志行，例如：

```console
16:32:30 [INFO] lsm_nice: [lsm-nice/src/main.rs:22] PID: 573354
```

现在我们可以尝试更改该进程的nice值。设置正值（降低优先级）应仍然有效：

```console
$ renice 10 -p 587184
587184 (process ID) old priority 0, new priority 10
```

但设置负值应不被允许：

```console
$ renice -10 -p 587184
renice: failed to set priority for 587184 (process ID): Operation not permitted
```

如果这样做导致`Operation not permitted`，恭喜，您的LSM eBPF程序正常工作！
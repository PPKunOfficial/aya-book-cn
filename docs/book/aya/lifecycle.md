# 程序生命周期

在Aya中，一个`Bpf`类型的实例管理通过它创建的所有eBPF对象的生命周期。

请看以下示例：

```rust
use aya::Bpf;
use aya::programs::{Xdp, XdpFlags};

fn main() {
    {
        // (1)
        let mut bpf = Bpf::load_file("bpf.o"))?;

        let program: &mut Xdp = bpf.program_mut("xdp").unwrap().try_into().unwrap();
        // (2)
        program.load()?;
        // (3)
        program.attach("eth0", XdpFlags::default()).unwrap();
    }
    // (4)

}
```

1. 当您调用`load`或`load_file`时，所有eBPF代码引用的映射都会被创建并存储在返回的Bpf实例中。
2. 同样，当您将一个程序加载到内核时，它也会被存储在`Bpf`实例中。
3. 当您附加一个程序时，它会保持附加状态，直到其父`Bpf`实例被删除。
4. 在这一点上，`bpf`变量已被删除。我们的程序和映射被分离/卸载。
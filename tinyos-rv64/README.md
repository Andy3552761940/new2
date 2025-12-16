# TinyOS-RV64（RISC-V 64 / QEMU virt）——Milestone-4：Sv39 + 用户/内核隔离 + PageFault Kill

这是一个面向课程/竞赛的“小型内核工程底座”。当前版本已实现：

✅ Milestone-0: 启动 + SBI 打印 + 清 BSS + panic/shutdown  
✅ Milestone-1: S-mode Trap 框架（stvec/scause/sepc）+ 定时器中断  
✅ Milestone-2: 抢占式轮转调度（全寄存器保存恢复）  
✅ Milestone-3: U-mode + 系统调用（write/yield/sleep/exit/get_ticks）  
✅ **Milestone-4: Sv39 虚拟内存 + 每进程独立页表 + 用户/内核隔离 + Page Fault 杀进程**

---

## 1. 依赖安装

```bash
rustup toolchain install stable
rustup target add riscv64gc-unknown-none-elf
rustup component add llvm-tools-preview
cargo install cargo-binutils

sudo apt install qemu-system-misc
sudo apt install gdb-multiarch   # 可选
```

---

## 2. 运行

```bash
make run
```

预期看到：

- `[U-A] alive` 持续打印（正常用户进程）
- `[U-B] about to touch kernel...` 打印一次后，立刻触发 **load page fault**
- 内核打印：`kill user task ... page fault ...`，但系统继续运行（U-A 仍在跑）

---

## 3. 目录结构

```
src/
  main.rs
  entry.S
  trap.S
  user_img.S           # 两个用户程序的“镜像字节”（会被拷贝到用户地址空间）

  mm/                  # Sv39 核心
    mod.rs
    frame.rs           # 物理页分配器（bump）
    pte.rs             # PTE flags 定义（V/R/W/X/U/A/D）
    pagetable.rs       # 三层页表 map/translate + satp 构造
    copy.rs            # copy_from_user / copy_to_user

  task.rs              # 进程/调度：每进程 satp + 状态机
  syscall.rs           # syscall 分发（write 通过翻译读取用户指针）
  timer.rs
  trap.rs
  sbi.rs
  arch.rs
  console.rs
  panic.rs
```

---

## 4. 关键设计点（答辩友好）

- **每进程一套页表 root**：`Task { root_pa, satp }`
- **内核映射共享**：每个进程页表都 identity-map DRAM 为 Supervisor-only（U=0）
- **用户区独立映射**：用户程序与栈映射到低地址，PTE.U=1
- **satp 切换 + sfence.vma**：上下文切换时切换地址空间
- **Page Fault Kill**：用户态访问 U=0 页（如 0x80200000）-> scause=13/15/12 -> kill 进程

---

## 5. 下一步冲一等奖（建议顺序）

1. 物理页分配器升级（bitmap/buddy）+ 回收
2. 用户程序 ELF loader（真正的“用户态应用”）
3. VFS + 文件系统（tmpfs -> FAT32/ext2）
4. virtio-blk / virtio-net 驱动
5. shell + benchmark + CI 完整工程化

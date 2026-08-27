再次 review 后，方向明显更好了，原先 6 个问题大部分已经修复。不过当前仍有两个必须立即修的回归，其中一个会编译失败，另一个可能写错地址。目前约 75%～80%，仍不建议提 PR。

## 必须先修

1. `EDATA_BANK1_BASE` 已不存在，当前无法编译

[edata.rs:27](C:/Users/yj.huang/Documents/Workspace/embassy/embassy-stm32/src/flash/edata.rs:27)：

```rust
(Self::Bank1, false) | (Self::Bank2, true) => EDATA_BANK1_BASE,
```

前面已经改名为 `EDATA_BASE`，这里应该是：

```rust
(Self::Bank1, false) | (Self::Bank2, true) => EDATA_BASE,
```

定向 `cargo check` 已确认报 `E0425`。

2. `edata_write()` 丢失了 `offset`

[edata.rs:195](C:/Users/yj.huang/Documents/Workspace/embassy/embassy-stm32/src/flash/edata.rs:195) 当前是：

```rust
family::blocking_write_edata(bank.base(), data)?;
```

这会导致：

```rust
edata_write(Bank1, 0x100, data)
```

实际仍从 bank 起始地址写，而不是 `0x100`。

建议直接复用现有检查函数：

```rust
let address = checked_address(bank, offset, data.len())?;

unsafe { family::unlock() };
let _lock = OnDrop::new(|| unsafe { family::lock() });

unsafe { family::blocking_write_edata(address, data) }
```

这是当前最危险的正确性问题。

## 底层写入建议到底是什么意思

是的，需要删除一些代码，但主要删除的是 `c5.rs` 里重复的底层实现，不一定删除用户能看到的 API。

理想调用关系是：

```text
edata_write_u16 ─┐
edata_write_u32 ─┼──> blocking_write_edata(address, &[u8])
edata_write ─────┘
```

也就是说，只有 `blocking_write_edata()` 真正负责：

- 设置 `PG`
- volatile 写入
- 等待 ready
- 检查错误
- 清除 EOP
- 清除 `PG`
- 执行屏障

当前 [c5.rs:39](C:/Users/yj.huang/Documents/Workspace/embassy/embassy-stm32/src/flash/c5.rs:39) 到 [c5.rs:198](C:/Users/yj.huang/Documents/Workspace/embassy/embassy-stm32/src/flash/c5.rs:198) 有三份独立流程。它们已经出现差异：`u32` 版本没有另外两个版本中的屏障和 fence。这正是建议合并的原因。

我的具体建议：

- 保留 `blocking_write_edata(address, data)`。
- 删除 `blocking_write_edata_u16()` 的底层实现，由上层把 `u16` 转成字节后调用通用函数。
- 删除 `blocking_write_edata_u32()` 的底层实现，同样转成字节。
- 第一版 PR 建议暂时删除 `edata_write_u16_slice()` 及对应底层函数；如果确实需要，再实现成薄包装。

例如 `u32` 上层可以直接写成：

```rust
let address = checked_address(bank, offset, 4)?;
let data = value.to_le_bytes();

unsafe { family::unlock() };
let _lock = OnDrop::new(|| unsafe { family::lock() });

unsafe { family::blocking_write_edata(address, &data) }
```

这会执行两次 16 位 programming，而不是一次 32 位 volatile store。STM32C5 同时支持 16/32 位 EDATA programming，因此功能上成立；如果以后有明确性能或单次 32 位编程需求，再在统一底层里增加优化即可。[STM32C5 HAL FLASH 实现](https://github.com/STMicroelectronics/stm32c5xx-drivers/blob/main/hal/stm32c5xx_hal_flash.c)

一句话概括：不是为了少几个函数名，而是为了寄存器操作流程只有一份。

## 仍未解决的问题

- [edata.rs:151](C:/Users/yj.huang/Documents/Workspace/embassy/embassy-stm32/src/flash/edata.rs:151) 的通用 byte read 仍然使用 `from_raw_parts + copy_from_slice`，不能保证实际产生 16 位 volatile 访问。第一版建议删除 `edata_read()`，只保留 `u16` 读取；或者要求偶数 offset/长度，并逐个 `read_volatile::<u16>()`。
- [c5.rs:21](C:/Users/yj.huang/Documents/Workspace/embassy/embassy-stm32/src/flash/c5.rs:21) 的 `unlock()` 仍只等待 `BSY`。建议改为 `while busy() {}`，覆盖 `BSY/DBNE/WBNE`。
- `u16/u16_slice/u32` 方法中的 unsafe 已经符合 Rust 2024 要求，但外层 unsafe 太大，因此产生三个 `unused_unsafe` warning。像 `edata_write()` 一样缩小 unsafe 范围即可。
- [c5.rs:420](C:/Users/yj.huang/Documents/Workspace/embassy/embassy-stm32/src/flash/c5.rs:420) 注释里的参考手册应从 `RM0492` 改为 `RM0522`。
- [mod.rs:151](C:/Users/yj.huang/Documents/Workspace/embassy/embassy-stm32/src/flash/mod.rs:151) 新增的 `Error::Unsupported` 已没有调用者，可以删除。
- `cargo fmt --check` 和 `git diff --check` 当前都失败，后者有三处 trailing whitespace。
- 工作区里的 `stm32-data/` 是未跟踪的嵌套仓库，提交前避免被 `git add .` 意外加入。

## SWAP_BANK 和元数据结论

当前实现已经选择了比较清晰的语义：

- `EDataBank::Bank1/Bank2` 表示物理 bank。
- `base()` 根据 `SWAP_BANK` 返回当前映射地址。
- `selector()` 始终返回物理 `BKSEL`。

这个方向是正确的，只需要给 `EDataBank` 和相关方法补文档，明确“物理 bank”语义。

根据 STM32C5 系列级 [RM0522](https://www.st.com/resource/en/reference_manual/rm0522-stm32c5-series-armbased-32bit-mcus-stmicroelectronics.pdf)，两 bank、每 bank `0x6000`、16 页、每页 `0x600` 是系列定义。因此固定常量目前比上次 review 时更有依据，不必为了第一版强行修改 `stm32-data`。但最好在常量旁注明 RM0522 章节来源；如果上游坚持元数据生成，再作为 review 后续调整。

建议修复顺序：先解决未定义常量和 offset 写错，再合并底层写入流程，最后处理 byte read、文档和格式。J
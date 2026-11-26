# rawpipe

将相机 RAW 处理成 8/16-bit sRGB 的 Rust 库。解码交给 rawler（当前使用个人 fork [michael-eddy/dnglab](https://github.com/michael-eddy/dnglab)），本仓库只做处理管线，不包含命令行程序。

## 特性

- 输入两类来源（`ImageSource`）：
  - `Raw(RawImage)`：rawler 解出的 Bayer / X-Trans / 单色 / LinearRaw / Foveon（X3F）/ 浮点 DNG
  - `Raster(DynamicImage)`：已解码位图，跳过 RAW 阶段
- 固定管线顺序：
  `to_float`（裁剪 + 黑白电平）→ `demosaic` → `rotatecrop` → `to_lab`（白平衡 + 色彩矩阵）→ `base_curve` → `from_lab` → `gamma` → `orientation`
- 输出：`SrgbImage`（8-bit，带 sRGB 编码）或 `SrgbImage16`（16-bit，**线性**）
- 可直接读取相机内嵌的预览图/缩略图（`preview` / `thumbnail`，文件没有时返回 `None`）
- 支持 X3F/Foveon（Merrill 与 Quattro），含 Quattro R/G 插值
- 支持限尺寸缩放、EXIF 方向、逐通道黑电平、奇数裁剪导致的 CFA 相位平移
- 行级 rayon 并行；`PipelineCache` 可跨调用复用中间结果
- 管线配置可通过 `to_yaml()` / `from_yaml()` 保存与恢复

## 用法

```rust
// 文件 → 8-bit sRGB
let mut pipeline = rawpipe::Pipeline::new_from_file("photo.NEF")?;
let image = pipeline.output_8bit(None)?;

// 一步到位
let image = rawpipe::decode_8bit("photo.NEF", 0, 0)?;

// 内嵌预览 / 缩略图（没有时返回 None）
let preview = rawpipe::preview("photo.X3F")?;
let thumbnail = rawpipe::thumbnail("photo.X3F")?;

// 限制最长边
let mut pipeline = rawpipe::Pipeline::new_from_file("photo.CR3")?;
pipeline.context.settings.max_width = 1920;
pipeline.context.settings.max_height = 1080;
let image = pipeline.output_8bit(None)?;

// 已解码位图
let raster = image::DynamicImage::ImageRgb8(rgb);
let mut pipeline = rawpipe::Pipeline::new_from_source(rawpipe::ImageSource::Raster(raster))?;
let image = pipeline.output_8bit(None)?;

// 手动白平衡（色温/色调）
pipeline.ops.to_lab.set_temp(5000.0, 1.0);

// 缓存中间缓冲区（参数是字节上限，例如 256 MiB）
let cache = rawpipe::Pipeline::new_cache(256 * 1024 * 1024);
let image = pipeline.output_8bit(Some(&cache))?;
```

- `output_8bit` 会应用 sRGB 传输函数；`output_16bit` 设置 `settings.linear = true`，输出线性值。
- `settings.use_fast_path` 对未修改默认参数的位图输入直接走 `image` crate 的快速路径。
- 输出图像的 `data` 是 RGB 交错的 `Vec<u8>` / `Vec<u16>`。
- `preview` / `thumbnail` 返回 `Option<image::DynamicImage>`；文件没有内嵌预览时返回 `Ok(None)`。

## 构建

- Rust 1.89+（rawler 的 MSRV），edition 2021
- 依赖：`rawler`（git，个人 fork `michael-eddy/dnglab` 的 `main`，当前 `aeeb8ea`）、`image`、`rayon`、`multicache`、`postcard`、`serde` 等
- `Cargo.toml` 的 `[profile.dev] opt-level = 3`：首次编译/测试会编译整个依赖树，较慢

## 测试

```sh
cargo test            # 58 个用例
```

| 测试 | 内容 |
| --- | --- |
| `decode_formats_test` | 23 个样片逐个出图（尺寸、非黑、亮度），全部按需下载 |
| `preview_test` | 内嵌 preview/thumbnail（X3F/CR2/CR3/NEF/PEF/RAF/RW2 尺寸）与缺失文件报错 |
| `decode_dng_test` | 运行时生成的 CFA DNG：相位跟随裁剪、相位旋转、渲染不退化、黑电平覆盖 |
| `decode_float_test` | 运行时生成的浮点 LinearRaw DNG（16×16 渐变，无 WhiteLevel/DefaultCrop） |
| `demosaic_test` | 2×8 与 12×12 CFA 图案的 full / scaled demosaic |
| `color_conversions_test` | 满量程颜色转换往返 |
| `orientation_test` | 8 种 EXIF 方向的缓冲区改写 |
| `rotatecrop_test` | 裁剪/旋转的尺寸往返（±1px） |
| `roundtrip_test` / `maxsize_test` / `scaling_test` / `curves_test` | 管线往返、限尺寸、缩放、曲线 |

样片：

- 样片不纳入版本库。`res/` 是本地缓存（已加入 `.gitignore`）；测试发现文件缺失或 BLAKE3 校验不符时，会按 `tests/support/mod.rs` 的清单从 [raw.pixls.us](https://raw.pixls.us/) 下载并校验。
- 首次运行需要网络和 `curl`，会下载约 500 MB；可用 `RAWPIPE_SAMPLE_DIR` 指定其他缓存目录。
- 需要特定 DNG 元数据的测试（`decode_dng_test` / `decode_float_test`）在运行时生成极小的合成 DNG（`synthetic_cfa.<pid>.dng`、`synthetic_float.<pid>.dng`），仓库不包含任何样片文件。

## 格式覆盖

当前覆盖 rawler 25 个解码器中的 20 个（下面按扩展名列出，其中 `rwl` 与 `rw2` 共用同一个解码器）：

```
cr2 cr3 dng nef nrw orf pef raf rw2 rwl arw srw mrw crw iiq tfr(3fr) kdc dcr erf mos x3f
```

已覆盖的数据形态：Bayer 2×2（含 BGGR/GRBG）、X-Trans 6×6、单色 DNG（`cpp=1 && !cfa`）、
LinearRaw 3 通道（sRAW CR2）、Foveon（Merrill 12-bit / Quattro 14-bit，含 Quattro 插值）、浮点 DNG、逐通道黑电平、奇数裁剪相位、`crop_area` 缺失、
EXIF 方向（单元测试覆盖全部 8 种，真实文件覆盖 Rotate90）、2×8 / 12×12 CFA（合成图案）。

未覆盖的解码器：`ari`、`dcs`、`mef`、`nkd`、`qtk`。其中 MEF 在 raw.pixls.us 只有非 CC0 样片。

## 待办

按优先级排列，完成后删除对应条目。修复在个人 fork [michael-eddy/dnglab](https://github.com/michael-eddy/dnglab) 上进行并提交；rawpipe 不做 vendored 补丁。

1. **浮点 CFA DNG（`cpp=1` 浮点）未验证** — 需要样片；找到后再判断是否需要上游修复。
2. **rawler 依赖未钉版本** — `Cargo.lock` 被忽略、依赖跟随 fork 的 `main`；修复稳定后应固定 `rev`。
3. **缺失解码器的样片** — `ari` / `dcs` / `nkd` / `qtk` 无 CC0 样片；MEF 样片非 CC0。

## 目录

```
src/
  buffer.rs            交错的 f32 像素缓冲（PixelBuffer）
  colorspaces.rs       相机 profile → XYZ 矩阵、白平衡、to_lab / from_lab
  color_conversions.rs Lab/XYZ/色温/sRGB 转换与查找表
  curves.rs            曝光 + 单调三次样条曲线
  demosaic.rs          Bayer / X-Trans / 宽图案 demosaic
  gamma.rs             sRGB 传输函数
  orientation.rs       EXIF 方向（旋转/镜像）
  rotatecrop.rs        任意角度旋转 + 相对裁剪
  scaling.rs           缩放与重采样
  to_float.rs          裁剪、黑白电平归一化
  pipeline.rs          管线、上下文、缓存、序列化
  hasher.rs            op 配置哈希
tests/                 集成测试
res/                   样片（CC0）
```

## 许可证

本项目采用 **LGPL-2.1-only**（见 [LICENSE](LICENSE) 和 `Cargo.toml`）。依赖的 `rawler` 也是 LGPL-2.1，两者许可一致。

分发（源码或二进制）时需遵守 LGPL-2.1：

- 保留 LICENSE、版权声明与修改说明；修改过的代码继续以 LGPL-2.1 发布。
- 分发二进制需附带对应源码（或等效的书面提供方式）。
- Rust 默认静态链接会触发 LGPL-2.1 §6 的“可重链接”要求：需提供动态库形式，或目标文件等可重新链接的手段。
- 闭源应用可以**动态链接**本库使用；静态链接的闭源二进制同样要满足 §6。

其他说明：

- `res/` 样片：raw.pixls.us 的 CC0 样片；X3F 样片由项目所有者提供。
- dcraw 自 r1.237 起对 Foveon 相关函数有 RESTRICTED 条款；X3F 实现应参考 LibRaw（LGPL-2.1/CDDL）或独立完成，不要直接搬运 dcraw 的 Foveon 代码。
- 当前 `rawler` 走 git 依赖（fork）；若要发布到 crates.io，需改用 crates.io 上的 `rawler 0.8.0` 或等上游发布包含修复的版本（crates.io 不接受 git 依赖）。

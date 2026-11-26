//! Sample files for the integration tests.
//!
//! `res/` is a local cache and is not tracked by git. Missing or mismatched
//! samples are downloaded from raw.pixls.us and verified against their BLAKE3
//! hash. Set `RAWPIPE_SAMPLE_DIR` to use a different cache directory.
//!
//! Tests that need specific DNG metadata generate their own tiny DNG files, so
//! the repository contains no sample images at all.

#![allow(dead_code)]

use std::fs::{self, File};
use std::io::{self, Read};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use rawler::rawimage::RawPhotometricInterpretation;
use rawler::{Orientation, RawImageData};
use rawpipe::{ImageSource, Pipeline, Rotation};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);
static FLOAT_DNG: OnceLock<PathBuf> = OnceLock::new();
static CFA_DNG: OnceLock<PathBuf> = OnceLock::new();

const SAMPLES: &[(&str, &str, &str)] = &[
    ("sample1.3fr", "https://raw.pixls.us/getfile.php/2851/nice/Hasselblad%20-%20H3D%20-%2016bit%20%284:3%29.3FR", "82b4839e5b50fae49cbb4c1e9c44aac2fde64fd3077e16e604c1d7ae79a19ccf"),
    ("sample1.arw", "https://raw.pixls.us/getfile.php/1582/nice/Sony%20-%20ILCE-7S%20-%2014bit%2014bit%20compressed%20%283:2%29.ARW", "c63f305752c789473e6a3505d325d84767a5b7b29a6abef8bed9deb195c750af"),
    ("sample1.cr2", "https://raw.pixls.us/getfile.php/2083/nice/Canon%20-%20EOS%205DS%20-%20RAW%20%283:2%29.CR2", "ef16caeadfec2f01e8d7bb6f12cb130a3d8d067b7201aab86a65c8f01f423e8b"),
    ("sample1.cr3", "https://raw.pixls.us/getfile.php/4656/nice/Canon%20-%20EOS%20M50%20-%203:2.CR3", "71607c2b2ef2d023762842bf752ff81f7c62bccfe731d430c0da978b71da66dd"),
    ("sample1.crw", "https://raw.pixls.us/getfile.php/1307/nice/Canon%20-%20EOS%20D30%20-%20RAW%20%283:2%29.CRW", "d0d8bb7d07c7cd0c74fb77e651325d6ff366e669ec5de5bd840c1668d6a4d37d"),
    ("sample1.dcr", "https://raw.pixls.us/getfile.php/1347/nice/Kodak%20-%20DCS760C%20-%2012bit%20%283:2%29.DCR", "6fb56da23cfb0a6760a28b2f9ff68e25880a6b1360def68c46d91fc5725da5ac"),
    ("sample1.erf", "https://raw.pixls.us/getfile.php/2680/nice/Epson%20-%20R-D1%20-%2012bit%20%283:2%29.ERF", "41f56456b7ad6874fb378eb483bc958b04d1510a010c3c1ff20878e857da2c18"),
    ("sample1.iiq", "https://raw.pixls.us/getfile.php/2765/nice/Phase%20One%20-%20P65%2B%20-%20IIQ%20S%20%284:3%29.IIQ", "8d15fa013b7f7b58d63efa784460f95c9a41dd8be4f70036f8fee8b1b3e7f521"),
    ("sample1.kdc", "https://raw.pixls.us/getfile.php/2345/nice/Kodak%20-%20KODAK%20P712%20ZOOM%20DIGITAL%20CAMERA%20-%20NAN.KDC", "504c688f859f59c8e2b98da5796f8ba4e3a2323789c367b3bbe7174629d4d8b3"),
    ("sample1.mos", "https://raw.pixls.us/getfile.php/3196/nice/Leaf%20-%20Aptus%2022%20-%2016bit%20%284:3%29.mos", "ab9aa28c2152d11bc345910476b1224925fa5d6303244349df254dabd488b2a0"),
    ("sample1.mrw", "https://raw.pixls.us/getfile.php/1826/nice/Minolta%20-%20Dynax%207D%20-%203:2.MRW", "128b7730cd84b59fbedaccf15cd124bdaf815e263a57bdb0fa0bd8051f7c5beb"),
    ("sample1.nef", "https://raw.pixls.us/getfile.php/3516/nice/Nikon%20-%20D3%20-%2012bit%2012bit%20compressed%20%28Lossy%20%28type%202%29%29%20%283:2%29.NEF", "435040bc53afb9c362ebadafbbb5b6d6abcccb90ddc71adc4ca68bf236a30b9d"),
    ("sample1.nrw", "https://raw.pixls.us/getfile.php/585/nice/Nikon%20-%20Coolpix%20P7100%20-%2012bit%2012bit-uncompressed.NRW", "fef3ee90c2e8d3f7e718daa35578aafae219c99b564f36a85c8b1a96be008c0c"),
    ("sample1.orf", "https://raw.pixls.us/getfile.php/2878/nice/Olympus%20-%20E-300%20-%2016bit%20%284:3%29.ORF", "b058cc353a199535ca91f17078b83533952b0bac353499d715965265267ad579"),
    ("sample1.pef", "https://raw.pixls.us/getfile.php/1480/nice/Pentax%20-%20K-5%20-%2014bit%20%283:2%29.PEF", "667fcc1f81612a3103cae8832d2090a36b94ac132ac7839c6f73a27b497a5bcb"),
    ("sample1.raf", "https://raw.pixls.us/getfile.php/2783/nice/Fujifilm%20-%20X-T3%20-%2014bit%2014bit%20compressed%20%283:2%29.RAF", "318b17b8aa59e87fb56dbcce49b26315cc4ba59cd7b8e21868b854bb6ead5fff"),
    ("sample1.rw2", "https://raw.pixls.us/getfile.php/3755/nice/Panasonic%20-%20DMC-FZ70%20-%204:3.RW2", "2de7d6aca739e368c140f3870ea57e0d01fe597b825c3a585cf8f4f58c9097da"),
    ("sample1.rwl", "https://raw.pixls.us/getfile.php/2811/nice/Leica%20-%20D-LUX%205%20-%201:1.RWL", "2f099ed325850be9b09245a86e1f73d2c86d302786a496528757c556e1efad17"),
    ("sample1.srw", "https://raw.pixls.us/getfile.php/3668/nice/Samsung%20-%20NX500%20-%2012bit%2012bit%20normal%20compression%20%283:2%29.SRW", "63fc62b1e25a3f1106704b094a11f3213ffa99a6dada41488d4a57ac25823d77"),
    ("sample1.x3f", "https://raw.pixls.us/getfile.php/855/nice/Sigma%20-%20SIGMA%20DP2%20Merrill%20-%203:2.X3F", "9831d919a81434e21dafffa99cabe2232da5287432cf4928aacc57ddfd92b157"),
    ("sample2.cr2", "https://raw.pixls.us/getfile.php/129/nice/Canon%20-%20EOS%207D%20-%20sRAW2%20%28sRAW%29%20%283:2%29.CR2", "2a3caa56bbf1d06b70b8efc5cd74bdec065a3e243e074928b22af47720d947ac"),
    ("sample2.dng", "https://raw.pixls.us/getfile.php/1087/nice/Leica%20-%20M%20Monochrom%20%28Typ%20246%29%20-%2012bit%20%283:2%29.DNG", "f5a6aaa90b4f36ee37638289c89acc4d30cd60e563a77dc5aa23070e1d96df74"),
    ("sample2.x3f", "https://raw.pixls.us/getfile.php/2494/nice/Sigma%20-%20sd%20Quattro%20H%20-%203:2.X3F", "cf6ba7d07fa45618248d567eedb20cb8c1775b0437acf17699cad6e8d4301ac1"),
    ("sample1.ari", "https://raw.pixls.us/getfile.php/3497/nice/Arri%20-%20Alexa%20Mini.ari", "3953e4c6735758e594e5f5c88a758702ebf4f2dd75e74f8c16411ed77581d05c"),
    ("sample1.crm", "https://8kraw-resources.oss-cn-shanghai.aliyuncs.com/ISO100.CRM", "da239fa4972ac70079eaea7f517a7afc2664cfa2b7a8d350df4a72d9bb8f5271"),
    ("sample1.dcs", "https://raw.pixls.us/getfile.php/2573/nice/Kodak%20-%20DCS520C%20-%2012bit%20%283:2%29.TIF", "92b9b7284ae129b0ec09475a9e879412e283581355f8c91b6ab38db3a7a3b97a"),
    ("sample2.dcs", "https://raw.pixls.us/getfile.php/2465/nice/Kodak%20-%20EOS%20DCS%203%20-%208bit%20%284:3%29.TIF", "db54636adbd05b2ae5bb3a9ae850e04d491d3340a20443b248108e202f1e425c"),
    ("sample1.fff", "https://raw.pixls.us/getfile.php/1640/nice/Hasselblad%20-%20H5D-40%20-%2016bit%20%284:3%29.fff", "890012a018f8ae950d449cd7828e6de999ac786e7d8ccc3cf56ac82c5c566ded"),
    ("sample1.mef", "https://raw.pixls.us/getfile.php/562/nice/Mamiya%20-%20ZD.MEF", "a8f2ef02800563a1bf7d6d7683eaea06db5d9c8109530b684f797cfbbe1bfff9"),
    ("sample1.ori", "https://raw.pixls.us/getfile.php/2857/nice/Olympus%20-%20E-M5%20Mark%20II%20-%2016bit%2016bit%20%22normal%22%20%284:3%29.ORI", "1e978477a0e36b33caf13e58d9eafd073596457c736d206291d169697c5a8bc9"),
    ("sample2.ori", "https://raw.pixls.us/getfile.php/7798/nice/OM%20System%20-%20OM-3%20-%2016bit%20%284:3%29.ORI", "74ab5375bf92a788c22478449d5142a2147c2927a240d16e2c1f76523a07c949"),
    ("sample1.qtk", "https://raw.githubusercontent.com/colinleroy/a2tools/main/src/quicktake/tests/QKTK/Image__1.qtk", "2ab45a0ca8c885e641db78f24f7514ea99a09cf5546f225cf2944213dd354dec"),
    ("sample2.qtk", "https://raw.githubusercontent.com/colinleroy/a2tools/main/src/quicktake/tests/QKTK/IMAGE01.QTK", "7a3eb03b0b665ec5a2615315c80442066fde2dbafa660249b37f19597d128549"),
    ("sample3.qtk", "https://raw.githubusercontent.com/colinleroy/a2tools/main/src/quicktake/tests/QKTN/TEST150.QTK", "dcc276f16c0bbbf379c3502232028323213ce2dd5a54c80dcd9d369fe60715d0"),
    ("sample1.s3is.raw", "https://raw.pixls.us/getfile.php/1765/nice/Canon%20-%20Canon%20PowerShot%20S3IS.CRW", "46ff75398a5c70c60be11b1990e44b02ff619492663fc257ec5ed8937706684e"),
    ("sample2.a720is.raw", "https://raw.pixls.us/getfile.php/2881/nice/Canon%20-%20Powershot%20A720IS.CRW", "a173b2fde24dea3154990f2975963df3068637dd38f1aa87f565b938e38c9b13"),
    ("sample3.d10.raw", "https://raw.pixls.us/getfile.php/1267/nice/Canon%20-%20PowerShot%20D10.CRW", "9dd43fa63b3362123b72f3f664b5355b57c1e51d9a41bb0bdb5e3efe1f9c48be"),
    ("sample4.g7x.raw", "https://raw.pixls.us/getfile.php/1274/nice/Canon%20-%20PowerShot%20G7%20X.CRW", "c635187d3845bd72a4a79b35a4c48955d9e5a1048ed9afe9c87dd74132bf47b0"),
    ("sample5.sx710hs.raw", "https://raw.pixls.us/getfile.php/2926/nice/Canon%20-%20PowerShot%20SX710%20HS%20-%20CHDK%20ver.%201.5.0.CRW", "8a21f8c30393bff5611bf9c6a7a2ea4daf67056b40d7c6c999de56e2e25d2d57"),
    ("sample1.arq", "https://raw.pixls.us/getfile.php/3255/nice/Sony%20-%20ILCE-7RM3%20-%2014bit%2014bit%20uncompressed%20%283:2%29.ARQ", "fe2274dd70d84cb012cb5dac4d011118ceb766789523ac2e2ce18e29a91a8f76"),
    ("sample1.sti", "https://raw.pixls.us/getfile.php/6942/nice/Sinarback%20-%20Sinarback%20eVolution%2075%2C%20Sinar%20p3%20/%20f3%20-%2016bit%20%284:3%29.sti", "f7fcf2577285e20311a0fd9e95cc1bbc8435480e2df99755a5025b060de2dc5f"),
    ("sample1.float.dng", "https://raw.pixls.us/getfile.php/883/nice/Canon%20-%20EOS%205D%20Mark%20III%20-%2032bit%2032bit%20RAW.dng", "ad236d8b3836daa2e9ce001290c5d461a9408e02f4699941fa3b0f0b67abd4f1"),
];

fn manifest_entry(name: &str) -> (&'static str, &'static str) {
    SAMPLES
        .iter()
        .find(|(sample, _, _)| *sample == name)
        .map(|(_, url, hash)| (*url, *hash))
        .unwrap_or_else(|| panic!("no download entry for sample {name}"))
}

fn cache_dir() -> PathBuf {
    std::env::var_os("RAWPIPE_SAMPLE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("res"))
}

fn hash_matches(path: &Path, expected: &str) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut hasher = blake3::Hasher::new();
    if io::copy(&mut file, &mut hasher).is_err() {
        return false;
    }
    hasher.finalize().to_hex().to_string() == expected
}

fn download(name: &str, url: &str, expected: &str, path: &Path) {
    let tmp = path.with_file_name(format!(
        "{name}.part.{}.{}",
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    eprintln!("rawpipe tests: downloading {name} from {url}");
    let status = Command::new("curl")
        .args(["--fail", "--location", "--retry", "3", "--retry-all-errors", "--silent", "--show-error", "--output"])
        .arg(&tmp)
        .arg(url)
        .status()
        .unwrap_or_else(|err| panic!("failed to run curl (install curl or pre-populate res/): {err}"));
    if !status.success() {
        let _ = fs::remove_file(&tmp);
        panic!("downloading {name} from {url} failed: {status}");
    }
    if !hash_matches(&tmp, expected) {
        let _ = fs::remove_file(&tmp);
        panic!("downloaded {name} does not match its expected checksum");
    }
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    fs::rename(&tmp, path).unwrap_or_else(|err| panic!("failed to store {name} at {}: {err}", path.display()));
}

/// Path to a RAW test sample, downloading it into the local cache when needed.
pub fn sample_path(name: &str) -> PathBuf {
    let dir = cache_dir();
    fs::create_dir_all(&dir).unwrap_or_else(|err| panic!("cannot create {}: {err}", dir.display()));

    let path = dir.join(name);
    let (url, expected) = manifest_entry(name);
    if hash_matches(&path, expected) {
        return path;
    }
    download(name, url, expected, &path);
    path
}

// ---------------------------------------------------------------------------
// Synthetic DNGs
// ---------------------------------------------------------------------------

const TYPE_BYTE: u16 = 1;
const TYPE_ASCII: u16 = 2;
const TYPE_SHORT: u16 = 3;
const TYPE_LONG: u16 = 4;
const TYPE_RATIONAL: u16 = 5;
const TYPE_SRATIONAL: u16 = 10;

struct Tag {
    tag: u16,
    typ: u16,
    count: u32,
    data: Vec<u8>,
}

fn tag(tag: u16, typ: u16, count: u32, data: Vec<u8>) -> Tag {
    Tag { tag, typ, count, data }
}

fn ascii(value: &str) -> Vec<u8> {
    let mut bytes = value.as_bytes().to_vec();
    bytes.push(0);
    bytes
}

fn shorts(values: &[u16]) -> Vec<u8> {
    values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

fn rationals(values: &[(u32, u32)]) -> Vec<u8> {
    values.iter().flat_map(|(num, den)| [*num, *den]).flat_map(u32::to_le_bytes).collect()
}

fn srationals(values: &[(i32, i32)]) -> Vec<u8> {
    values.iter().flat_map(|(num, den)| [*num, *den]).flat_map(i32::to_le_bytes).collect()
}

fn identity_color_matrix() -> Vec<u8> {
    srationals(&[(1, 1), (0, 1), (0, 1), (0, 1), (1, 1), (0, 1), (0, 1), (0, 1), (1, 1)])
}

/// Build a little-endian TIFF with one IFD and a single strip of pixels.
fn build_tiff(mut tags: Vec<Tag>, pixels: Vec<u8>) -> Vec<u8> {
    tags.sort_by_key(|tag| tag.tag);
    let ifd_offset = 8usize;
    let ifd_size = 2 + 12 * tags.len() + 4;
    let mut out = vec![0u8; ifd_offset + ifd_size];
    out[0..2].copy_from_slice(b"II");
    out[2..4].copy_from_slice(&42u16.to_le_bytes());
    out[4..8].copy_from_slice(&(ifd_offset as u32).to_le_bytes());

    let mut entries = Vec::new();
    entries.extend_from_slice(&(tags.len() as u16).to_le_bytes());
    let mut strip_offset_position = None;
    for item in &tags {
        let position = entries.len();
        entries.extend_from_slice(&item.tag.to_le_bytes());
        entries.extend_from_slice(&item.typ.to_le_bytes());
        entries.extend_from_slice(&item.count.to_le_bytes());
        if item.tag == 273 {
            strip_offset_position = Some(position + 8);
            entries.extend_from_slice(&[0u8; 4]);
        } else if item.data.len() <= 4 {
            entries.extend_from_slice(&item.data);
            entries.resize(entries.len() + 4 - item.data.len(), 0);
        } else {
            let offset = out.len() as u32;
            out.extend_from_slice(&item.data);
            while out.len() % 4 != 0 {
                out.push(0);
            }
            entries.extend_from_slice(&offset.to_le_bytes());
        }
    }
    entries.extend_from_slice(&[0u8; 4]);

    let pixel_offset = out.len() as u32;
    if let Some(position) = strip_offset_position {
        entries[position..position + 4].copy_from_slice(&pixel_offset.to_le_bytes());
    }
    out.extend_from_slice(&pixels);
    out[ifd_offset..ifd_offset + entries.len()].copy_from_slice(&entries);
    out
}

fn write_generated(name: &str, data: &[u8]) -> PathBuf {
    let dir = cache_dir();
    fs::create_dir_all(&dir).unwrap_or_else(|err| panic!("cannot create {}: {err}", dir.display()));
    // The process id keeps parallel test binaries from sharing one file.
    let path = dir.join(format!("{name}.{}.dng", std::process::id()));
    fs::write(&path, data).unwrap_or_else(|err| panic!("cannot write {}: {err}", path.display()));
    path
}

/// A 16x16 float `LinearRaw` DNG: grayscale gradient, identity colour matrix,
/// no white level and no crop.
pub fn synthetic_float_dng() -> PathBuf {
    FLOAT_DNG.get_or_init(|| synthetic_float_dng_impl()).clone()
}

fn synthetic_float_dng_impl() -> PathBuf {
    const SIZE: usize = 16;
    let mut pixels = Vec::with_capacity(SIZE * SIZE * 3 * 4);
    for _row in 0..SIZE {
        for col in 0..SIZE {
            let value = col as f32 / (SIZE - 1) as f32;
            for _ in 0..3 {
                pixels.extend_from_slice(&value.to_le_bytes());
            }
        }
    }

    let tags = vec![
        tag(256, TYPE_LONG, 1, (SIZE as u32).to_le_bytes().to_vec()),
        tag(257, TYPE_LONG, 1, (SIZE as u32).to_le_bytes().to_vec()),
        tag(258, TYPE_SHORT, 3, shorts(&[32, 32, 32])),
        tag(259, TYPE_SHORT, 1, shorts(&[1])),
        tag(262, TYPE_SHORT, 1, shorts(&[34892])),
        tag(271, TYPE_ASCII, 6, ascii("SIGMA")),
        tag(272, TYPE_ASCII, 16, ascii("Synthetic Float")),
        tag(273, TYPE_LONG, 1, Vec::new()),
        tag(277, TYPE_SHORT, 1, shorts(&[3])),
        tag(278, TYPE_LONG, 1, (SIZE as u32).to_le_bytes().to_vec()),
        tag(279, TYPE_LONG, 1, (pixels.len() as u32).to_le_bytes().to_vec()),
        tag(284, TYPE_SHORT, 1, shorts(&[1])),
        tag(339, TYPE_SHORT, 3, shorts(&[3, 3, 3])),
        tag(50706, TYPE_BYTE, 4, vec![1, 4, 0, 0]),
        tag(50707, TYPE_BYTE, 4, vec![1, 1, 0, 0]),
        tag(50721, TYPE_SRATIONAL, 9, identity_color_matrix()),
        tag(50728, TYPE_RATIONAL, 3, rationals(&[(1, 1), (1, 1), (1, 1)])),
        tag(50778, TYPE_SHORT, 1, shorts(&[21])),
    ];
    write_generated("synthetic_float", &build_tiff(tags, pixels))
}

/// A 64x64 CFA DNG with an odd top crop offset (2, 1), a 0 black level and
/// 4095 white level. Most pixels are dark, a few are bright.
pub fn synthetic_cfa_dng() -> PathBuf {
    CFA_DNG.get_or_init(|| synthetic_cfa_dng_impl()).clone()
}

fn synthetic_cfa_dng_impl() -> PathBuf {
    const SIZE: usize = 64;
    const CROP: (u32, u32) = (2, 1);
    const CROP_SIZE: (u32, u32) = (60, 62);

    let mut pixels = Vec::with_capacity(SIZE * SIZE * 2);
    for row in 0..SIZE {
        for col in 0..SIZE {
            let value: u16 = if row == 32 && col % 16 == 8 {
                4000
            } else {
                100 + (col * 800 / (SIZE - 1)) as u16
            };
            pixels.extend_from_slice(&value.to_le_bytes());
        }
    }

    let tags = vec![
        tag(256, TYPE_LONG, 1, (SIZE as u32).to_le_bytes().to_vec()),
        tag(257, TYPE_LONG, 1, (SIZE as u32).to_le_bytes().to_vec()),
        tag(258, TYPE_SHORT, 1, shorts(&[16])),
        tag(259, TYPE_SHORT, 1, shorts(&[1])),
        tag(262, TYPE_SHORT, 1, shorts(&[32803])),
        tag(271, TYPE_ASCII, 14, ascii("Synthetic CFA")),
        tag(272, TYPE_ASCII, 14, ascii("Synthetic CFA")),
        tag(273, TYPE_LONG, 1, Vec::new()),
        tag(277, TYPE_SHORT, 1, shorts(&[1])),
        tag(278, TYPE_LONG, 1, (SIZE as u32).to_le_bytes().to_vec()),
        tag(279, TYPE_LONG, 1, (pixels.len() as u32).to_le_bytes().to_vec()),
        tag(284, TYPE_SHORT, 1, shorts(&[1])),
        tag(339, TYPE_SHORT, 1, shorts(&[1])),
        tag(33421, TYPE_SHORT, 2, shorts(&[2, 2])),
        tag(33422, TYPE_BYTE, 4, vec![0, 1, 1, 2]),
        tag(50706, TYPE_BYTE, 4, vec![1, 4, 0, 0]),
        tag(50707, TYPE_BYTE, 4, vec![1, 1, 0, 0]),
        tag(50714, TYPE_SHORT, 1, shorts(&[0])),
        tag(50717, TYPE_SHORT, 1, shorts(&[4095])),
        tag(50719, TYPE_SHORT, 2, shorts(&[CROP.0 as u16, CROP.1 as u16])),
        tag(50720, TYPE_SHORT, 2, shorts(&[CROP_SIZE.0 as u16, CROP_SIZE.1 as u16])),
        tag(50721, TYPE_SRATIONAL, 9, identity_color_matrix()),
        tag(50778, TYPE_SHORT, 1, shorts(&[21])),
    ];
    write_generated("synthetic_cfa", &build_tiff(tags, pixels))
}

// ---------------------------------------------------------------------------
// Real RAW sample verification helpers
// ---------------------------------------------------------------------------

pub struct SampleExpectation {
    pub sample: &'static str,
    pub raw_dims: (usize, usize),
    pub out_dims: (usize, usize),
    pub cpp: usize,
    pub is_float: bool,
    pub is_linear_raw: bool,
    pub cfa: &'static str,
    pub black_levels: [f32; 4],
    pub white_levels: [f32; 4],
    pub orientation: Option<Orientation>,
    pub preview: Option<(u32, u32)>,
    pub thumbnail: Option<(u32, u32)>,
    pub scale_max: (usize, usize),
    pub scale_out: (usize, usize),
    pub min_range: u8,
}

fn shift_cfa(pattern: &str, x: usize, y: usize) -> String {
    let chars: Vec<char> = pattern.chars().collect();
    if chars.len() != 4 {
        return pattern.to_string();
    }
    let at = |row: usize, col: usize| chars[((row + y) % 2) * 2 + (col + x) % 2];
    [at(0, 0), at(0, 1), at(1, 0), at(1, 1)].into_iter().collect()
}

fn expected_rotation(orient: Option<Orientation>) -> (Rotation, bool, bool) {
    match orient {
        None | Some(Orientation::Normal) => (Rotation::Normal, false, false),
        Some(Orientation::VerticalFlip) => (Rotation::Normal, false, true),
        Some(Orientation::HorizontalFlip) => (Rotation::Normal, true, false),
        Some(Orientation::Rotate180) => (Rotation::Rotate180, false, false),
        Some(Orientation::Transpose) => (Rotation::Rotate90, false, true),
        Some(Orientation::Rotate90) => (Rotation::Rotate90, false, false),
        Some(Orientation::Rotate270) => (Rotation::Rotate270, false, false),
        Some(Orientation::Transverse) => (Rotation::Rotate270, true, false),
    }
}

fn same_rotation(a: Rotation, b: Rotation) -> bool {
    matches!(
        (a, b),
        (Rotation::Normal, Rotation::Normal)
            | (Rotation::Rotate90, Rotation::Rotate90)
            | (Rotation::Rotate180, Rotation::Rotate180)
            | (Rotation::Rotate270, Rotation::Rotate270)
    )
}

fn assert_matrix_finite_and_nonzero<const R: usize, const C: usize>(
    name: &str,
    sample: &str,
    matrix: &[[f32; C]; R],
) {
    for (row_idx, row) in matrix.iter().enumerate() {
        assert!(
            row.iter().all(|v| v.is_finite()),
            "{sample}: {name}[{row_idx}] has non-finite values: {row:?}"
        );
        if row_idx < 3 {
            assert!(
                row.iter().any(|v| v.abs() > 1e-6),
                "{sample}: {name}[{row_idx}] is all zero: {row:?}"
            );
        }
    }
}

fn assert_image_not_black_or_flat(label: &str, data: &[u8], min_range: u8) {
    assert!(!data.is_empty(), "{label}: empty pixel buffer");
    let min = *data.iter().min().unwrap();
    let max = *data.iter().max().unwrap();
    assert!(
        max - min >= min_range,
        "{label}: flat/black image (min={min}, max={max}, required range>={min_range})"
    );
    let pixels = (data.len() / 3) as u64;
    let lit = data.chunks_exact(3).filter(|p| p.iter().any(|&v| v > 16)).count() as u64;
    assert!(lit * 2 > pixels, "{label}: only {lit} of {pixels} pixels are lit (>16)");

    let mean = data.iter().map(|&v| v as f64).sum::<f64>() / data.len() as f64;
    let variance = data
        .iter()
        .map(|&v| {
            let d = v as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / data.len() as f64;
    assert!(
        variance.sqrt() > 4.0,
        "{label}: image variance too low (mean={mean:.2}, std={:.2})",
        variance.sqrt()
    );
}

fn assert_embedded_image(
    kind: &str,
    sample: &str,
    actual: Option<image::DynamicImage>,
    expected: Option<(u32, u32)>,
) {
    match (actual, expected) {
        (Some(img), Some(exp)) => {
            assert_eq!((img.width(), img.height()), exp, "{sample}: {kind} size");
            let rgb = img.to_rgb8();
            let raw = rgb.as_raw();
            let min = *raw.iter().min().unwrap();
            let max = *raw.iter().max().unwrap();
            assert!(max > min, "{sample}: {kind} is flat ({min}..={max})");
        }
        (None, None) => {}
        (got, exp) => panic!(
            "{sample}: {kind} mismatch: got {:?}, expected {:?}",
            got.map(|i| (i.width(), i.height())),
            exp
        ),
    }
}

/// Call every public file-reading entry point inside `catch_unwind` and ensure
/// none of them panic on malformed input.
pub fn assert_file_ops_do_not_panic(path: &Path, context: &str) {
    for (op, res) in [
        (
            "Pipeline::new_from_file",
            catch_unwind(AssertUnwindSafe(|| Pipeline::new_from_file(path).map(|_| ()))),
        ),
        (
            "decode_8bit",
            catch_unwind(AssertUnwindSafe(|| rawpipe::decode_8bit(path, 320, 320).map(|_| ()))),
        ),
        (
            "preview",
            catch_unwind(AssertUnwindSafe(|| rawpipe::preview(path).map(|_| ()))),
        ),
        (
            "thumbnail",
            catch_unwind(AssertUnwindSafe(|| rawpipe::thumbnail(path).map(|_| ()))),
        ),
    ] {
        assert!(res.is_ok(), "{context}: {op} panicked on {}", path.display());
    }
}

pub fn assert_corrupt_variants_do_not_panic(source_path: &Path, sample: &str) {
    let dir = std::env::temp_dir().join(format!(
        "rawpipe_bad_{}_{}",
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).unwrap();

    let file_len = fs::metadata(source_path).unwrap().len() as usize;
    let prefix_len = file_len.min(512 * 1024);
    let mut prefix = vec![0u8; prefix_len];
    {
        let mut f = File::open(source_path).unwrap();
        f.read_exact(&mut prefix).unwrap();
    }

    // Truncated prefixes: empty, tiny, header-level, and payload-level
    let mut cut_lengths = vec![0usize, 16, 512, 4096];
    if file_len <= 1024 * 1024 {
        cut_lengths.push(file_len / 2);
    } else {
        cut_lengths.push(prefix_len);
    }

    for len in cut_lengths {
        let slice = &prefix[..prefix.len().min(len)];
        let bad = dir.join(format!("{sample}.trunc_{len}.bad"));
        fs::write(&bad, slice).unwrap();
        assert_file_ops_do_not_panic(&bad, &format!("{sample} trunc={len}"));
        if len <= 16 {
            assert!(
                Pipeline::new_from_file(&bad).is_err(),
                "{sample}: trunc={len} should fail to load"
            );
        }
        let _ = fs::remove_file(&bad);
    }

    // Corrupted header bytes on a non-empty prefix
    if prefix.len() >= 64 {
        let mut corrupt = prefix;
        for byte in &mut corrupt[..64] {
            *byte ^= 0xFF;
        }
        let bad = dir.join(format!("{sample}.corrupt_hdr.bad"));
        fs::write(&bad, &corrupt).unwrap();
        assert_file_ops_do_not_panic(&bad, &format!("{sample} corrupt_hdr"));
        let _ = fs::remove_file(&bad);
    }

    let _ = fs::remove_dir_all(&dir);
}

pub fn verify_real_sample(spec: &SampleExpectation) {
    let path = sample_path(spec.sample);

    // 1. Load & verify metadata (CFA, colour matrix, black/white levels, orientation)
    let mut pipeline = Pipeline::new_from_file(&path)
        .unwrap_or_else(|e| panic!("{}: failed to load: {}", spec.sample, e));

    match &pipeline.context.image {
        ImageSource::Raw(raw) => {
            assert_eq!((raw.width, raw.height), spec.raw_dims, "{}: raw dims", spec.sample);
            assert_eq!(raw.cpp, spec.cpp, "{}: cpp", spec.sample);
            assert_eq!(
                matches!(raw.data, RawImageData::Float(_)),
                spec.is_float,
                "{}: float data kind",
                spec.sample
            );
            if spec.is_linear_raw {
                assert!(
                    matches!(raw.photometric, RawPhotometricInterpretation::LinearRaw),
                    "{}: expected LinearRaw, got {:?}",
                    spec.sample,
                    raw.photometric
                );
            } else {
                assert!(
                    matches!(raw.photometric, RawPhotometricInterpretation::Cfa(_)),
                    "{}: expected Cfa photometric, got {:?}",
                    spec.sample,
                    raw.photometric
                );
            }

            assert_eq!(raw.camera.cfa.to_string(), spec.cfa, "{}: CFA", spec.sample);
            assert!(raw.camera.cfa.is_valid(), "{}: CFA should be valid", spec.sample);
            assert!(pipeline.ops.to_float.is_cfa, "{}: to_float.is_cfa", spec.sample);

            let expected_demosaic_cfa = shift_cfa(
                spec.cfa,
                pipeline.ops.to_float.crop_left % 2,
                pipeline.ops.to_float.crop_top % 2,
            );
            assert_eq!(
                pipeline.ops.demosaic.cfa, expected_demosaic_cfa,
                "{}: phase-shifted demosaic CFA",
                spec.sample
            );

            assert!(raw.best_color_matrix().is_some(), "{}: missing best_color_matrix", spec.sample);
            assert_matrix_finite_and_nonzero("cam_to_xyz", spec.sample, &pipeline.ops.to_lab.cam_to_xyz);
            assert_matrix_finite_and_nonzero(
                "cam_to_xyz_normalized",
                spec.sample,
                &pipeline.ops.to_lab.cam_to_xyz_normalized,
            );
            assert_matrix_finite_and_nonzero("xyz_to_cam", spec.sample, &pipeline.ops.to_lab.xyz_to_cam);

            for i in 0..4 {
                assert!(
                    (pipeline.ops.to_float.black_levels[i] - spec.black_levels[i]).abs() < 0.01,
                    "{}: black_levels[{}] = {}, expected {}",
                    spec.sample,
                    i,
                    pipeline.ops.to_float.black_levels[i],
                    spec.black_levels[i]
                );
                assert!(
                    (pipeline.ops.to_float.white_levels[i] - spec.white_levels[i]).abs() < 0.01,
                    "{}: white_levels[{}] = {}, expected {}",
                    spec.sample,
                    i,
                    pipeline.ops.to_float.white_levels[i],
                    spec.white_levels[i]
                );
                if i < spec.cpp.max(1) || !spec.is_linear_raw {
                    assert!(
                        pipeline.ops.to_float.white_levels[i] > pipeline.ops.to_float.black_levels[i],
                        "{}: white <= black at channel {}",
                        spec.sample,
                        i
                    );
                }
            }

            assert_eq!(raw.orientation, spec.orientation, "{}: EXIF orientation", spec.sample);
            let (exp_rot, exp_fh, exp_fv) = expected_rotation(spec.orientation);
            assert!(
                same_rotation(pipeline.ops.orientation.rotation, exp_rot),
                "{}: OpOrientation rotation {:?} != {:?}",
                spec.sample,
                pipeline.ops.orientation.rotation,
                exp_rot
            );
            assert_eq!(
                (
                    pipeline.ops.orientation.flip_horizontal,
                    pipeline.ops.orientation.flip_vertical
                ),
                (exp_fh, exp_fv),
                "{}: orientation flips",
                spec.sample
            );
        }
        ImageSource::Raster(_) => panic!("{}: expected raw image", spec.sample),
    }

    // 2. Full decode: output dimensions + non-black / non-flat
    let image = pipeline
        .output_8bit(None)
        .unwrap_or_else(|e| panic!("{}: output_8bit failed: {}", spec.sample, e));
    assert_eq!((image.width, image.height), spec.out_dims, "{}: output dims", spec.sample);
    assert_eq!(image.data.len(), image.width * image.height * 3, "{}: data len", spec.sample);
    assert_image_not_black_or_flat(spec.sample, &image.data, spec.min_range);

    // 3. Embedded preview & thumbnail
    let pv = rawpipe::preview(&path)
        .unwrap_or_else(|e| panic!("{}: preview failed: {}", spec.sample, e));
    assert_embedded_image("preview", spec.sample, pv, spec.preview);
    let th = rawpipe::thumbnail(&path)
        .unwrap_or_else(|e| panic!("{}: thumbnail failed: {}", spec.sample, e));
    assert_embedded_image("thumbnail", spec.sample, th, spec.thumbnail);

    // 4. decode_8bit max_width / max_height scaling
    let scaled = rawpipe::decode_8bit(&path, spec.scale_max.0, spec.scale_max.1)
        .unwrap_or_else(|e| panic!("{}: decode_8bit scaling failed: {}", spec.sample, e));
    assert_eq!(
        (scaled.width, scaled.height),
        spec.scale_out,
        "{}: scaled dims for max {:?}",
        spec.sample,
        spec.scale_max
    );
    assert!(
        scaled.width <= spec.scale_max.0 && scaled.height <= spec.scale_max.1,
        "{}: scaled {:?} exceeds max {:?}",
        spec.sample,
        (scaled.width, scaled.height),
        spec.scale_max
    );
    assert_eq!(scaled.data.len(), scaled.width * scaled.height * 3);
    assert_image_not_black_or_flat(&format!("{} scaled", spec.sample), &scaled.data, spec.min_range / 2);

    // 5. Malformed / truncated / corrupted files must not panic
    assert_corrupt_variants_do_not_panic(&path, spec.sample);
}

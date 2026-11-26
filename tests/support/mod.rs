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
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

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

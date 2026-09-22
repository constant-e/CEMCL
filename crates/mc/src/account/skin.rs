//! 玩家皮肤与账号头像
//!
//! 头像取皮肤贴图的头部正面（8×8 像素），并按游戏内的做法把第二层（帽子层）叠加到
//! 内层上。默认皮肤的选择与贴图处理都对齐最新 Java 版客户端：
//! `DefaultPlayerSkin.get(UUID)`、`SkinTextureDownloader.processLegacySkin`。

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use log::debug;
use reqwest::Client;
use serde_json::Value;
use std::{io::Cursor, sync::LazyLock, time::Duration};

/// 头像（头部正面）的像素尺寸
const AVATAR_SIZE: u32 = 8;

/// 皮肤贴图宽度，游戏只接受 64 宽
const SKIN_WIDTH: u32 = 64;
/// 旧版皮肤（只有内层）的高度
const LEGACY_SKIN_HEIGHT: u32 = 32;
/// 新版皮肤（内层 + 第二层）的高度
const SKIN_HEIGHT: u32 = 64;

/// 头部内层正面在贴图中的位置
const HEAD_FRONT: (u32, u32) = (8, 8);
/// 头部第二层相对于内层的偏移
const HEAD_OVERLAY_OFFSET: u32 = 32;

/// 默认皮肤名，与 `DefaultPlayerSkin.DEFAULT_SKINS` 前半部分的顺序一致
/// （后半部分是同样的名字、slim 换成 wide，头部像素完全相同，见 `DEFAULT_SKIN_TEXTURES`）
const DEFAULT_SKIN_NAMES: [&str; 9] = [
    "Alex", "Ari", "Efe", "Kai", "Makena", "Noor", "Steve", "Sunny", "Zuri",
];
/// 与 `DEFAULT_SKIN_NAMES` 一一对应的贴图，取自 26.3 客户端
/// `assets/minecraft/textures/entity/player/wide/`
const DEFAULT_SKIN_TEXTURES: [&[u8]; 9] = [
    include_bytes!("../../res/skins/alex.png"),
    include_bytes!("../../res/skins/ari.png"),
    include_bytes!("../../res/skins/efe.png"),
    include_bytes!("../../res/skins/kai.png"),
    include_bytes!("../../res/skins/makena.png"),
    include_bytes!("../../res/skins/noor.png"),
    include_bytes!("../../res/skins/steve.png"),
    include_bytes!("../../res/skins/sunny.png"),
    include_bytes!("../../res/skins/zuri.png"),
];
/// `DefaultPlayerSkin.DEFAULT_SKINS` 的长度（slim、wide 各 9 种）
const DEFAULT_SKIN_COUNT: i32 = 18;
/// 没有皮肤可用时游戏使用的默认皮肤（`DefaultPlayerSkin.getDefaultSkin()`，即 slim/steve）
const FALLBACK_SKIN_INDEX: usize = 6;

/// 默认皮肤头像，与 `DEFAULT_SKIN_NAMES` 一一对应，只在首次使用时解码合成
static DEFAULT_AVATARS: LazyLock<Vec<Avatar>> = LazyLock::new(|| {
    DEFAULT_SKIN_TEXTURES
        .iter()
        .map(|texture| {
            avatar_from_skin(texture)
                .unwrap_or_else(|e| panic!("bundled default skin is invalid: {e}"))
        })
        .collect()
});

/// 复用的http客户端，带整体超时，避免网络卡死时无限等待
static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("failed to build http client")
});

#[derive(Debug)]
pub enum SkinError {
    Base64Error(base64::DecodeError),
    DecodeError(png::DecodingError),
    DeserializationError(serde_json::Error),
    EncodeError(png::EncodingError),
    InvalidSize(u32, u32),
    ReqwestError(reqwest::Error),
    UnsupportedColorType(png::ColorType),
}

impl From<base64::DecodeError> for SkinError {
    fn from(err: base64::DecodeError) -> Self {
        SkinError::Base64Error(err)
    }
}

impl From<png::DecodingError> for SkinError {
    fn from(err: png::DecodingError) -> Self {
        SkinError::DecodeError(err)
    }
}

impl From<png::EncodingError> for SkinError {
    fn from(err: png::EncodingError) -> Self {
        SkinError::EncodeError(err)
    }
}

impl From<reqwest::Error> for SkinError {
    fn from(err: reqwest::Error) -> Self {
        SkinError::ReqwestError(err)
    }
}

impl From<serde_json::Error> for SkinError {
    fn from(err: serde_json::Error) -> Self {
        SkinError::DeserializationError(err)
    }
}

impl std::fmt::Display for SkinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkinError::Base64Error(e) => write!(f, "{e}"),
            SkinError::DecodeError(e) => write!(f, "{e}"),
            SkinError::DeserializationError(e) => write!(f, "{e}"),
            SkinError::EncodeError(e) => write!(f, "{e}"),
            SkinError::InvalidSize(width, height) => {
                write!(f, "Invalid image size {width}x{height}")
            }
            SkinError::ReqwestError(e) => write!(f, "{e}"),
            SkinError::UnsupportedColorType(color_type) => {
                write!(f, "Unsupported color type {color_type:?}")
            }
        }
    }
}

/// 账号头像：皮肤头部正面（8×8 的 RGBA8 位图）
#[derive(Clone)]
pub struct Avatar {
    rgba: Vec<u8>,
}

impl Avatar {
    /// 位图边长
    pub const SIZE: u32 = AVATAR_SIZE;

    pub fn into_rgba(self) -> Vec<u8> {
        self.rgba
    }

    /// 解码缓存的头像
    pub fn from_png(png: &[u8]) -> Result<Self, SkinError> {
        let bitmap = Bitmap::decode(png)?;
        if bitmap.width != AVATAR_SIZE || bitmap.height != AVATAR_SIZE {
            return Err(SkinError::InvalidSize(bitmap.width, bitmap.height));
        }
        Ok(Self { rgba: bitmap.rgba })
    }

    /// 编码头像，用于缓存
    pub fn to_png(&self) -> Result<Vec<u8>, SkinError> {
        Bitmap {
            height: AVATAR_SIZE,
            rgba: self.rgba.clone(),
            width: AVATAR_SIZE,
        }
        .encode()
    }
}

/// 默认皮肤的名字（Steve、Alex 等），取法与游戏内一致；uuid 无效时返回游戏默认皮肤的名字
pub fn default_skin_name(uuid: &str) -> &'static str {
    match default_skin_index(uuid) {
        Some(index) => DEFAULT_SKIN_NAMES[index % DEFAULT_SKIN_NAMES.len()],
        None => DEFAULT_SKIN_NAMES[FALLBACK_SKIN_INDEX],
    }
}

/// uuid 对应的默认皮肤头像，取法与游戏内一致；uuid 无效时返回游戏默认皮肤的头像
pub fn default_avatar(uuid: &str) -> Avatar {
    match default_skin_index(uuid) {
        Some(index) => DEFAULT_AVATARS[index % DEFAULT_AVATARS.len()].clone(),
        None => fallback_avatar(),
    }
}

/// 没有皮肤可用时游戏使用的默认头像（Steve）
pub fn fallback_avatar() -> Avatar {
    DEFAULT_AVATARS[FALLBACK_SKIN_INDEX].clone()
}

/// 从皮肤贴图取头像，只接受游戏支持的尺寸（64×32 与 64×64），其余视为无效皮肤
pub fn avatar_from_skin(png: &[u8]) -> Result<Avatar, SkinError> {
    let mut skin = Bitmap::decode(png)?;
    if skin.width != SKIN_WIDTH || (skin.height != LEGACY_SKIN_HEIGHT && skin.height != SKIN_HEIGHT)
    {
        return Err(SkinError::InvalidSize(skin.width, skin.height));
    }

    // 旧版皮肤没有第二层，贴图右半部分是编辑器留下的占位数据
    if skin.height == LEGACY_SKIN_HEIGHT {
        drop_legacy_overlay(&mut skin);
    }
    // 内层一律不透明
    force_head_opaque(&mut skin);

    Ok(Avatar {
        rgba: head_front(&skin),
    })
}

/// 查询玩家的皮肤贴图，`Ok(None)` 表示该玩家没有自定义皮肤（游戏内会退回默认皮肤）
pub async fn request_skin(uuid: &str) -> Result<Option<Vec<u8>>, SkinError> {
    let url = format!(
        "https://sessionserver.mojang.com/session/minecraft/profile/{}",
        uuid.replace('-', "")
    );

    debug!("Request skin of {uuid}");
    let res = CLIENT.get(url).send().await?;
    // 204：这个 uuid 没有对应的玩家
    if res.status() == reqwest::StatusCode::NO_CONTENT {
        return Ok(None);
    }
    let json = serde_json::from_str::<Value>(&res.error_for_status()?.text().await?)?;

    let Some(value) = json["properties"]
        .as_array()
        .and_then(|properties| {
            properties
                .iter()
                .find(|property| property["name"] == "textures")
        })
        .and_then(|property| property["value"].as_str())
    else {
        return Ok(None);
    };
    let textures = serde_json::from_slice::<Value>(&BASE64.decode(value)?)?;

    let Some(url) = textures["textures"]["SKIN"]["url"].as_str() else {
        return Ok(None);
    };
    // 皮肤服务给出的地址是 http
    let url = url.replacen("http://", "https://", 1);

    let texture = CLIENT.get(url).send().await?.error_for_status()?;
    let skin = texture.bytes().await?.to_vec();

    debug!("Finish skin of {uuid}");
    Ok(Some(skin))
}

/// `DefaultPlayerSkin.DEFAULT_SKINS` 中的下标
fn default_skin_index(uuid: &str) -> Option<usize> {
    let uuid = uuid::Uuid::parse_str(uuid).ok()?;
    let (most, least) = uuid.as_u64_pair();
    // 与 java.util.UUID.hashCode() 一致：(int) (hilo >> 32) ^ (int) hilo
    let hash = ((most ^ least) >> 32) as u32 ^ (most ^ least) as u32;

    Some((hash as i32).rem_euclid(DEFAULT_SKIN_COUNT) as usize)
}

/// 旧版皮肤的第二层是编辑器留下的占位数据：整块都不透明时游戏会将其清除，否则原样保留
/// （`doNotchTransparencyHack`）
fn drop_legacy_overlay(skin: &mut Bitmap) {
    let is_garbage = (0..LEGACY_SKIN_HEIGHT)
        .all(|y| (SKIN_WIDTH / 2..SKIN_WIDTH).all(|x| skin.pixel(x, y)[3] >= 128));
    if !is_garbage {
        return;
    }

    for y in 0..LEGACY_SKIN_HEIGHT {
        for x in SKIN_WIDTH / 2..SKIN_WIDTH {
            let [r, g, b, _] = skin.pixel(x, y);
            skin.set_pixel(x, y, [r, g, b, 0]);
        }
    }
}

/// 头部内层一律强制不透明，保留 RGB（`setNoAlpha(image, 0, 0, 32, 16)`）
fn force_head_opaque(skin: &mut Bitmap) {
    for y in 0..HEAD_FRONT.1 + AVATAR_SIZE {
        for x in 0..SKIN_WIDTH / 2 {
            let [r, g, b, _] = skin.pixel(x, y);
            skin.set_pixel(x, y, [r, g, b, 255]);
        }
    }
}

/// 头部正面：内层与第二层正向合成
fn head_front(skin: &Bitmap) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((AVATAR_SIZE * AVATAR_SIZE * 4) as usize);
    for y in 0..AVATAR_SIZE {
        for x in 0..AVATAR_SIZE {
            let base = skin.pixel(HEAD_FRONT.0 + x, HEAD_FRONT.1 + y);
            let overlay = skin.pixel(HEAD_FRONT.0 + HEAD_OVERLAY_OFFSET + x, HEAD_FRONT.1 + y);
            rgba.extend_from_slice(&blend(base, overlay));
        }
    }
    rgba
}

/// 第二层按源 alpha 叠加到内层上，与游戏渲染一致
fn blend(base: [u8; 4], overlay: [u8; 4]) -> [u8; 4] {
    let alpha = overlay[3] as u32;
    if alpha == 0 {
        return base;
    }
    if alpha == 255 {
        return overlay;
    }

    let mix =
        |src: u8, dst: u8| ((src as u32 * alpha + dst as u32 * (255 - alpha) + 127) / 255) as u8;

    [
        mix(overlay[0], base[0]),
        mix(overlay[1], base[1]),
        mix(overlay[2], base[2]),
        // 内层不透明，合成结果也一定不透明
        255,
    ]
}

/// RGBA8 位图
struct Bitmap {
    height: u32,
    rgba: Vec<u8>,
    width: u32,
}

impl Bitmap {
    /// 解码 PNG，统一成 8 位 RGBA
    fn decode(png: &[u8]) -> Result<Self, SkinError> {
        let mut decoder = png::Decoder::new(Cursor::new(png));
        decoder.set_transformations(
            png::Transformations::normalize_to_color8() | png::Transformations::ALPHA,
        );
        let mut reader = decoder.read_info()?;
        let mut buf = vec![0; reader.output_buffer_size().unwrap_or_default()];
        let info = reader.next_frame(&mut buf)?;

        let (width, height) = (info.width, info.height);
        let rgba = match info.color_type {
            png::ColorType::Rgba => {
                let mut rgba = Vec::with_capacity((width * height * 4) as usize);
                for line in buf[..info.buffer_size()].chunks_exact(info.line_size) {
                    rgba.extend_from_slice(&line[..width as usize * 4]);
                }
                rgba
            }
            png::ColorType::GrayscaleAlpha => {
                let mut rgba = Vec::with_capacity((width * height * 4) as usize);
                for line in buf[..info.buffer_size()].chunks_exact(info.line_size) {
                    for pixel in line[..width as usize * 2].chunks_exact(2) {
                        rgba.extend_from_slice(&[pixel[0], pixel[0], pixel[0], pixel[1]]);
                    }
                }
                rgba
            }
            color_type => return Err(SkinError::UnsupportedColorType(color_type)),
        };

        Ok(Self {
            height,
            rgba,
            width,
        })
    }

    /// 编码成 PNG
    fn encode(&self) -> Result<Vec<u8>, SkinError> {
        let mut png = Vec::new();
        let mut encoder = png::Encoder::new(&mut png, self.width, self.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header()?.write_image_data(&self.rgba)?;
        Ok(png)
    }

    fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let offset = ((y * self.width + x) * 4) as usize;
        self.rgba[offset..offset + 4].try_into().unwrap()
    }

    fn set_pixel(&mut self, x: u32, y: u32, pixel: [u8; 4]) {
        let offset = ((y * self.width + x) * 4) as usize;
        self.rgba[offset..offset + 4].copy_from_slice(&pixel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 按给定颜色函数生成一张皮肤贴图
    fn build_skin(width: u32, height: u32, pixel: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
        let mut bitmap = Bitmap {
            height,
            rgba: vec![0; (width * height * 4) as usize],
            width,
        };
        for y in 0..height {
            for x in 0..width {
                bitmap.set_pixel(x, y, pixel(x, y));
            }
        }
        bitmap.encode().unwrap()
    }

    /// 贴图是否每一格都是同一个颜色
    fn is_uniform(avatar: &Avatar) -> bool {
        avatar
            .rgba
            .chunks_exact(4)
            .all(|pixel| pixel == &avatar.rgba[..4])
    }

    /// 默认皮肤名取自 `DefaultPlayerSkin.get(UUID)`（向量由 26.3 客户端的算法在 JDK 上算出）
    #[test]
    fn default_skin_name_matches_the_game() {
        let cases = [
            ("069a79f4-44e9-4726-a5be-fca90e38aaf5", "Alex"),
            ("853c80ef-3c37-49fd-aa49-938b674adae6", "Ari"),
            ("00000000-0000-0000-0000-000000000000", "Alex"),
            ("9c2f2c1a-6a4e-4f31-9b58-6d7b3d9c1f42", "Sunny"),
            ("1f4a3d2b-8c9e-47a5-b6d3-0e5c7a91d284", "Noor"),
            // 不带连字符、大小写不同的写法是同一个 uuid
            ("9C2F2C1A6A4E4F319B586D7B3D9C1F42", "Sunny"),
            // 无效 uuid 用游戏默认皮肤
            ("not-a-uuid", "Steve"),
        ];
        for (uuid, name) in cases {
            assert_eq!(default_skin_name(uuid), name, "{uuid}");
        }
    }

    /// 每个账号都能拿到一张不透明、有内容的头像
    #[test]
    fn avatars_are_opaque_and_not_blank() {
        for name in DEFAULT_SKIN_NAMES {
            let index = DEFAULT_SKIN_NAMES.iter().position(|n| *n == name).unwrap();
            let avatar = DEFAULT_AVATARS[index].clone();
            assert_eq!(
                avatar.rgba.len(),
                (AVATAR_SIZE * AVATAR_SIZE * 4) as usize,
                "{name}"
            );
            assert!(avatar.rgba.chunks_exact(4).all(|p| p[3] == 255), "{name}");
            assert!(!is_uniform(&avatar), "{name}");

            // 缓存一次往返后内容不变
            let cached = Avatar::from_png(&avatar.to_png().unwrap()).unwrap();
            assert_eq!(cached.rgba, avatar.rgba, "{name}");
        }
    }

    /// 第二层按源 alpha 叠加在内层上；不透明的第二层完全盖住内层
    #[test]
    fn avatar_composites_head_overlay() {
        let base = [10, 20, 30, 255];
        let head_base = |x: u32, y: u32| (8..16).contains(&x) && (8..16).contains(&y);
        let head_overlay = |x: u32, y: u32| (40..48).contains(&x) && (8..16).contains(&y);

        // 半透明第二层：(200 * 128 + 10 * 127 + 127) / 255 = 105，其余通道同理
        let skin = build_skin(SKIN_WIDTH, SKIN_HEIGHT, |x, y| {
            match (head_base(x, y), head_overlay(x, y)) {
                (true, _) => base,
                (_, true) => [200, 100, 50, 128],
                _ => [0, 0, 0, 0],
            }
        });
        let avatar = avatar_from_skin(&skin).unwrap();
        assert!(avatar.rgba.chunks_exact(4).all(|p| p == [105, 60, 40, 255]));

        // 不透明的第二层
        let skin = build_skin(SKIN_WIDTH, SKIN_HEIGHT, |x, y| {
            match (head_base(x, y), head_overlay(x, y)) {
                (true, _) => base,
                (_, true) => [200, 100, 50, 255],
                _ => [0, 0, 0, 0],
            }
        });
        let avatar = avatar_from_skin(&skin).unwrap();
        assert!(
            avatar
                .rgba
                .chunks_exact(4)
                .all(|p| p == [200, 100, 50, 255])
        );
    }

    /// 内层透明时游戏会强制不透明，第二层完全透明时露出内层
    #[test]
    fn avatar_forces_head_opaque() {
        let skin = build_skin(SKIN_WIDTH, SKIN_HEIGHT, |x, y| {
            if (8..16).contains(&x) && (8..16).contains(&y) {
                [10, 20, 30, 0]
            } else {
                [0, 0, 0, 0]
            }
        });
        let avatar = avatar_from_skin(&skin).unwrap();
        assert!(avatar.rgba.chunks_exact(4).all(|p| p == [10, 20, 30, 255]));
    }

    /// 旧版皮肤（64×32）右半部分是占位数据：整块不透明时清除，否则原样保留
    #[test]
    fn legacy_skin_overlay() {
        let garbage = |x: u32, y: u32| {
            if (8..16).contains(&x) && (8..16).contains(&y) {
                [10, 20, 30, 255]
            } else if x >= SKIN_WIDTH / 2 && y < LEGACY_SKIN_HEIGHT {
                [255, 0, 255, 255]
            } else {
                [0, 0, 0, 0]
            }
        };
        let avatar =
            avatar_from_skin(&build_skin(SKIN_WIDTH, LEGACY_SKIN_HEIGHT, garbage)).unwrap();
        assert!(avatar.rgba.chunks_exact(4).all(|p| p == [10, 20, 30, 255]));

        // 右半部分存在半透明像素时游戏不做处理，占位数据会盖住头部正面
        let kept = |x: u32, y: u32| {
            if x == 32 && y == 0 {
                [0, 0, 0, 127]
            } else {
                garbage(x, y)
            }
        };
        let avatar = avatar_from_skin(&build_skin(SKIN_WIDTH, LEGACY_SKIN_HEIGHT, kept)).unwrap();
        assert!(avatar.rgba.chunks_exact(4).all(|p| p == [255, 0, 255, 255]));
    }

    /// 只接受游戏支持的贴图尺寸
    #[test]
    fn avatar_rejects_unsupported_sizes() {
        for (width, height) in [(128, 128), (64, 16), (32, 32)] {
            let skin = build_skin(width, height, |_, _| [10, 20, 30, 255]);
            assert!(
                matches!(avatar_from_skin(&skin), Err(SkinError::InvalidSize(..))),
                "{width}x{height}"
            );
        }
    }
}

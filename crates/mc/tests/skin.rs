//! 账号头像：默认皮肤选择、头部合成与打包的默认皮肤贴图

use mc::account::skin::{Avatar, SkinError, avatar_from_skin, default_skin_name};

/// 皮肤贴图宽度，游戏只接受 64 宽
const SKIN_WIDTH: u32 = 64;
/// 旧版皮肤（只有内层）的高度
const LEGACY_SKIN_HEIGHT: u32 = 32;
/// 新版皮肤（内层 + 第二层）的高度
const SKIN_HEIGHT: u32 = 64;

/// 按给定颜色函数生成一张皮肤贴图
fn build_skin(width: u32, height: u32, pixel: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            rgba.extend_from_slice(&pixel(x, y));
        }
    }
    let mut png = Vec::new();
    let mut encoder = png::Encoder::new(&mut png, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&rgba)
        .unwrap();
    png
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

/// 打包的默认皮肤都能得到不透明、有内容的头像，缓存往返后内容不变
#[test]
fn bundled_default_skins_are_valid() {
    for name in [
        "alex", "ari", "efe", "kai", "makena", "noor", "steve", "sunny", "zuri",
    ] {
        let png = std::fs::read(format!(
            "{}/res/skins/{name}.png",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let avatar = avatar_from_skin(&png).unwrap_or_else(|e| panic!("{name}: {e}"));
        let rgba = avatar.clone().into_rgba();
        assert_eq!(
            rgba.len(),
            (Avatar::SIZE * Avatar::SIZE * 4) as usize,
            "{name}"
        );
        assert!(rgba.chunks_exact(4).all(|p| p[3] == 255), "{name}");
        assert!(!rgba.chunks_exact(4).all(|p| p == &rgba[..4]), "{name}");

        let cached = Avatar::from_png(&avatar.to_png().unwrap()).unwrap();
        assert_eq!(cached.into_rgba(), rgba, "{name}");
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
    assert!(
        avatar
            .clone()
            .into_rgba()
            .chunks_exact(4)
            .all(|p| p == [105, 60, 40, 255])
    );

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
            .into_rgba()
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
    assert!(
        avatar
            .into_rgba()
            .chunks_exact(4)
            .all(|p| p == [10, 20, 30, 255])
    );
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
    let avatar = avatar_from_skin(&build_skin(SKIN_WIDTH, LEGACY_SKIN_HEIGHT, garbage)).unwrap();
    assert!(
        avatar
            .into_rgba()
            .chunks_exact(4)
            .all(|p| p == [10, 20, 30, 255])
    );

    // 右半部分存在半透明像素时游戏不做处理，占位数据会盖住头部正面
    let kept = |x: u32, y: u32| {
        if x == 32 && y == 0 {
            [0, 0, 0, 127]
        } else {
            garbage(x, y)
        }
    };
    let avatar = avatar_from_skin(&build_skin(SKIN_WIDTH, LEGACY_SKIN_HEIGHT, kept)).unwrap();
    assert!(
        avatar
            .into_rgba()
            .chunks_exact(4)
            .all(|p| p == [255, 0, 255, 255])
    );
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

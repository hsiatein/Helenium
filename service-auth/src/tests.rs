use ed25519_dalek::SigningKey;
use ed25519_dalek::VerifyingKey;
use ed25519_dalek::ed25519::signature::SignerMut;
use ed25519_dalek::pkcs8::DecodePrivateKey;
use ed25519_dalek::pkcs8::DecodePublicKey;
use ed25519_dalek::pkcs8::EncodePrivateKey;
use ed25519_dalek::pkcs8::EncodePublicKey;
use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;
use rand::rngs::OsRng;

use crate::generate_challenge;

#[test]
fn test_generate_key() {
    let mut csprng = OsRng;
    let mut signing_key: SigningKey = SigningKey::generate(&mut csprng);
    println!("{:?}", signing_key);
    let pem = signing_key
        .to_pkcs8_pem(LineEnding::LF)
        .expect("转pem失败")
        .to_string();
    let verify = signing_key.verifying_key();
    let pub_key = verify.to_public_key_pem(LineEnding::LF).expect("转pem失败");
    println!("{}", pem);
    println!("{}", pub_key);
    let challenge = generate_challenge();
    let signature = signing_key.sign(&challenge);
    verify
        .verify_strict(&challenge, &signature)
        .expect("验证失败");
    let _doc = signing_key
        .to_pkcs8_der()
        .expect("生成失败")
        .write_der_file("./key.der");
    // println!("{:?}",doc);
}

#[test]
fn test_read_key() {
    dotenvy::dotenv().ok();
    let pub_key_str = "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAezkZozB1KXNmhUxch+lYqWnO3Bwrh1smaFogHa4PTr0=\n-----END PUBLIC KEY-----";
    let pri_key_path = std::env::var("HELENIUM_KEY").expect("没有设置HELENIUM_KEY");
    let pri_key_str = std::fs::read_to_string(pri_key_path)
        .expect("没有设置HELENIUM_KEY")
        .trim()
        .replace("\r\n", "\n");
    let key = SigningKey::from_pkcs8_pem(&pri_key_str)
        .map_err(|e| {
            // 打印详细的错误，pkcs8 的错误信息通常比 dalek 的更细
            eprintln!("PKCS8 详细错误: {:?}", e);
            e
        })
        .expect("解析失败");
    println!("key: {:?}", key);
    println!("{:?}", pri_key_str);
    let mut signing_key: SigningKey =
        SigningKey::from_pkcs8_pem(&pri_key_str).expect("SigningKey转换失败");
    let pub_key: VerifyingKey =
        VerifyingKey::from_public_key_pem(&pub_key_str).expect("VerifyingKey转换失败");
    let challenge = generate_challenge();
    let signature = signing_key.sign(&challenge);
    pub_key
        .verify_strict(&challenge, &signature)
        .expect("验证失败");
}

#[test]
fn test_read_key_loopback() {
    use ed25519_dalek::pkcs8::DecodePrivateKey;
    use ed25519_dalek::pkcs8::EncodePrivateKey;
    use rand::rngs::OsRng;

    // 1. 生成
    let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);

    // 2. 序列化
    let pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();

    // --- 打印出来观察长度 ---
    let base64_part = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect::<String>();
    println!("Base64 纯内容长度: {}", base64_part.len());
    println!("Base64 纯内容: {}", base64_part);
    // ----------------------

    // 3. 立即反序列化 (不要经过任何手动复制)
    let recovered_key = ed25519_dalek::SigningKey::from_pkcs8_pem(&pem).expect("闭环解析失败");

    assert_eq!(signing_key.to_bytes(), recovered_key.to_bytes());
}

#[test]
fn test_key_file_io_loopback() {
    use ed25519_dalek::pkcs8::DecodePrivateKey;
    use ed25519_dalek::pkcs8::EncodePrivateKey;
    use std::fs;
    use std::io::Read;

    // 1. 获取路径（模拟你的环境变量逻辑）
    // 为了测试安全，如果环境变量不存在，我们暂存到当前目录
    let pri_key_path =
        std::env::var("HELENIUM_KEY").unwrap_or_else(|_| "test_ed25519.key".to_string());

    // 2. 生成标准的 SigningKey
    let original_key = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
    println!(
        "{}",
        original_key
            .verifying_key()
            .to_public_key_pem(LineEnding::LF)
            .unwrap()
    );
    // 3. 序列化为 PEM
    // 注意：显式指定 LineEnding::LF (Unix风格)，这是最不容易出问题的格式
    let pem_to_save = original_key.to_pkcs8_pem(LineEnding::LF).unwrap();

    // 4. 保存到文件
    fs::write(&pri_key_path, pem_to_save.as_bytes()).expect("无法写入密钥文件");

    // 5. 从文件读取
    // 技巧：使用 fs::read_to_string 会自动处理 UTF-8 校验
    let mut loaded_content = String::new();
    fs::File::open(&pri_key_path)
        .expect("无法打开文件")
        .read_to_string(&mut loaded_content)
        .expect("无法读取文件内容");

    // 6. 【关键步骤】清洗数据
    // 这一步能解决 90% 的 Base64 InvalidEncoding 问题
    let clean_pem = loaded_content.trim();

    // 7. 解析
    let recovered_key = ed25519_dalek::SigningKey::from_pkcs8_pem(clean_pem)
        .map_err(|e| {
            eprintln!("解析失败！路径: {}", pri_key_path);
            eprintln!("读取到的长度: {}", clean_pem.len());
            eprintln!("读取到的内容预览: {:?}", clean_pem); // 查看是否有隐藏的 \r 或 \0
            e
        })
        .expect("从文件加载并解析 SigningKey 失败");

    // 8. 验证一致性
    assert_eq!(original_key.to_bytes(), recovered_key.to_bytes());
    println!("密钥闭环测试成功！路径: {}", pri_key_path);
}

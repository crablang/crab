#![feature(stdsimd)]
#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

extern crate cupid;
#[macro_use]
extern crate std_detect;

#[test]
fn dump() {
    println!("aes: {:?}", is_x86_feature_detected!("aes"));
    println!("pclmulqdq: {:?}", is_x86_feature_detected!("pclmulqdq"));
    println!("rdrand: {:?}", is_x86_feature_detected!("rdrand"));
    println!("rdseed: {:?}", is_x86_feature_detected!("rdseed"));
    println!("tsc: {:?}", is_x86_feature_detected!("tsc"));
    println!("sse: {:?}", is_x86_feature_detected!("sse"));
    println!("sse2: {:?}", is_x86_feature_detected!("sse2"));
    println!("sse3: {:?}", is_x86_feature_detected!("sse3"));
    println!("ssse3: {:?}", is_x86_feature_detected!("ssse3"));
    println!("sse4.1: {:?}", is_x86_feature_detected!("sse4.1"));
    println!("sse4.2: {:?}", is_x86_feature_detected!("sse4.2"));
    println!("sse4a: {:?}", is_x86_feature_detected!("sse4a"));
    println!("sha: {:?}", is_x86_feature_detected!("sha"));
    println!("f16c: {:?}", is_x86_feature_detected!("f16c"));
    println!("avx: {:?}", is_x86_feature_detected!("avx"));
    println!("avx2: {:?}", is_x86_feature_detected!("avx2"));
    println!("avx512f {:?}", is_x86_feature_detected!("avx512f"));
    println!("avx512cd {:?}", is_x86_feature_detected!("avx512cd"));
    println!("avx512er {:?}", is_x86_feature_detected!("avx512er"));
    println!("avx512pf {:?}", is_x86_feature_detected!("avx512pf"));
    println!("avx512bw {:?}", is_x86_feature_detected!("avx512bw"));
    println!("avx512dq {:?}", is_x86_feature_detected!("avx512dq"));
    println!("avx512vl {:?}", is_x86_feature_detected!("avx512vl"));
    println!("avx512_ifma {:?}", is_x86_feature_detected!("avx512ifma"));
    println!("avx512vbmi {:?}", is_x86_feature_detected!("avx512vbmi"));
    println!(
        "avx512_vpopcntdq {:?}",
        is_x86_feature_detected!("avx512vpopcntdq")
    );
    println!("avx512vbmi2 {:?}", is_x86_feature_detected!("avx512vbmi2"));
    println!("gfni {:?}", is_x86_feature_detected!("gfni"));
    println!("vaes {:?}", is_x86_feature_detected!("vaes"));
    println!("vpclmulqdq {:?}", is_x86_feature_detected!("vpclmulqdq"));
    println!("avx512vnni {:?}", is_x86_feature_detected!("avx512vnni"));
    println!(
        "avx512bitalg {:?}",
        is_x86_feature_detected!("avx512bitalg")
    );
    println!("avx512bf16 {:?}", is_x86_feature_detected!("avx512bf16"));
    println!(
        "avx512vp2intersect {:?}",
        is_x86_feature_detected!("avx512vp2intersect")
    );
    println!("fma: {:?}", is_x86_feature_detected!("fma"));
    println!("abm: {:?}", is_x86_feature_detected!("abm"));
    println!("bmi: {:?}", is_x86_feature_detected!("bmi1"));
    println!("bmi2: {:?}", is_x86_feature_detected!("bmi2"));
    println!("tbm: {:?}", is_x86_feature_detected!("tbm"));
    println!("popcnt: {:?}", is_x86_feature_detected!("popcnt"));
    println!("lzcnt: {:?}", is_x86_feature_detected!("lzcnt"));
    println!("fxsr: {:?}", is_x86_feature_detected!("fxsr"));
    println!("xsave: {:?}", is_x86_feature_detected!("xsave"));
    println!("xsaveopt: {:?}", is_x86_feature_detected!("xsaveopt"));
    println!("xsaves: {:?}", is_x86_feature_detected!("xsaves"));
    println!("xsavec: {:?}", is_x86_feature_detected!("xsavec"));
    println!("cmpxchg16b: {:?}", is_x86_feature_detected!("cmpxchg16b"));
    println!("adx: {:?}", is_x86_feature_detected!("adx"));
    println!("rtm: {:?}", is_x86_feature_detected!("rtm"));
    println!("movbe: {:?}", is_x86_feature_detected!("movbe"));
}

#[cfg(feature = "std_detect_env_override")]
#[test]
fn env_override_no_avx() {
    if let Ok(disable) = std::env::var("RUST_STD_DETECT_UNSTABLE") {
        let information = cupid::master().unwrap();
        for d in disable.split(" ") {
            match d {
                "avx" => {
                    if information.avx() {
                        assert_ne!(is_x86_feature_detected!("avx"), information.avx())
                    }
                }
                "avx2" => {
                    if information.avx2() {
                        assert_ne!(is_x86_feature_detected!("avx2"), information.avx2())
                    }
                }
                _ => {}
            }
        }
    }
}

#[test]
fn compare_with_cupid() {
    let information = cupid::master().unwrap();
    assert_eq!(is_x86_feature_detected!("aes"), information.aesni());
    assert_eq!(
        is_x86_feature_detected!("pclmulqdq"),
        information.pclmulqdq()
    );
    assert_eq!(is_x86_feature_detected!("rdrand"), information.rdrand());
    assert_eq!(is_x86_feature_detected!("rdseed"), information.rdseed());
    assert_eq!(is_x86_feature_detected!("tsc"), information.tsc());
    assert_eq!(is_x86_feature_detected!("sse"), information.sse());
    assert_eq!(is_x86_feature_detected!("sse2"), information.sse2());
    assert_eq!(is_x86_feature_detected!("sse3"), information.sse3());
    assert_eq!(is_x86_feature_detected!("ssse3"), information.ssse3());
    assert_eq!(is_x86_feature_detected!("sse4.1"), information.sse4_1());
    assert_eq!(is_x86_feature_detected!("sse4.2"), information.sse4_2());
    assert_eq!(is_x86_feature_detected!("sse4a"), information.sse4a());
    assert_eq!(is_x86_feature_detected!("sha"), information.sha());
    assert_eq!(is_x86_feature_detected!("f16c"), information.f16c());
    assert_eq!(is_x86_feature_detected!("avx"), information.avx());
    assert_eq!(is_x86_feature_detected!("avx2"), information.avx2());
    assert_eq!(is_x86_feature_detected!("avx512f"), information.avx512f());
    assert_eq!(is_x86_feature_detected!("avx512cd"), information.avx512cd());
    assert_eq!(is_x86_feature_detected!("avx512er"), information.avx512er());
    assert_eq!(is_x86_feature_detected!("avx512pf"), information.avx512pf());
    assert_eq!(is_x86_feature_detected!("avx512bw"), information.avx512bw());
    assert_eq!(is_x86_feature_detected!("avx512dq"), information.avx512dq());
    assert_eq!(is_x86_feature_detected!("avx512vl"), information.avx512vl());
    assert_eq!(
        is_x86_feature_detected!("avx512ifma"),
        information.avx512_ifma()
    );
    assert_eq!(
        is_x86_feature_detected!("avx512vbmi"),
        information.avx512_vbmi()
    );
    assert_eq!(
        is_x86_feature_detected!("avx512vpopcntdq"),
        information.avx512_vpopcntdq()
    );
    assert_eq!(is_x86_feature_detected!("fma"), information.fma());
    assert_eq!(is_x86_feature_detected!("bmi1"), information.bmi1());
    assert_eq!(is_x86_feature_detected!("bmi2"), information.bmi2());
    assert_eq!(is_x86_feature_detected!("popcnt"), information.popcnt());
    assert_eq!(is_x86_feature_detected!("abm"), information.lzcnt());
    assert_eq!(is_x86_feature_detected!("tbm"), information.tbm());
    assert_eq!(is_x86_feature_detected!("lzcnt"), information.lzcnt());
    assert_eq!(is_x86_feature_detected!("xsave"), information.xsave());
    assert_eq!(is_x86_feature_detected!("xsaveopt"), information.xsaveopt());
    assert_eq!(
        is_x86_feature_detected!("xsavec"),
        information.xsavec_and_xrstor()
    );
    assert_eq!(
        is_x86_feature_detected!("xsaves"),
        information.xsaves_xrstors_and_ia32_xss()
    );
    assert_eq!(
        is_x86_feature_detected!("cmpxchg16b"),
        information.cmpxchg16b(),
    );
    assert_eq!(is_x86_feature_detected!("adx"), information.adx(),);
    assert_eq!(is_x86_feature_detected!("rtm"), information.rtm(),);
    assert_eq!(is_x86_feature_detected!("movbe"), information.movbe(),);
}

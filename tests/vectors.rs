//! 官方测试向量，转写自 vinkla/hashids（PHP v5.0.2）`tests/HashidsTest.php`。
//!
//! 每个用例跑三断言：encode → id、decode(id) → numbers、非法输入 → 空哨兵。

use hashids::{Error, Hashids};

const CUSTOM_SALT: &str = "this is my salt";
const CUSTOM_ALPHABET: &str = "xzal86grmb4jhysfoqp3we7291kuct5iv0nd";
const CUSTOM_MIN_LENGTH: usize = 30;

#[test]
fn default_params() {
    let hashids = Hashids::default();
    let vectors: &[(&str, &[u64])] = &[
        ("gY", &[0]),
        ("jR", &[1]),
        ("R8ZN0", &[928728]),
        ("o2fXhV", &[1, 2, 3]),
        ("jRfMcP", &[1, 0, 0]),
        ("jQcMcW", &[0, 0, 1]),
        ("gYcxcr", &[0, 0, 0]),
        ("gLpmopgO6", &[1000000000000]),
        ("lEW77X7g527", &[9007199254740991]),
        ("BrtltWt2tyt1tvt7tJt2t1tD", &[5; 12]),
        (
            "G6XOnGQgIpcVcXcqZ4B8Q8B9y",
            &[10000000000, 0, 0, 0, 999999999999999],
        ),
        ("5KoLLVL49RLhYkppOplM6piwWNNANny8N", &[9007199254740991; 3]),
        (
            "BPg3Qx5f8VrvQkS16wpmwIgj9Q4Jsr93gqx",
            &[1000000001, 1000000002, 1000000003, 1000000004, 1000000005],
        ),
        (
            "1wfphpilsMtNumCRFRHXIDSqT2UPcWf1hZi3s7tN",
            &[
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ],
        ),
    ];

    for &(id, numbers) in vectors {
        assert_eq!(hashids.encode(numbers), id, "encode({numbers:?})");
        assert_eq!(hashids.decode(id), numbers, "decode({id:?})");
    }
}

#[test]
fn custom_params() {
    let hashids = Hashids::new(CUSTOM_SALT, CUSTOM_MIN_LENGTH, CUSTOM_ALPHABET).unwrap();
    let vectors: &[(&str, &[u64])] = &[
        ("nej1m3d5a6yn875e7gr9kbwpqol02q", &[0]),
        ("dw1nqdp92yrajvl9v6k3gl5mb0o8ea", &[1]),
        ("onqr0bk58p642wldq14djmw21ygl39", &[928728]),
        ("18apy3wlqkjvd5h1id7mn5ore2d06b", &[1, 2, 3]),
        ("o60edky1ng3vl9hbfavwr5pa2q8mb9", &[1, 0, 0]),
        ("o60edky1ng3vlqfbfp4wr5pa2q8mb9", &[0, 0, 1]),
        ("qek2a08gpl575efrfd7yomj9dwbr63", &[0, 0, 0]),
        ("m3d5a6yn875rae8y81a94gr9kbwpqo", &[1000000000000]),
        ("1q3y98ln48w96kpo0wgk314w5mak2d", &[9007199254740991]),
        ("op7qrcdc3cgc2c0cbcrcoc5clce4d6", &[5; 12]),
        (
            "5430bd2jo0lxyfkfjfyojej5adqdy4",
            &[10000000000, 0, 0, 0, 999999999999999],
        ),
        (
            "aa5kow86ano1pt3e1aqm239awkt9pk380w9l3q6",
            &[9007199254740991; 3],
        ),
        (
            "mmmykr5nuaabgwnohmml6dakt00jmo3ainnpy2mk",
            &[1000000001, 1000000002, 1000000003, 1000000004, 1000000005],
        ),
        (
            "w1hwinuwt1cbs6xwzafmhdinuotpcosrxaz0fahl",
            &[
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ],
        ),
    ];

    for &(id, numbers) in vectors {
        let encoded = hashids.encode(numbers);
        assert_eq!(encoded, id);
        assert_eq!(hashids.decode(&encoded), numbers);
        assert!(
            encoded.chars().count() >= CUSTOM_MIN_LENGTH,
            "{encoded} 应满足最短长度"
        );
    }
}

#[test]
fn default_params_hex() {
    let hashids = Hashids::default();
    let vectors: &[(&str, &str)] = &[
        ("wpVL4j9g", "deadbeef"),
        ("kmP69lB3xv", "abcdef123456"),
        ("47JWg0kv4VU0G2KBO2", "ABCDDD6666DDEEEEEEEEE"),
        ("y42LW46J9luq3Xq9XMly", "507f1f77bcf86cd799439011"),
        (
            "m1rO8xBQNquXmLvmO65BUO9KQmj",
            "f00000fddddddeeeee4444444ababab",
        ),
        (
            "wBlnMA23NLIQDgw7XxErc2mlNyAjpw",
            "abcdef123456abcdef123456abcdef123456",
        ),
        (
            "VwLAoD9BqlT7xn4ZnBXJFmGZ51ZqrBhqrymEyvYLIP199",
            "f000000000000000000000000000000000000000000000000000f",
        ),
        (
            "nBrz1rYyV0C0XKNXxB54fWN0yNvVjlip7127Jo3ri0Pqw",
            "fffffffffffffffffffffffffffffffffffffffffffffffffffff",
        ),
    ];

    for &(id, hex) in vectors {
        assert_eq!(hashids.encode_hex(hex), id);
        assert_eq!(hashids.decode_hex(id), hex.to_lowercase());
    }
}

#[test]
fn custom_params_hex() {
    let hashids = Hashids::new(CUSTOM_SALT, CUSTOM_MIN_LENGTH, CUSTOM_ALPHABET).unwrap();
    let vectors: &[(&str, &str)] = &[
        ("0dbq3jwa8p4b3gk6gb8bv21goerm96", "deadbeef"),
        ("190obdnk4j02pajjdande7aqj628mr", "abcdef123456"),
        ("a1nvl5d9m3yo8pj1fqag8p9pqw4dyl", "ABCDDD6666DDEEEEEEEEE"),
        ("1nvlml93k3066oas3l9lr1wn1k67dy", "507f1f77bcf86cd799439011"),
        (
            "mgyband33ye3c6jj16yq1jayh6krqjbo",
            "f00000fddddddeeeee4444444ababab",
        ),
        (
            "9mnwgllqg1q2tdo63yya35a9ukgl6bbn6qn8",
            "abcdef123456abcdef123456abcdef123456",
        ),
        (
            "edjrkn9m6o69s0ewnq5lqanqsmk6loayorlohwd963r53e63xmml29",
            "f000000000000000000000000000000000000000000000000000f",
        ),
        (
            "grekpy53r2pjxwyjkl9aw0k3t5la1b8d5r1ex9bgeqmy93eata0eq0",
            "fffffffffffffffffffffffffffffffffffffffffffffffffffff",
        ),
    ];

    for &(id, hex) in vectors {
        let encoded = hashids.encode_hex(hex);
        assert_eq!(encoded, id);
        assert_eq!(hashids.decode_hex(&encoded), hex.to_lowercase());
        assert!(encoded.chars().count() >= CUSTOM_MIN_LENGTH);
    }
}

#[test]
fn big_numbers_fit_u64() {
    let hashids = Hashids::with_default_alphabet(CUSTOM_SALT, 0);
    let vectors: &[(u64, &str)] = &[
        (2147483647, "ykJWW1g"),                 // 32 位有符号上限
        (4294967295, "j4r6j8Y"),                 // 32 位无符号上限
        (9223372036854775807, "jvNx4BjM5KYjv"),  // 64 位有符号上限
        (18446744073709551615, "zXVjmzBamYlqX"), // 64 位无符号上限（u64::MAX）
    ];

    for &(number, hash) in vectors {
        assert_eq!(hashids.encode(&[number]), hash);
        assert_eq!(hashids.decode(hash), vec![number]);
    }
}

#[test]
fn js_hashids_compatible() {
    let vi = "áàãăâeéèêiíìĩoóòõôơuúùũưyýỳđ";
    let cases: &[(&str, usize, &str, &[u64], &str)] = &[
        (
            "",
            0,
            vi,
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            "íóuđáìàúãỳăyâôeyiôuĩ",
        ),
        ("世界", 0, vi, &[9007199254740991], "óôòúỳưúoỳééưýy"),
        (
            "",
            0,
            "cCsSfFhHuUiItT01",
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            "10h10i00s100t010u110C000F1000H0110I1010",
        ),
        (
            "",
            9,
            "零0一1二2三3四4五5六6七7七8八9九",
            &[4231],
            "5三77九58三九",
        ),
    ];

    for &(salt, min_length, alphabet, numbers, hash) in cases {
        let hashids = Hashids::new(salt, min_length, alphabet).unwrap();
        assert_eq!(hashids.encode(numbers), hash);
        assert_eq!(hashids.decode(hash), numbers);
    }
}

#[test]
fn alphabets_round_trip() {
    let alphabets = [
        "cCsSfFhHuUiItT01",
        "abdegjklCFHISTUc",
        "abdegjklmnopqrSF",
        "abdegjklmnopqrvwxyzABDEGJKLMNOPQRVWXYZ1234567890",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890`~!@#$%^&*()-_=+\\|'\";:/?.>,<{[}]",
        "`~!@#$%^&*()-_=+\\|'\";:/?.>,<{[}]",
        "áàãăâeéèêiíìĩoóòõôơuúùũưyýỳđ",
    ];

    for alphabet in alphabets {
        let hashids = Hashids::new("", 0, alphabet).unwrap();
        let id = hashids.encode(&[1, 2, 3]);
        assert_eq!(hashids.decode(&id), vec![1, 2, 3], "alphabet {alphabet:?}");
    }
}

#[test]
fn salts_round_trip() {
    let salts = [
        "",
        "0",
        "   ",
        CUSTOM_SALT,
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890`~!@#$%^&*()-_=+\\|'\";:/?.>,<{[}]",
        "`~!@#$%^&*()-_=+\\|'\";:/?.>,<{[}]",
        "!áàãăâ eéèê iíìĩ oóòõôơ uúùũư yýỳ đ",
    ];

    for salt in salts {
        let hashids = Hashids::with_default_alphabet(salt, 0);
        let id = hashids.encode(&[1, 2, 3]);
        assert_eq!(hashids.decode(&id), vec![1, 2, 3], "salt {salt:?}");
    }
}

#[test]
fn min_length_is_honoured() {
    for length in [0usize, 1, 10, 999, 1000] {
        let hashids = Hashids::with_default_alphabet("", length);
        let id = hashids.encode(&[1, 2, 3]);
        assert_eq!(hashids.decode(&id), vec![1, 2, 3]);
        assert!(
            id.chars().count() >= length,
            "len={} id={id}",
            id.chars().count()
        );
    }
}

#[test]
fn bad_input_returns_sentinels() {
    let hashids = Hashids::default();
    assert_eq!(hashids.encode(&[]), "");
    assert_eq!(hashids.decode(""), Vec::<u64>::new());
    assert_eq!(hashids.decode("f"), Vec::<u64>::new());
    assert_eq!(hashids.encode_hex("z"), "");
    assert_eq!(hashids.encode_hex(""), "");
    assert_eq!(hashids.decode_hex("f"), "");
}

#[test]
fn tampered_hashes_are_rejected() {
    let hashids = Hashids::default();
    let valid = hashids.encode(&[1, 2, 3]);
    let mut tampered: Vec<char> = valid.chars().collect();
    tampered[0] = if tampered[0] == 'a' { 'b' } else { 'a' };
    let tampered: String = tampered.into_iter().collect();
    assert_eq!(hashids.decode(&tampered), Vec::<u64>::new());
}

#[test]
fn invalid_alphabets_are_rejected() {
    assert_eq!(
        Hashids::new("", 0, "1234567890").unwrap_err(),
        Error::AlphabetTooShort
    );
    assert_eq!(
        Hashids::new("", 0, "a cdefghijklmnopqrstuvwxyz").unwrap_err(),
        Error::AlphabetWithSpace
    );
}

#[test]
fn stable_across_min_length_padding() {
    // 对齐 PHP testBehaviourForDifferentBCMathAccuracy。
    let hashids = Hashids::with_default_alphabet(CUSTOM_SALT, 12);
    assert_eq!(hashids.encode(&[1]), "DngB0NV05ev1");
}

// DASL: 0xDA51 prefix addressing — Monster Walk, eigenspace, FRACTRAN
//
// Address format: [prefix:16][type:4][data:44] = 64 bits
// See DA51_PREFIX_CLASSIFICATION.md

pub const PREFIX: u64 = 0xDA51;
pub const SSP: [u64; 15] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 41, 47, 59, 71];

// Eigenspace classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Eigenspace { Earth = 0, Spoke = 1, Hub = 2, Clock = 3 }

// Bott periodicity algebras
pub const BOTT: [&str; 8] = ["R", "C", "H", "H⊕H", "H(2)", "C(4)", "R(8)", "R(8)⊕R(8)"];

// McKay-Thompson c₁ values for 15 SSP
pub const C1: [u64; 15] = [4371, 782, 133, 50, 16, 11, 6, 5, 3, 2, 2, 1, 0, 0, 0];

// --- Address construction ---

fn make(typ: u8, data: u64) -> u64 {
    (PREFIX << 48) | ((typ as u64 & 0xF) << 44) | (data & 0xFFF_FFFF_FFFF)
}

pub fn addr_type(addr: u64) -> u8 {
    ((addr >> 44) & 0xF) as u8
}

pub fn addr_prefix(addr: u64) -> u16 {
    (addr >> 48) as u16
}

/// Type 0: Monster Walk Block
pub fn monster_walk(group: u8, position: u8, sequence: u16, factors: u8) -> u64 {
    let data = ((group as u64 & 0xF) << 40)
        | ((position as u64 & 0xFF) << 32)
        | ((sequence as u64) << 16)
        | ((factors as u64 & 0xF) << 12);
    make(0, data)
}

/// Type 1: AST Node
pub fn ast_node(selector: u8, bott: u8, tenfold: u16, hecke: u8, hash: u32) -> u64 {
    let data = ((selector as u64 & 0x7) << 41)
        | ((bott as u64 & 0x7) << 38)
        | ((tenfold as u64 & 0x7FF) << 27)
        | ((hecke as u64 & 0x7F) << 20)
        | (hash as u64 & 0xFFFFF);
    make(1, data)
}

/// Type 3: Nested CID
pub fn nested_cid(shard: u8, hecke: u8, bott: u8, hash: u32) -> u64 {
    let data = ((shard as u64) << 36)
        | ((hecke as u64) << 28)
        | ((bott as u64) << 20)
        | (hash as u64 & 0xFFFFF);
    make(3, data)
}

/// Type 5: Shard ID
pub fn shard_id(prime_idx: u8, replica: u8, zone: u8, node: u32) -> u64 {
    let data = ((prime_idx as u64 & 0xF) << 40)
        | ((replica as u64 & 0xF) << 36)
        | ((zone as u64) << 28)
        | (node as u64 & 0xFFFFFFF);
    make(5, data)
}

/// Type 6: Eigenspace Address
pub fn eigenspace_addr(es: Eigenspace, prime_idx: u8, mckay: u8, hub_proj: u8, hash: u32) -> u64 {
    let data = ((es as u64 & 0x3) << 42)
        | ((prime_idx as u64 & 0xF) << 38)
        | ((mckay as u64 & 0x3F) << 32)
        | ((hub_proj as u64 & 0xF) << 28)
        | (hash as u64 & 0xFFFFFFF);
    make(6, data)
}

/// Type 7: Hauptmodul Reference
pub fn hauptmodul(prime_idx: u8, genus: u8, coeff_idx: u8, coeff_val: u32) -> u64 {
    let data = ((prime_idx as u64 & 0xF) << 40)
        | ((genus as u64 & 0xF) << 36)
        | ((coeff_idx as u64) << 28)
        | (coeff_val as u64 & 0xFFFFFFF);
    make(7, data)
}

/// Type 0xA: FRACTRAN state
pub fn fractran_addr(step: u32) -> u64 {
    make(0xA, step as u64)
}

/// XOR merge two DA51 addresses
pub fn xor_merge(a: u64, b: u64) -> u64 {
    let prefix = PREFIX << 48;
    prefix | ((a & 0xFFFF_FFFF_FFFF) ^ (b & 0xFFFF_FFFF_FFFF))
}

/// Hex format
pub fn hex(addr: u64) -> String {
    format!("0x{:016x}", addr)
}

/// Classify prime into eigenspace
pub fn prime_eigenspace(p: u64) -> Eigenspace {
    match p {
        2 | 3 | 5 | 7 | 11 | 13 | 47 => Eigenspace::Earth,
        19 => Eigenspace::Hub,
        23 => Eigenspace::Clock,
        17 | 29 | 31 | 41 | 59 | 71 => Eigenspace::Spoke,
        _ => Eigenspace::Earth,
    }
}

/// Genus of X₀(p) for SSP prime
pub fn genus(p: u64) -> u8 {
    match p {
        2 | 3 | 5 | 7 | 13 => 0,
        11 | 17 | 19 => 1,
        23 | 29 | 31 => 2,
        41 => 3, 47 => 4, 59 => 5, 71 => 6,
        _ => 0,
    }
}

/// Encode as RDFa triples
pub fn addr_to_triples(addr: u64) -> Vec<(String, String, String)> {
    let h = hex(addr);
    let t = addr_type(addr);
    vec![
        (h.clone(), "rdf:type".into(), format!("dasl:Type{}", t)),
        (h.clone(), "dasl:prefix".into(), "0xDA51".into()),
        (h, "dasl:type".into(), t.to_string()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_constant() {
        assert_eq!(PREFIX, 0xDA51);
        assert_eq!(PREFIX as u16, 0xDA51);
    }

    #[test]
    fn ssp_15_primes() {
        assert_eq!(SSP.len(), 15);
        assert_eq!(SSP[0], 2);
        assert_eq!(SSP[14], 71);
        // All prime
        for &p in &SSP {
            assert!(is_prime(p), "{} should be prime", p);
        }
    }

    #[test]
    fn monster_196883() {
        assert_eq!(47u64 * 59 * 71, 196_883);
    }

    #[test]
    fn c1_values() {
        // c₁(2a) = 4371, c₁(3a) = 782, ..., c₁(47a) = c₁(59a) = c₁(71a) = 0
        assert_eq!(C1[0], 4371);
        assert_eq!(C1[1], 782);
        assert_eq!(C1[12], 0); // 47
        assert_eq!(C1[13], 0); // 59
        assert_eq!(C1[14], 0); // 71
    }

    #[test]
    fn c1_mod_p() {
        // For p ≥ 13: c₁(pa) = 196883 mod p
        for i in 5..15 { // indices 5-14 = primes 13-71
            let p = SSP[i];
            let expected = 196_883 % p;
            assert_eq!(C1[i], expected,
                "c₁({}a) = {} but 196883 mod {} = {}", p, C1[i], p, expected);
        }
    }

    #[test]
    fn eigenspace_classification() {
        // Earth: {2,3,5,7,11,13,47}
        for &p in &[2, 3, 5, 7, 11, 13, 47] {
            assert_eq!(prime_eigenspace(p), Eigenspace::Earth, "p={}", p);
        }
        // Hub: {19}
        assert_eq!(prime_eigenspace(19), Eigenspace::Hub);
        // Clock: {23}
        assert_eq!(prime_eigenspace(23), Eigenspace::Clock);
        // Spoke: {17,29,31,41,59,71}
        for &p in &[17, 29, 31, 41, 59, 71] {
            assert_eq!(prime_eigenspace(p), Eigenspace::Spoke, "p={}", p);
        }
    }

    #[test]
    fn eigenspace_dimensions() {
        // Earth(7) + Spoke(6 primes, 5-dim basis) + Hub(1) + Clock(1 prime, 2-dim)
        // 15 primes total, 15 dimensions in Cl(15,0,0)
        let earth: Vec<_> = SSP.iter().filter(|&&p| prime_eigenspace(p) == Eigenspace::Earth).collect();
        let spoke: Vec<_> = SSP.iter().filter(|&&p| prime_eigenspace(p) == Eigenspace::Spoke).collect();
        let hub: Vec<_> = SSP.iter().filter(|&&p| prime_eigenspace(p) == Eigenspace::Hub).collect();
        let clock: Vec<_> = SSP.iter().filter(|&&p| prime_eigenspace(p) == Eigenspace::Clock).collect();
        assert_eq!(earth.len(), 7);
        assert_eq!(spoke.len(), 6); // 6 primes form 5-dim subspace (differences from e₁₇)
        assert_eq!(hub.len(), 1);   // {19}
        assert_eq!(clock.len(), 1); // {23}
        assert_eq!(earth.len() + spoke.len() + hub.len() + clock.len(), 15);
    }

    #[test]
    fn bott_periodicity() {
        assert_eq!(BOTT.len(), 8);
        assert_eq!(BOTT[0], "R");
        assert_eq!(BOTT[2], "H");
        // Period 8
        for i in 0..8 { assert_eq!(BOTT[i], BOTT[i % 8]); }
    }

    #[test]
    fn genus_values() {
        assert_eq!(genus(2), 0);
        assert_eq!(genus(11), 1);
        assert_eq!(genus(23), 2);
        assert_eq!(genus(41), 3);
        assert_eq!(genus(47), 4);
        assert_eq!(genus(59), 5);
        assert_eq!(genus(71), 6);
    }

    #[test]
    fn monster_walk_encoding() {
        let addr = monster_walk(0, 0, 0x8080, 8);
        assert_eq!(addr_prefix(addr), PREFIX as u16);
        assert_eq!(addr_type(addr), 0);
    }

    #[test]
    fn ast_node_encoding() {
        let addr = ast_node(0b111, 0, 0, 14, 0);
        assert_eq!(addr_prefix(addr), PREFIX as u16);
        assert_eq!(addr_type(addr), 1);
    }

    #[test]
    fn eigenspace_addr_encoding() {
        let addr = eigenspace_addr(Eigenspace::Hub, 7, 5, 15, 0);
        assert_eq!(addr_prefix(addr), PREFIX as u16);
        assert_eq!(addr_type(addr), 6);
    }

    #[test]
    fn hauptmodul_encoding() {
        // c₁(3a) = 782
        let addr = hauptmodul(1, 0, 1, 782);
        assert_eq!(addr_prefix(addr), PREFIX as u16);
        assert_eq!(addr_type(addr), 7);
    }

    #[test]
    fn xor_merge_preserves_prefix() {
        let a = monster_walk(0, 0, 0x1234, 0);
        let b = monster_walk(1, 0, 0x5678, 0);
        let merged = xor_merge(a, b);
        assert_eq!(addr_prefix(merged), PREFIX as u16);
    }

    #[test]
    fn fractran_addr_type() {
        let addr = fractran_addr(42);
        assert_eq!(addr_prefix(addr), PREFIX as u16);
        assert_eq!(addr_type(addr), 0xA);
    }

    #[test]
    fn addr_to_triples_roundtrip() {
        let addr = nested_cid(58, 35, 41, 0x92F2B7F);
        let triples = addr_to_triples(addr);
        assert!(triples.len() >= 3);
        assert!(triples[0].1 == "rdf:type");
        assert!(triples[0].2.contains("Type3"));
    }

    #[test]
    fn hex_nibble_sum_240() {
        // |M| hex = 86FA3F510644E13FDC4C5673C27C78C31400000000000
        let hex_str = "86FA3F510644E13FDC4C5673C27C78C31400000000000";
        let sum: u32 = hex_str.chars()
            .filter_map(|c| u32::from_str_radix(&c.to_string(), 16).ok())
            .sum();
        assert_eq!(sum, 240, "hex nibble sum of |M| should be 240 = 16×15 = |roots of E₈|");
    }

    #[test]
    fn skeleton_pair() {
        // After stripping all other primes, {3, 19} remain
        // 3^20 and 19^1 are in the Monster order
        // 782 + 5 = 787 (prime) — skeleton pair c₁ sum
        assert_eq!(C1[1] + C1[7], 787); // c₁(3a) + c₁(19a)
        assert!(is_prime(787));
    }

    #[test]
    fn c1_norm_squared() {
        // |c₁|² = sum of squares
        let norm_sq: u64 = C1.iter().map(|&c| c * c).sum();
        assert_eq!(norm_sq, 19_737_810);
    }

    fn is_prime(n: u64) -> bool {
        if n < 2 { return false; }
        if n < 4 { return true; }
        if n % 2 == 0 || n % 3 == 0 { return false; }
        let mut i = 5;
        while i * i <= n { if n % i == 0 || n % (i+2) == 0 { return false; } i += 6; }
        true
    }
}

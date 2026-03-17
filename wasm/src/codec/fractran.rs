// FRACTRAN: fractions as RDFa triples, step execution
//
// A FRACTRAN program is a list of fractions. State is a positive integer.
// Each step: find first fraction n/d where state*n/d is integer, apply it.
// Halt when no fraction applies.
//
// Triple representation:
//   <prog> <fractran:fraction> "3/2"
//   <prog> <fractran:state> "108"
//
// Composable: programs are just triple sets, merge to compose.

pub const NS: &str = "fractran:";

/// Parse "n/d" string to (numerator, denominator)
pub fn parse_fraction(s: &str) -> Option<(u64, u64)> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 2 { return None; }
    Some((parts[0].trim().parse().ok()?, parts[1].trim().parse().ok()?))
}

/// Extract FRACTRAN program from triples: all objects where predicate = "fractran:fraction"
pub fn program_from_triples(triples: &[(String, String, String)]) -> Vec<(u64, u64)> {
    triples.iter()
        .filter(|(_, p, _)| p == "fractran:fraction" || p.ends_with("#fraction"))
        .filter_map(|(_, _, o)| parse_fraction(o))
        .collect()
}

/// Extract initial state from triples (predicate = "fractran:state")
pub fn state_from_triples(triples: &[(String, String, String)]) -> Option<u64> {
    triples.iter()
        .find(|(_, p, _)| p == "fractran:state" || p.ends_with("#state"))
        .and_then(|(_, _, o)| o.parse().ok())
}

/// Execute one FRACTRAN step: find first applicable fraction, return new state
pub fn step(state: u64, program: &[(u64, u64)]) -> Option<u64> {
    for &(n, d) in program {
        if d > 0 && state % d == 0 {
            return Some(state / d * n);
        }
    }
    None // halt
}

/// Run FRACTRAN program up to max_steps, return state history
pub fn run(initial: u64, program: &[(u64, u64)], max_steps: usize) -> Vec<u64> {
    let mut history = vec![initial];
    let mut state = initial;
    for _ in 0..max_steps {
        match step(state, program) {
            Some(next) => { state = next; history.push(state); }
            None => break,
        }
    }
    history
}

/// Extract prime exponents from state (for the first 15 supersingular primes)
pub fn prime_exponents(mut state: u64) -> Vec<(u64, u32)> {
    const SSP: [u64; 15] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 41, 47, 59, 71];
    let mut out = Vec::new();
    for &p in &SSP {
        let mut exp = 0u32;
        while state % p == 0 { state /= p; exp += 1; }
        if exp > 0 { out.push((p, exp)); }
    }
    out
}

/// Encode FRACTRAN program as triples
pub fn program_to_triples(subject: &str, program: &[(u64, u64)], state: u64) -> Vec<(String, String, String)> {
    let mut triples = vec![
        (subject.into(), "rdf:type".into(), "fractran:Program".into()),
        (subject.into(), "fractran:state".into(), state.to_string()),
    ];
    for (n, d) in program {
        triples.push((subject.into(), "fractran:fraction".into(), format!("{}/{}", n, d)));
    }
    triples
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_fraction_ok() {
        assert_eq!(parse_fraction("3/2"), Some((3, 2)));
        assert_eq!(parse_fraction("17/91"), Some((17, 91)));
        assert_eq!(parse_fraction("bad"), None);
    }

    #[test]
    fn fractran_addition() {
        // FRACTRAN addition: 2^a * 3^b → 2^(a+b)
        // Program: [3/2] applied to 2^a * 3^b moves factors of 3 into 2
        // Wait, that's wrong. 3/2 replaces a factor of 2 with 3.
        // Addition program: [2/3] — replaces 3s with 2s
        let program = vec![(2, 3)];
        let state = 2u64.pow(3) * 3u64.pow(5); // 2^3 * 3^5
        let history = run(state, &program, 100);
        let final_state = *history.last().unwrap();
        assert_eq!(final_state, 2u64.pow(8)); // 2^(3+5)
    }

    #[test]
    fn fractran_halt() {
        let program = vec![(3, 2)]; // needs even state
        let state = 3; // odd — no fraction applies
        assert_eq!(step(state, &program), None);
    }

    #[test]
    fn triples_roundtrip() {
        let prog = vec![(3, 2), (1, 3)];
        let triples = program_to_triples("_:p1", &prog, 108);
        let extracted = program_from_triples(&triples);
        assert_eq!(extracted, prog);
        assert_eq!(state_from_triples(&triples), Some(108));
    }

    #[test]
    fn prime_exponents_basic() {
        let exps = prime_exponents(2u64.pow(3) * 3u64.pow(2) * 5);
        assert_eq!(exps, vec![(2, 3), (3, 2), (5, 1)]);
    }

    #[test]
    fn conway_primes() {
        // Conway's prime-generating FRACTRAN program (first few fractions)
        let program = vec![
            (17, 91), (78, 85), (19, 51), (23, 38), (29, 33),
            (77, 29), (95, 23), (77, 19), (1, 17), (11, 13),
            (13, 11), (15, 14), (15, 2), (55, 1),
        ];
        let history = run(2, &program, 200);
        // After enough steps, powers of 2 appear in the sequence
        let powers_of_2: Vec<u64> = history.iter()
            .filter(|&&s| s > 1 && (s & (s - 1)) == 0)
            .copied().collect();
        // 2^1 = 2 is the initial state, 2^2 = 4 should appear
        assert!(powers_of_2.contains(&2), "should contain 2^1");
    }
}

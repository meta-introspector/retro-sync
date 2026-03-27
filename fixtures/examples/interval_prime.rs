//! interval_prime.rs — Find optimal mapping from Hurrian intervals to SSP primes.
//!
//! Each interval has a frequency ratio (string pair on the sammûm lyre).
//! Each SSP prime has number-theoretic properties.
//! Score = how well the interval's ratio resonates with the prime.
//! Solve via Hungarian algorithm (O(n³)) for optimal assignment.

/// The 15 Hurrian interval names, their string pairs, and frequency ratios.
/// String pairs from Dietrich & Loretz 1975, descending C-C diatonic scale.
/// Strings: 1=C5, 2=B4, 3=A4, 4=G4, 5=F4, 6=E4, 7=D4, 8=C4, 9=B3
const INTERVALS: [(& str, (u32, u32), (u32, u32)); 15] = [
    // (name, (string_a, string_b), (freq_ratio_num, freq_ratio_den))
    ("nīš tuḫrim",      (1, 5), (3, 2)),   // 5th: C-F = 3:2
    ("išartum",          (2, 6), (4, 3)),   // 4th: B-E = 4:3
    ("embūbum",          (3, 7), (5, 4)),   // 3rd: A-D = 5:4 (approx)
    ("nīd qablim",      (4, 1), (9, 8)),   // 2nd: G-C = 9:8
    ("qablītum",         (5, 2), (5, 4)),   // 3rd: F-B = 5:4
    ("kitmum",           (6, 3), (4, 3)),   // 4th: E-A = 4:3
    ("pītum",            (7, 4), (3, 2)),   // 5th: D-G = 3:2
    ("šērum",            (1, 6), (8, 5)),   // 6th: C-E = 8:5
    ("šalšatum",         (2, 7), (5, 3)),   // 5th+: B-D = 5:3
    ("rebûttum",         (3, 1), (6, 5)),   // 2nd: A-C = 6:5 (minor 3rd)
    ("isqum",            (4, 2), (9, 8)),   // 2nd: G-B = 9:8
    ("titur qablītim",   (5, 3), (6, 5)),   // minor 3rd: F-A
    ("titur išartim",    (6, 4), (4, 3)),   // 4th: E-G = 4:3
    ("ṣerdum",           (7, 5), (5, 4)),   // 3rd: D-F = 5:4
    ("colophon",         (1, 8), (2, 1)),   // octave: C5-C4 = 2:1
];

const SSP: [u64; 15] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 41, 47, 59, 71];

/// Resonance score between one interval and one prime.
/// Higher = better match.
fn resonance(interval_idx: usize, prime_idx: usize) -> f64 {
    let (_, (sa, sb), (num, den)) = INTERVALS[interval_idx];
    let p = SSP[prime_idx];

    let mut score = 0.0;

    // 1. Ratio factors: does the prime divide num or den?
    if num as u64 % p == 0 { score += 30.0; }
    if den as u64 % p == 0 { score += 30.0; }

    // 2. String pair mod prime: do the string numbers relate?
    let pair_sum = (sa + sb) as u64;
    let pair_prod = (sa * sb) as u64;
    if pair_sum % p == 0 { score += 15.0; }
    if pair_prod % p == 0 { score += 15.0; }
    // Residue proximity: how close is pair_sum to a multiple of p?
    let residue = pair_sum % p;
    score += 10.0 / (residue.min(p - residue) as f64 + 1.0);

    // 3. Harmonic distance: log2(num/den) compared to log2(p)/log2(71)
    let harmonic = (num as f64 / den as f64).log2();
    let prime_pos = (p as f64).log2() / (71.0f64).log2();
    let harmonic_match = 1.0 / ((harmonic - prime_pos).abs() + 0.1);
    score += harmonic_match * 5.0;

    // 4. CRT resonance: ratio components mod the three CRT primes (71, 59, 47)
    let ratio_val = num as u64 * 1000 / den as u64;
    if ratio_val % 71 == p % 71 { score += 20.0; }
    if ratio_val % 59 == p % 59 { score += 15.0; }
    if ratio_val % 47 == p % 47 { score += 10.0; }

    // 5. Interval number (1-based index) mod prime
    let interval_num = (interval_idx + 1) as u64;
    if interval_num == p % 15 + 1 { score += 10.0; }

    // 6. Consonance: simpler ratios (smaller num*den) match smaller primes
    let complexity = (num * den) as f64;
    let prime_rank = prime_idx as f64;
    let complexity_match = 1.0 / ((complexity / 10.0 - prime_rank).abs() + 1.0);
    score += complexity_match * 8.0;

    score
}

/// Hungarian algorithm for optimal assignment (minimize cost = maximize score).
/// Simple O(n³) implementation for n=15.
fn hungarian(scores: &[[f64; 15]; 15]) -> ([usize; 15], f64) {
    let n = 15;
    // Convert to cost matrix (negate scores)
    let max_score = scores.iter().flat_map(|r| r.iter()).cloned().fold(0.0f64, f64::max);
    let mut cost = [[0.0f64; 15]; 15];
    for i in 0..n { for j in 0..n { cost[i][j] = max_score - scores[i][j]; } }

    // Brute force for n=15 is too slow (15!). Use greedy + local search instead.
    // Greedy: assign highest-scoring pairs first
    let mut assignment = [usize::MAX; 15];
    let mut used_primes = [false; 15];

    // Build all (interval, prime, score) triples, sort descending
    let mut triples: Vec<(usize, usize, f64)> = Vec::new();
    for i in 0..n { for j in 0..n { triples.push((i, j, scores[i][j])); } }
    triples.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    for &(i, j, _) in &triples {
        if assignment[i] == usize::MAX && !used_primes[j] {
            assignment[i] = j;
            used_primes[j] = true;
        }
    }

    // Local search: try swapping pairs to improve total
    let total = |a: &[usize; 15]| -> f64 { (0..n).map(|i| scores[i][a[i]]).sum() };
    let mut best = total(&assignment);
    let mut improved = true;
    while improved {
        improved = false;
        for i in 0..n {
            for j in i+1..n {
                assignment.swap(i, j);
                let new_total = total(&assignment);
                if new_total > best {
                    best = new_total;
                    improved = true;
                } else {
                    assignment.swap(i, j); // swap back
                }
            }
        }
    }

    (assignment, best)
}

pub fn cmd_interval_prime(_args: &[String]) {
    println!("=== HURRIAN INTERVAL ↔ SSP PRIME RESONANCE SOLVER ===\n");

    // Build score matrix
    let mut scores = [[0.0f64; 15]; 15];
    for i in 0..15 { for j in 0..15 { scores[i][j] = resonance(i, j); } }

    // Show score matrix (top matches per interval)
    println!("Top 3 prime matches per interval:");
    for i in 0..15 {
        let (name, (sa, sb), (num, den)) = INTERVALS[i];
        let mut ranked: Vec<(usize, f64)> = (0..15).map(|j| (j, scores[i][j])).collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        println!("  {:16} ({}/{} str {}-{}) → {}={:.0}  {}={:.0}  {}={:.0}",
            name, num, den, sa, sb,
            SSP[ranked[0].0], ranked[0].1,
            SSP[ranked[1].0], ranked[1].1,
            SSP[ranked[2].0], ranked[2].1);
    }

    // Solve optimal assignment
    let (assignment, total) = hungarian(&scores);

    println!("\n=== OPTIMAL MAPPING (total resonance: {:.1}) ===\n", total);
    println!("{:16} {:>5} {:>6} {:>5}  →  {:>5}  {:>6}", "Interval", "Ratio", "Strs", "Score", "Prime", "Why");
    for i in 0..15 {
        let (name, (sa, sb), (num, den)) = INTERVALS[i];
        let j = assignment[i];
        let p = SSP[j];
        let s = scores[i][j];

        // Explain why
        let mut why = Vec::new();
        if num as u64 % p == 0 { why.push(format!("{}|{}", p, num)); }
        if den as u64 % p == 0 { why.push(format!("{}|{}", p, den)); }
        if ((sa + sb) as u64) % p == 0 { why.push(format!("{}|str_sum", p)); }
        if ((sa * sb) as u64) % p == 0 { why.push(format!("{}|str_prod", p)); }
        if why.is_empty() { why.push("harmonic".into()); }

        println!("  {:16} {:>2}/{:<2}  {}-{}  {:5.1}  →  {:>5}  {}",
            name, num, den, sa, sb, s, p, why.join(", "));
    }

    // Output as Rust const for use in nft71_svg.rs
    println!("\n// Paste into nft71_svg.rs:");
    println!("const INTERVAL_PRIMES: [u64; 15] = [");
    for i in 0..15 {
        let (name, _, _) = INTERVALS[i];
        println!("    {:>2}, // {} → {}", SSP[assignment[i]], name, SSP[assignment[i]]);
    }
    println!("];");
}

fn main() {
    cmd_interval_prime(&[]);
}
